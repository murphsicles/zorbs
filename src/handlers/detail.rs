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
    let download_url = format!("/{}/{}/download", name.replace('@', "").replace('/', "-"), latest.version);
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
