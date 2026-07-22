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
        "他=tha\n很=ngận\n好=hảo\n红=hồng\n中=trung\n人=nhân\n甲=giáp\n乙=ất\n丙=bính\n丁=đinh\n格=cách\n尔=nhĩ\n斯=tư\n泰=thái\n特=đặc",
        "丁格尔斯泰特=Dingelstedt\n中人=trung nhân",
        "",
        "很好=rất tốt/rất ổn\n红中人=cả cụm\n甲乙=A\n乙丙丁=B",
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
async fn translate_standardizes_chinese_punctuation() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他、他。他",
            "mode": "vietphrase-one",
            "pretty": true,
            "ranges": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(body["translated"], "Tha, tha. Tha");
    assert_eq!(
        body["sourceRanges"][1],
        serde_json::json!({ "start": 1, "length": 1 })
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
async fn translate_exposes_engine_options() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "丁格尔斯泰特",
            "mode": "vietphrase-one",
            "scanRange": 5
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":" đinh cách nhĩ tư thái đặc"}"#
    );

    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "甲乙丙丁",
            "mode": "vietphrase-one",
            "translationAlgorithm": 0,
            "pretty": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":"Giáp B"}"#);

    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "红中人",
            "mode": "vietphrase-one",
            "prioritizedName": false,
            "pretty": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":"Cả cụm"}"#);
}

#[tokio::test]
async fn translate_rejects_invalid_engine_options() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他",
            "mode": "vietphrase",
            "scanRange": 0
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_string(resp).await,
        r#"{"error":"scanRange must be between 1 and 100"}"#
    );

    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他",
            "mode": "vietphrase",
            "translationAlgorithm": 3
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_string(resp).await,
        r#"{"error":"translationAlgorithm must be 0, 1, or 2"}"#
    );
}

#[tokio::test]
async fn non_bmp_fallback_uses_one_utf16_span_per_rust_scalar() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "😀",
            "mode": "vietphrase-one",
            "ranges": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":"😀","sourceRanges":[{"start":0,"length":2}],"targetRanges":[{"start":0,"length":2}]}"#
    );
}

#[tokio::test]
async fn translate_converts_chinese_numbers_and_returns_utf16_ranges() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "一百二十三",
            "mode": "vietphrase-one",
            "pretty": true,
            "ranges": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":"123","sourceRanges":[{"start":0,"length":5}],"targetRanges":[{"start":0,"length":3}]}"#
    );
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
async fn batch_accepts_engine_options() {
    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({
            "texts": ["丁格尔斯泰特"],
            "mode": "vietphrase-one",
            "scanRange": 5
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":[" đinh cách nhĩ tư thái đặc"]}"#
    );
}

#[tokio::test]
async fn batch_returns_parallel_range_matrices_when_requested() {
    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({
            "texts": ["一二", "三四万"],
            "mode": "vietphrase",
            "ranges": true
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":[" 1-2"," 3-4 vạn"],"sourceRanges":[[{"start":0,"length":2}],[{"start":0,"length":3}]],"targetRanges":[[{"start":1,"length":3}],[{"start":1,"length":7}]]}"#
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
