use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

pub async fn require_auth(session: Session, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();

    let is_public = path == "/login"
        || path.starts_with("/api/login")
        || path.starts_with("/api/logout")
        || path.starts_with("/pkg")
        || path.starts_with("/style");

    if is_public {
        return next.run(request).await;
    }

    let authenticated = session
        .get::<bool>("authenticated")
        .await
        .unwrap_or(None);

    if authenticated == Some(true) {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}
