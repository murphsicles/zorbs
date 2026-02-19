// src/handlers/detail.rs
use axum::extract::{State, Path};
use maud::{html, Markup, PreEscaped};
use std::sync::Arc;
use crate::state::AppState;
use crate::db::queries;
use crate::models::Zorb;
use crate::views;

pub async fn zorb_detail(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Markup {
    render_detail(name, state).await
}

pub async fn zorb_detail_scoped(
    Path((scope, name)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Markup {
    let full_name = format!("@{}/{}", scope, name);
    render_detail(full_name, state).await
}

async fn render_detail(name: String, state: Arc<AppState>) -> Markup {
    let versions: Vec<Zorb> = queries::get_zorb_versions(&state.db, &name).await;

    if versions.is_empty() {
        return html! { (PreEscaped(include_str!("../../views/404.html"))) };
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

    html! { (PreEscaped(page)) }
}
