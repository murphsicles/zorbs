// src/handlers/docs.rs
use axum_login::AuthSession;
use maud::{html, Markup, PreEscaped};
use crate::models::user::UserBackend;
use crate::views;

pub async fn docs_page(auth_session: AuthSession<UserBackend>) -> Markup {
    let user = &auth_session.user;
    let auth_markup = if let Some(user) = user {
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
    let mut html_str = views::DOCS_HTML.to_string();
    let auth_str = auth_markup.into_string();
    if let Some(pos) = html_str.find("<!-- AUTH_SLOT -->") {
        html_str.replace_range(pos..pos + "<!-- AUTH_SLOT -->".len(), &auth_str);
    }
    if let Some(pos) = html_str.find("<!-- AUTH_SLOT_MOBILE -->") {
        html_str.replace_range(pos..pos + "<!-- AUTH_SLOT_MOBILE -->".len(), &auth_str);
    }
    html! { (PreEscaped(html_str)) }
}
