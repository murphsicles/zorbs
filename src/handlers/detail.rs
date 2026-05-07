// src/handlers/detail.rs
use axum::extract::{State, Path};
use axum_login::AuthSession;
use maud::{html, Markup, PreEscaped};
use std::sync::Arc;
use crate::state::AppState;
use crate::db::queries;
use crate::models::Zorb;
use crate::views;
use crate::models::user::UserBackend;

pub async fn zorb_detail(
    auth_session: AuthSession<UserBackend>,
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Markup {
    render_detail(auth_session, name, state).await
}

pub async fn zorb_detail_scoped(
    auth_session: AuthSession<UserBackend>,
    Path((scope, name)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Markup {
    let full_name = format!("@{}/{}", scope, name);
    render_detail(auth_session, full_name, state).await
}

async fn render_detail(auth_session: AuthSession<UserBackend>, name: String, state: Arc<AppState>) -> Markup {
    let versions: Vec<Zorb> = queries::get_zorb_versions(&state.db, &name).await;
    if versions.is_empty() {
        return html! { (PreEscaped(include_str!("../views/404.html"))) };
    }
    let latest = &versions[0];
    let download_url = format!("/{}/{}/download", name, latest.version);
    let mut page = views::DETAIL_HTML.to_string();
    page = page.replace("{{name}}", &latest.name);
    page = page.replace("{{latest.version}}", &latest.version);
    page = page.replace("{{latest.downloads}}", &latest.downloads.to_string());
    page = page.replace(
        "{{latest.description}}",
        latest.description.as_deref().unwrap_or("No description provided.")
    );
    page = page.replace(
        "{{latest.license}}",
        latest.license.as_deref().unwrap_or("No license specified")
    );
    page = page.replace(
        "{{latest.repository}}",
        latest.repository.as_deref().unwrap_or("#")
    );
    page = page.replace(
        "{{latest.created_at}}",
        &latest.created_at.format("%b %d, %Y").to_string()
    );
    page = page.replace("href=\"#\"", &format!("href=\"{}\"", download_url));

    // Fetch and cache README from GitHub if not already stored
    let readme_html = if let Some(readme) = &latest.readme {
        readme.clone()
    } else if let Some(repo_url) = &latest.repository {
        if repo_url.contains("github.com") {
            let raw_url = repo_url
                .replace("https://github.com/", "https://raw.githubusercontent.com/")
                .trim_end_matches('/').to_string()
                + "/main/README.md";
            match reqwest::get(&raw_url).await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        if !text.is_empty() {
                            let _ = sqlx::query!(
                                "UPDATE zorbs SET readme = $1 WHERE name = $2 AND version = $3",
                                text,
                                latest.name,
                                latest.version
                            )
                            .execute(&state.db)
                            .await;
                            text
                        } else { String::new() }
                    } else { String::new() }
                }
                Err(_) => String::new(),
            }
        } else { String::new() }
    } else { String::new() };

    let readme_section = if readme_html.is_empty() {
        String::new()
    } else {
        let escaped = readme_html
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace('$', "\\$");
        format!(
            r##"<div class="mt-16 bg-zinc-900 border border-zinc-800 rounded-3xl p-10">
            <h2 class="text-3xl font-semibold mb-8 flex items-center gap-3">
                <span class="text-cyan-400">📖</span> README
            </h2>
            <div id="readme-content" class="markdown-body"></div>
        </div>
        <script>
            if (typeof marked !== "undefined") {{
                const readme = `{escaped}`;
                document.getElementById("readme-content").innerHTML = marked.parse(readme);
            }}
        </script>"##
        )
    };
    page = page.replace("<!-- README_SECTION -->", &readme_section);

    // Build dynamic version history rows from DB
    let version_rows: String = versions.iter().map(|v| {
        let dl_url = format!("/{}/{}/download", name, v.version);
        let downloads_str = if v.downloads >= 1_000_000 {
            format!("{:.1}M downloads", v.downloads as f64 / 1_000_000.0)
        } else if v.downloads >= 1_000 {
            format!("{:.1}k downloads", v.downloads as f64 / 1_000.0)
        } else {
            format!("{} downloads", v.downloads)
        };
        format!(
            r##"<tr class="hover:bg-zinc-800 transition">
                            <td class="px-8 py-6 font-mono text-cyan-400">{version}</td>
                            <td class="px-8 py-6 text-zinc-400">{date}</td>
                            <td class="px-8 py-6 text-zinc-400">{downloads}</td>
                            <td class="px-8 py-6 text-right">
                                <a href="{dl_url}" class="text-cyan-400 hover:text-cyan-300 font-medium flex items-center justify-end gap-2">
                                    Download
                                    <i class="fa-solid fa-arrow-down"></i>
                                </a>
                            </td>
                        </tr>"##,
            version = v.version,
            date = v.created_at.format("%b %d, %Y").to_string(),
            downloads = downloads_str,
            dl_url = dl_url,
        )
    }).collect::<Vec<_>>().join("\n");
    page = page.replace("<!-- VERSION_HISTORY_ROWS -->", &version_rows);

    // Build dynamic dependency cards from DB
    let dep_map = latest.dependencies_map();
    let dep_cards: String = if dep_map.is_empty() {
        r##"<div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6 col-span-full">
                    <p class="text-zinc-500 text-sm">No dependencies</p>
                </div>"##.to_string()
    } else {
        let mut cards = Vec::new();
        for (dep_name, version_req) in &dep_map {
            let dep_info = queries::get_latest_zorb(&state.db, dep_name).await;
            let desc = dep_info.as_ref()
                .and_then(|d| d.description.as_deref())
                .unwrap_or("Zeta package");
            let href = format!("/{}", dep_name);
            cards.push(format!(
                r##"<div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6">
                    <a href="{href}" class="font-mono text-cyan-400 hover:text-cyan-300">{dep_name} {version_req}</a>
                    <p class="text-xs text-zinc-500 mt-1">{desc}</p>
                </div>"##,
                href = href,
                dep_name = dep_name,
                version_req = version_req,
                desc = desc,
            ));
        }
        cards.join("\n")
    };
    page = page.replace("<!-- DEPENDENCY_CARDS -->", &dep_cards);

    // dynamic nav with Passkey-ready modal trigger
    let user = &auth_session.user;
    let auth_html = if let Some(user) = user {
        html! {
            div class="flex items-center gap-6" {
                span class="text-sm font-medium text-zinc-300" { "@" (user.username) }
                a href="/auth/logout" class="px-6 py-3 bg-red-500/10 hover:bg-red-500/20 text-red-400 font-medium rounded-2xl transition-all" {
                    "Logout"
                }
            }
        }
    } else {
        html! {
            button onclick="openLoginModal()" class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2" {
                "Sign in"
                i class="fa-solid fa-right-to-bracket" {}
            }
        }
    };
    if let Some(pos) = page.find("<!-- AUTH_SLOT -->") {
        page.replace_range(pos..pos + "<!-- AUTH_SLOT -->".len(), &auth_html.into_string());
    }

    html! { (PreEscaped(page)) }
}
