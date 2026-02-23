use leptos::prelude::*;
use leptos::server_fn::error::NoCustomError;

#[server(Login, "/api")]
pub async fn login(password: String) -> Result<bool, ServerFnError> {
    use tower_sessions::Session;

    let expected =
        std::env::var("AUTH_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let matched = password == expected;

    if matched {
        let session = leptos_axum::extract::<Session>()
            .await
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
        session
            .insert("authenticated", true)
            .await
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    }

    Ok(matched)
}

#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    use tower_sessions::Session;

    let session = leptos_axum::extract::<Session>()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    session
        .flush()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    Ok(())
}
