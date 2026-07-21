use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt; // for `oneshot`

use qt_api::{build_router, AppState};
use qt_core::{Dictionaries, Engine};

fn test_state() -> Arc<AppState> {
    // Tiny engine: enough for routing/handler tests.
    let d = Dictionaries::build(
        "他=tha\n很=ngận\n好=hảo",
        "",
        "",
        "很好=rất tốt/rất ổn",
    );
    Arc::new(AppState { engine: Arc::new(Engine::from_dicts(d)) })
}

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn health_ok() {
    let app = build_router(test_state());
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"status":"ok"}"#);
}
