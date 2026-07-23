use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use qt_api::{AppState, NameFilterServices};
use qt_core::{Dictionaries, DictionaryDefaults, Engine};
use tower::ServiceExt;

fn test_state() -> Arc<AppState> {
    let dictionaries = Dictionaries::build(
        "来=lai\n人=nhân\n名=danh\n为=vi\n萧=tiêu\n炎=viêm\n走=tẩu",
        "",
        "",
        "",
    );
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(dictionaries)),
        dictionary_defaults: Arc::new(DictionaryDefaults::default()),
        name_filter_services: NameFilterServices::default(),
    })
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn exposes_name_filter_surface_only() {
    let capabilities = qt_ner_api::build_router(test_state())
        .oneshot(
            Request::builder()
                .uri("/capabilities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(capabilities.status(), StatusCode::OK);
    assert_eq!(
        body_json(capabilities).await,
        serde_json::json!({
            "nerConfigured": false,
            "aiConfigured": false
        })
    );

    let translation = qt_ner_api::build_router(test_state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/translate")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"很好","mode":"vietphrase-one"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(translation.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn filters_names_with_the_shared_contract() {
    let response = qt_ner_api::build_router(test_state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/names/filter")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"text":"来人名为萧炎。萧炎走来。","mode":"hybrid","minOccurrences":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert!(body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|candidate| candidate["text"] == "萧炎"));
}
