use axum::{
    extract::{Multipart, Query, State},
    response::Html,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Zorb {
    pub name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
    pub downloads: u32,
    pub created_at: String,
}

pub type AppState = Arc<Mutex<Vec<Zorb>>>;

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

pub fn initial_state() -> AppState {
    Arc::new(Mutex::new(vec![
        Zorb{name:"@zeta/tokio".into(),version:"1.42.0".into(),description:"The async runtime that powers Zeta".into(),license:"MIT".into(),repository:Some("https://github.com/zeta-lang/tokio".into()),downloads:428000,created_at:"2026-02-01".into()},
        Zorb{name:"@http/axum".into(),version:"0.8.1".into(),description:"Ergonomic web framework".into(),license:"MIT".into(),repository:Some("https://github.com/zeta-lang/axum".into()),downloads:312000,created_at:"2026-01-15".into()},
        Zorb{name:"@data/serde".into(),version:"1.0.210".into(),description:"Fast & safe serialization".into(),license:"MIT OR Apache-2.0".into(),repository:Some("https://github.com/zeta-lang/serde".into()),downloads:289000,created_at:"2026-01-10".into()},
        Zorb{name:"@logging/tracing".into(),version:"0.2.5".into(),description:"Structured, performant logging".into(),license:"MIT".into(),repository:Some("https://github.com/zeta-lang/tracing".into()),downloads:197000,created_at:"2026-02-05".into()},
    ]))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_home))
        .route("/publish", get(serve_publish))
        .route("/api/search", get(search_handler))
        .route("/api/zorbs/new", post(publish_handler))
        .with_state(state)
}

async fn serve_home() -> Html<&'static str> {
    Html(include_str!("../views/home.html"))
}

async fn serve_publish() -> Html<&'static str> {
    Html(include_str!("../views/publish.html"))
}

async fn search_handler(Query(params): Query<SearchQuery>, State(state): State<AppState>) -> Html<String> {
    let query = params.q.unwrap_or_default().to_lowercase();
    let zorbs = state.lock().await;
    let filtered: Vec<_> = zorbs.iter().filter(|z| query.is_empty() || z.name.to_lowercase().contains(&query) || z.description.to_lowercase().contains(&query)).collect();
    let mut html = String::from(r#"<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mt-8">"#);
    for z in &filtered {
        html.push_str(&format!(r#"<div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8"><div class="flex justify-between items-start"><div><span class="font-mono text-cyan-400">{}</span><p class="text-zinc-400 mt-2 text-sm">{}</p></div><span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full">v{}</span></div><div class="mt-8 text-xs text-zinc-500 flex gap-6"><span>↓ {}k</span><span>★ 2.4k</span></div></div>"#, z.name, z.description, z.version, z.downloads/1000));
    }
    if filtered.is_empty() {
        html.push_str(r#"<div class="col-span-full text-center py-12 text-zinc-500">No matching zorbs found</div>"#);
    }
    html.push_str("</div>");
    Html(html)
}

async fn publish_handler(State(state): State<AppState>, mut multipart: Multipart) -> Html<String> {
    let mut name = String::new(); let mut version = String::new(); let mut license = String::new(); let mut description = String::new(); let mut repository = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name().unwrap() {
            "name" => name = field.text().await.unwrap(),
            "version" => version = field.text().await.unwrap(),
            "license" => license = field.text().await.unwrap(),
            "description" => description = field.text().await.unwrap(),
            "repository" => { let r = field.text().await.unwrap(); if !r.is_empty() { repository = Some(r); } }
            "file" => { let _ = field.bytes().await.unwrap(); }
            _ => {}
        }
    }
    let new_zorb = Zorb { name: name.clone(), version, description, license, repository, downloads: 0, created_at: "2026-02-19".to_string() };
    state.lock().await.push(new_zorb.clone());
    Html(format!(r#"<div class="bg-emerald-500/10 border border-emerald-400 text-emerald-400 rounded-3xl p-10 text-center"><div class="text-6xl mb-4">🎉</div><h2 class="text-3xl font-bold">Published successfully!</h2><p class="mt-4 text-xl">{} v{} is now live on zorbs.io</p><a href="/" class="mt-8 inline-block px-10 py-4 bg-white text-black font-bold rounded-3xl hover:bg-cyan-400 transition">← Back to Discover</a></div>"#, name, new_zorb.version))
}
