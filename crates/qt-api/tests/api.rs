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
        "丁格尔斯泰特=Dingelstedt",
        "",
        "很好=rất tốt/rất ổn",
    );
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(d)),
    })
}

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn post_json(
    app: axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn health_ok() {
    let app = build_router(test_state());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"status":"ok"}"#);
}

#[tokio::test]
async fn translate_faithful_and_pretty() {
    // faithful: leading space, lowercase first word
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "他很好", "mode": "vietphrase" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":" tha rất tốt/rất ổn"}"#
    );

    // pretty: trimmed + first letter capitalized
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "他很好", "mode": "vietphrase", "pretty": true }),
    )
    .await;
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":"Tha rất tốt/rất ổn"}"#
    );
}

#[tokio::test]
async fn translate_invalid_mode_is_400() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "他", "mode": "nope" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_string(resp).await, r#"{"error":"invalid mode: nope"}"#);
}

#[tokio::test]
async fn translate_wraps_and_selects_one_meaning() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "很好",
            "mode": "vietphrase-one",
            "wrap": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" [rất tốt]"}"#);
}

#[tokio::test]
async fn translate_uses_qt_scan_range_for_long_phrases() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "丁格尔斯泰特",
            "mode": "vietphrase-one"
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" Dingelstedt"}"#);
}

#[tokio::test]
async fn batch_preserves_order() {
    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({ "texts": ["他很好", "他"], "mode": "vietphrase" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":[" tha rất tốt/rất ổn"," tha"]}"#
    );
}

#[tokio::test]
async fn batch_invalid_mode_is_400() {
    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({ "texts": ["他"], "mode": "nope" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_string(resp).await, r#"{"error":"invalid mode: nope"}"#);
}

#[tokio::test]
async fn modes_lists_supported() {
    let app = build_router(test_state());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/modes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"modes":["hanviet","vietphrase","vietphrase-one"]}"#
    );
}
