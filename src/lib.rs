#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
pub mod api;
pub mod app;
pub mod components;
#[cfg(feature = "ssr")]
pub mod db;
pub mod models;
pub mod pages;
#[cfg(feature = "ssr")]
pub mod reports;
pub mod server_functions;

pub use app::*;

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
#[cfg(feature = "ssr")]
use sqlx::SqlitePool;

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub leptos_options: leptos::prelude::LeptosOptions,
    pub pool: SqlitePool,
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for leptos::prelude::LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[cfg(feature = "ssr")]
impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    _ = console_log::init();

    leptos::mount::hydrate_body(App);
}
