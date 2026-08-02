use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt; // for `oneshot`

use qt_api::{build_router, AppState, NameFilterServices};
use qt_core::{Dictionaries, DictionaryDefaults, Engine};

fn test_state() -> Arc<AppState> {
    // Tiny engine: enough for routing/handler tests.
    let d = Dictionaries::build(
        "他=tha\n很=ngận\n好=hảo\n红=hồng\n中=trung\n人=nhân\n甲=giáp\n乙=ất\n丙=bính\n丁=đinh\n格=cách\n尔=nhĩ\n斯=tư\n泰=thái\n特=đặc\n在=tại\n身=thân\n后=hậu\n张=trương\n先=tiên\n生=sinh",
        "丁格尔斯泰特=Dingelstedt\n中人=trung nhân",
        "",
        "很好=rất tốt/rất ổn\n红中人=cả cụm\n甲乙=A\n乙丙丁=B",
    );
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(d)),
        dictionary_defaults: Arc::new(DictionaryDefaults {
            names: "丁格尔斯泰特=Dingelstedt\n中人=trung nhân".to_string(),
            names2: "红中人=Hồng Trung Nhân".to_string(),
            pronouns: "他=hắn".to_string(),
            ..DictionaryDefaults::default()
        }),
        name_filter_services: NameFilterServices::default(),
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
async fn dictionary_defaults_returns_raw_customizable_files_only() {
    let resp = build_router(test_state())
        .oneshot(
            Request::builder()
                .uri("/dictionaries/defaults")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("cache-control").unwrap(),
        "public, max-age=3600"
    );
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(
        body,
        serde_json::json!({
            "names": "丁格尔斯泰特=Dingelstedt\n中人=trung nhân",
            "names2": "红中人=Hồng Trung Nhân",
            "luatNhan": "",
            "pronouns": "他=hắn",
            "danhTu": "",
            "hoNguoi": "",
            "hauTu": "",
            "ignoredChinesePhrases": ""
        })
    );
    assert!(body.get("vietphrase").is_none());
    assert!(body.get("chinesePhienAmWords").is_none());
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
async fn translate_applies_request_names_without_leaking_between_requests() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "很好",
            "mode": "vietphrase-one",
            "dictionaries": {
                "names": "很好=custom name/alternative"
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" custom name"}"#);

    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "很好", "mode": "vietphrase-one" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" rất tốt"}"#);
}

#[tokio::test]
async fn translate_applies_request_luat_nhan_and_pronouns() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "在他身后",
            "mode": "vietphrase-one",
            "dictionaries": {
                "luatNhan": "在{n}身后=sau lưng {n}",
                "pronouns": "他=hắn"
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" sau lưng hắn"}"#);
}

#[tokio::test]
async fn translate_applies_request_surname_and_suffix_dictionaries() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "张先生",
            "mode": "vietphrase-one",
            "dictionaries": {
                "hoNguoi": "张=Trương",
                "hauTu": "先生=tiên sinh"
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":" Trương tiên sinh"}"#
    );
}

#[tokio::test]
async fn translate_rejects_invalid_request_luat_nhan() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他",
            "mode": "vietphrase-one",
            "dictionaries": {
                "luatNhan": "([{n}=invalid"
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("invalid LuatNhan rule"));
}

#[tokio::test]
async fn translate_rejects_attempts_to_replace_fixed_dictionaries() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他",
            "mode": "vietphrase-one",
            "dictionaries": {
                "vietPhrase": "他=override"
            }
        }),
    )
    .await;
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn translate_layers_compact_patches_over_fixed_dictionaries() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他很好",
            "mode": "vietphrase-one",
            "dictionaryPatches": {
                "vietPhrase": {
                    "很好": "ổn lắm/rất ổn"
                },
                "chinesePhienAmWords": {
                    "他": "hắn"
                }
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" hắn ổn lắm"}"#);

    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "他很好", "mode": "vietphrase-one" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":" tha rất tốt"}"#);
}

#[tokio::test]
async fn translate_rejects_multi_character_phien_am_patch_keys() {
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({
            "text": "他很好",
            "mode": "vietphrase-one",
            "dictionaryPatches": {
                "chinesePhienAmWords": {
                    "很好": "rất tốt"
                }
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_string(resp).await,
        r#"{"error":"dictionaryPatches.chinesePhienAmWords keys must contain exactly one character"}"#
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
async fn batch_applies_dictionary_patches_to_every_text() {
    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({
            "texts": ["很好", "他"],
            "mode": "vietphrase-one",
            "dictionaryPatches": {
                "vietPhrase": { "很好": "ổn lắm" },
                "chinesePhienAmWords": { "他": "hắn" }
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":[" ổn lắm"," hắn"]}"#
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

#[tokio::test]
async fn filters_names_with_book_memory_and_utf16_ranges() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "😀张先生走来。",
            "mode": "hybrid",
            "knownNames": { "张先生": "Trương Tiên Sinh" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    let candidate = body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["text"] == "张先生")
        .expect("known name candidate");
    assert_eq!(candidate["suggested"], "Trương Tiên Sinh");
    assert_eq!(candidate["known"], true);
    assert_eq!(candidate["ranges"][0]["start"], 2);
    assert_eq!(candidate["ranges"][0]["length"], 3);
    assert_eq!(body["stats"]["scannedCharacters"], 7);
    assert_eq!(body["capabilities"]["aiConfigured"], false);
}

#[tokio::test]
async fn name_filter_applies_ignored_phrases_before_scanning() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "本章张先生完。😀张先生走来。",
            "mode": "hybrid",
            "knownNames": { "张先生": "Trương Tiên Sinh" },
            "dictionaries": {
                "ignoredChinesePhrases": "本章张先生完"
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    let candidate = body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["text"] == "张先生")
        .expect("name outside ignored text");
    assert_eq!(candidate["occurrences"], 1);
    assert_eq!(candidate["ranges"][0]["start"], 9);
    assert_eq!(candidate["ranges"][0]["length"], 3);
    assert_eq!(candidate["contexts"].as_array().unwrap().len(), 1);
    assert!(candidate["contexts"][0]
        .as_str()
        .unwrap()
        .contains("😀【张先生】走来"));
    assert_eq!(body["stats"]["scannedCharacters"], 14);
}

#[tokio::test]
async fn name_filter_suppresses_book_rejections() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。张先生走来。",
            "rejectedNames": ["张先生"]
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert!(body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|candidate| candidate["text"] != "张先生"));
}

#[tokio::test]
async fn name_filter_reports_unconfigured_optional_providers_without_failing() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ner": { "enabled": true },
            "aiExtract": { "enabled": true },
            "aiFallback": { "enabled": true }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(body["warnings"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn name_filter_warns_when_ai_enabled_without_credentials() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiExtract": { "enabled": true },
            "aiFallback": { "enabled": true }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    let warnings = body["warnings"].as_array().unwrap();
    assert_eq!(
        warnings
            .iter()
            .filter(|warning| warning.as_str().unwrap().contains("no AI credentials"))
            .count(),
        2
    );
    assert_eq!(body["capabilities"]["aiConfigured"], false);
}

#[tokio::test]
async fn name_filter_rejects_malformed_ai_credentials_before_any_work() {
    // Unknown provider.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "openai", "apiKey": "sk-test" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("ai.provider"));

    // Empty key.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "deepseek", "apiKey": "  " }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("ai.apiKey"));

    // Gemini without a model.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "gemini", "apiKey": "AIza-test" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("ai.model is required"));

    // Model with a path separator (would inject into the Gemini URL).
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "gemini", "apiKey": "AIza-test", "model": "models/../evil" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("ai.model must contain"));

    // Base URLs are never accepted from the request (SSRF guard).
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "deepseek", "apiKey": "sk-test", "baseUrl": "http://169.254.169.254" }
        }),
    )
    .await;
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn name_filter_validates_ai_options_even_when_disabled() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiExtract": { "enabled": false, "minConfidence": 5.0 }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiExtract.minConfidence must be between 0 and 1"));

    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiFallback": { "enabled": false, "maxCandidates": 500 }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiFallback.maxCandidates must be between 1 and 50"));
}

#[tokio::test]
async fn name_filter_rejects_unknown_mode() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({ "text": "张先生", "mode": "magic" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("invalid name filter mode"));
}

#[tokio::test]
async fn name_filter_rejects_out_of_range_options() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({ "text": "张先生", "maxCandidates": 0 }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("maxCandidates must be between 1 and 1000"));
}
