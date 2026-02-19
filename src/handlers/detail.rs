use axum::extract::{State, Path};
use maud::{html, Markup, PreEscaped};
use std::sync::Arc;

use crate::state::AppState;
use crate::db::queries;
use crate::models::Zorb;
use crate::views::detail::DETAIL_HTML;

pub async fn zorb_detail(Path(name): Path<String>, State(state): State<Arc<AppState>>) -> Markup {
    let versions: Vec<Zorb> = queries::get_zorb_versions(&state.db, &name).await;

    if versions.is_empty() {
        return html! { (PreEscaped(include_str!("../../views/404.html"))) };
    }

    html! { (PreEscaped(DETAIL_HTML)) }
}
