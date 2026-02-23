#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use ai_meal_planning::*;
    use axum_server::tls_rustls::RustlsConfig;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::services::ServeDir;
    use tower_sessions::{MemoryStore, SessionManagerLayer, cookie::SameSite};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,ai_meal_planning=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let pool = db::init_db().await.expect("Failed to initialize database");

    tracing::info!("Database initialized successfully");

    // Setting get_configuration(Some("Cargo.toml")) means we'll be reading from Cargo.toml
    let conf = leptos::prelude::get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Build application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    // Set up session layer
    let cert_file = std::env::var("CERT_FILE").ok();
    let is_https = cert_file.is_some();
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(is_https)
        .with_same_site(SameSite::Strict);

    // Build our application with routes
    let app = axum::Router::<AppState>::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let pool = pool.clone();
                move || {
                    provide_context(pool.clone());
                }
            },
            {
                let opts = leptos_options.clone();
                move || view! { <Shell options=opts.clone()/> }
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(
            |opts| view! { <Shell options=opts/> },
        ))
        .layer(axum::Extension(pool))
        .layer(axum::middleware::from_fn(auth::require_auth))
        .layer(session_layer)
        .nest_service("/pkg", ServeDir::new("./target/site/pkg"))
        .nest_service("/style", ServeDir::new("./style"))
        .with_state(app_state);

    let key_file = std::env::var("KEY_FILE").ok();
    match (cert_file, key_file) {
        (Some(cert), Some(key)) => {
            tracing::info!("Starting HTTPS server on https://{}", &addr);
            let config = RustlsConfig::from_pem_file(cert, key)
                .await
                .expect("Failed to load TLS certificate/key");
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        _ => {
            tracing::info!("Listening on http://{}", &addr);
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        }
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // No-op for WASM
}
