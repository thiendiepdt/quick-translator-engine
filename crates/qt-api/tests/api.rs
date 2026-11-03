use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt; // for `oneshot`

use qt_api::{
    build_lite_router, build_lite_router_with_body_limit, build_router,
    build_router_with_body_limit, AppState, LiteAppState, DEFAULT_MAX_BODY_BYTES,
};
use qt_core::{Dictionaries, DictionaryDefaults, Engine, LiteEngine};

const TEST_HAN_VIET: &str = "他=tha\n很=ngận\n好=hảo\n红=hồng\n中=trung\n人=nhân\n甲=giáp\n乙=ất\n丙=bính\n丁=đinh\n格=cách\n尔=nhĩ\n斯=tư\n泰=thái\n特=đặc\n在=tại\n身=thân\n后=hậu\n张=trương\n先=tiên\n生=sinh";
const TEST_NAMES: &str = "丁格尔斯泰特=Dingelstedt\n中人=trung nhân";
const TEST_VIETPHRASE: &str = "很好=rất tốt/rất ổn\n红中人=cả cụm\n甲乙=A\n乙丙丁=B";

fn test_state() -> Arc<AppState> {
    // Tiny engine: enough for routing/handler tests.
    let d = Dictionaries::build(TEST_HAN_VIET, TEST_NAMES, "", TEST_VIETPHRASE)
        .with_lac_viet("金=✚[jīn] Hán Việt: KIM\\n\\t1. vàng\n美=✚[měi] Hán Việt: MĨ\\n\\t1. đẹp");
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(d)),
        dictionary_defaults: Arc::new(DictionaryDefaults {
            names: "丁格尔斯泰特=Dingelstedt\n中人=trung nhân".to_string(),
            names2: "红中人=Hồng Trung Nhân".to_string(),
            pronouns: "他=hắn".to_string(),
            ..DictionaryDefaults::default()
        }),
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
async fn meanings_returns_lac_viet_definitions_for_each_selected_character() {
    let resp = post_json(
        build_router(test_state()),
        "/meanings",
        serde_json::json!({ "text": "金美" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body_string(resp).await).unwrap(),
        serde_json::json!({
            "entries": [
                {
                    "source": "金",
                    "definition": "✚[jīn] Hán Việt: KIM\n\t1. vàng"
                },
                {
                    "source": "美",
                    "definition": "✚[měi] Hán Việt: MĨ\n\t1. đẹp"
                }
            ]
        })
    );
}

#[tokio::test]
async fn translate_faithful_and_pretty() {
    // faithful: leading space, lowercase first word
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "他很好", "mode": "vietphrase", "pretty": false }),
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
async fn translate_defaults_to_the_primary_mode_with_a_minimal_response() {
    // A bare {"text": ...} request works: mode falls back to vietphrase-one
    // and the response carries only `translated`.
    let resp = post_json(
        build_router(test_state()),
        "/translate",
        serde_json::json!({ "text": "很好" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":"Rất tốt"}"#);

    let resp = post_json(
        build_router(test_state()),
        "/translate/batch",
        serde_json::json!({ "texts": ["很好"] }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"translated":["Rất tốt"]}"#);
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
            "wrap": true,
            "pretty": false
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
            "mode": "vietphrase-one",
            "pretty": false
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
            "scanRange": 5,
            "pretty": false
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
            "pretty": false,
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
        serde_json::json!({ "text": "很好", "mode": "vietphrase-one", "pretty": false }),
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
            "pretty": false,
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
            "pretty": false,
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
            "pretty": false,
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
        serde_json::json!({ "text": "他很好", "mode": "vietphrase-one", "pretty": false }),
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
        serde_json::json!({ "texts": ["他很好", "他"], "mode": "vietphrase", "pretty": false }),
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
            "scanRange": 5,
            "pretty": false
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
            "pretty": false,
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
            "ranges": true,
            "pretty": false
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
            "ner": { "enabled": true }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    let warnings = body["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].as_str().unwrap().contains("aiEntities"));
}

#[tokio::test]
async fn name_filter_merges_caller_supplied_ai_entities() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "😀张先生走来。",
            "aiEntities": {
                "entities": [
                    // New candidate the rules cannot see (not in any dictionary).
                    { "text": "张先生", "entityType": "person", "suggested": "Trương Tiên Sinh", "confidence": 0.9 },
                    // Below the merge threshold: ignored.
                    { "text": "走来", "confidence": 0.2 }
                ]
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(body["stats"]["aiMergedCandidates"], 1);
    let candidate = body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["text"] == "张先生")
        .expect("AI entity merged as candidate");
    assert_eq!(candidate["suggested"], "Trương Tiên Sinh");
    assert_eq!(candidate["entityType"], "person");
    // Entities carry no spans; occurrences are located in the scan document
    // and mapped back to the caller's UTF-16 input (emoji shifts by 2).
    assert_eq!(candidate["ranges"][0]["start"], 2);
    assert_eq!(candidate["ranges"][0]["length"], 3);
    assert!(candidate["sources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|source| source == "ai-fallback"));
    assert!(body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|candidate| candidate["text"] != "走来"));
}

#[tokio::test]
async fn name_filter_honors_ai_entities_merge_threshold() {
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": {
                "minConfidence": 0.1,
                "entities": [
                    { "text": "张先生", "confidence": 0.2 }
                ]
            }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&body_string(resp).await).unwrap();
    assert_eq!(body["stats"]["aiMergedCandidates"], 1);
}

#[tokio::test]
async fn name_filter_rejects_invalid_ai_entities_before_any_work() {
    // Legacy server-side AI fields are gone: requests carrying them fail.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "ai": { "provider": "deepseek", "apiKey": "sk-test" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Unknown fields inside aiEntities (e.g. a smuggled baseUrl) fail too.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": { "entities": [], "baseUrl": "http://169.254.169.254" }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Out-of-range merge threshold.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": { "entities": [], "minConfidence": 5.0 }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiEntities.minConfidence must be between 0 and 1"));

    // Empty entity text.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": { "entities": [{ "text": "  " }] }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiEntities.entities[].text"));

    // Out-of-range entity confidence.
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": { "entities": [{ "text": "张先生", "confidence": 7.5 }] }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiEntities.entities[].confidence"));

    // Too many entities.
    let entities: Vec<serde_json::Value> = (0..501)
        .map(|index| serde_json::json!({ "text": format!("名字{index}") }))
        .collect();
    let resp = post_json(
        build_router(test_state()),
        "/names/filter",
        serde_json::json!({
            "text": "张先生走来。",
            "aiEntities": { "entities": entities }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("aiEntities.entities must not exceed 500"));
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

// ---- qt-lite-server ----

fn lite_state() -> Arc<LiteAppState> {
    // Cùng dữ liệu với `test_state` để so output hai server.
    let engine = LiteEngine::build(
        TEST_HAN_VIET,
        TEST_NAMES,
        "",
        TEST_VIETPHRASE,
        "",
        "",
        "",
        "",
        "",
    )
    .unwrap();
    Arc::new(LiteAppState {
        engine: Arc::new(engine),
    })
}

async fn get_path(app: axum::Router, uri: &str) -> axum::response::Response {
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn lite_translate_matches_full_server_one_meaning() {
    let requests = [
        serde_json::json!({ "text": "他很好" }),
        serde_json::json!({ "text": "丁格尔斯泰特很好，红中人", "pretty": false, "ranges": false }),
        serde_json::json!({ "text": "甲乙丙丁", "mode": "vietphrase-one", "wrap": true }),
        serde_json::json!({
            "text": "张先生在他身后很好",
            "dictionaries": {
                "names": "张先生=Trương tiên sinh/ông Trương",
                "luatNhan": "在{n}身后=sau lưng {n}",
                "pronouns": "他=hắn"
            },
            "dictionaryPatches": {
                "vietPhrase": { "很好": "ổn lắm/rất ổn" },
                "chinesePhienAmWords": { "好": "HẢO" }
            }
        }),
    ];
    for request in requests {
        let full = post_json(build_router(test_state()), "/translate", request.clone()).await;
        let lite = post_json(
            build_lite_router(lite_state()),
            "/translate",
            request.clone(),
        )
        .await;
        assert_eq!(full.status(), StatusCode::OK);
        assert_eq!(lite.status(), StatusCode::OK);
        assert_eq!(
            body_string(lite).await,
            body_string(full).await,
            "{request}"
        );
    }
}

#[tokio::test]
async fn lite_rejects_ranges_requests() {
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate",
        serde_json::json!({ "text": "他很好", "ranges": true }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp)
        .await
        .contains("ranges is not supported by this server"));
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate/batch",
        serde_json::json!({ "texts": ["他很好"], "ranges": true }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// Không xin range thì server đi đường không dựng range: text phải trùng
/// với khi xin range, kể cả có override và wrap.
#[tokio::test]
async fn translate_text_is_identical_with_and_without_ranges() {
    for mut request in [
        serde_json::json!({ "text": "丁格尔斯泰特很好，红中人甲乙丙丁", "pretty": false }),
        serde_json::json!({ "text": "他很好。", "mode": "vietphrase", "wrap": true }),
        serde_json::json!({ "text": "他很好", "mode": "hanviet" }),
        serde_json::json!({
            "text": "张先生在他身后很好",
            "dictionaries": { "names": "张先生=Trương tiên sinh", "pronouns": "他=hắn" },
            "dictionaryPatches": { "vietPhrase": { "很好": "ổn lắm" } }
        }),
    ] {
        let without = post_json(build_router(test_state()), "/translate", request.clone()).await;
        assert_eq!(without.status(), StatusCode::OK);
        let without: serde_json::Value = serde_json::from_str(&body_string(without).await).unwrap();
        request["ranges"] = serde_json::json!(true);
        let with = post_json(build_router(test_state()), "/translate", request.clone()).await;
        assert_eq!(with.status(), StatusCode::OK);
        let with: serde_json::Value = serde_json::from_str(&body_string(with).await).unwrap();
        assert_eq!(with["translated"], without["translated"], "{request}");
        assert!(with["sourceRanges"].is_array() && without.get("sourceRanges").is_none());
    }
}

#[tokio::test]
async fn lite_batch_applies_names_and_patches_to_every_text() {
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate/batch",
        serde_json::json!({
            "texts": ["张先生很好", "很好"],
            "pretty": false,
            "dictionaries": { "names": "张先生=Trương tiên sinh" },
            "dictionaryPatches": { "vietPhrase": { "很好": "ổn lắm/rất ổn" } }
        }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_string(resp).await,
        r#"{"translated":[" Trương tiên sinh ổn lắm"," ổn lắm"]}"#
    );
}

#[tokio::test]
async fn lite_rejects_modes_other_than_vietphrase_one() {
    for mode in ["vietphrase", "hanviet"] {
        let resp = post_json(
            build_lite_router(lite_state()),
            "/translate",
            serde_json::json!({ "text": "他", "mode": mode }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(body_string(resp)
            .await
            .contains(&format!("mode {mode} is not supported by this server")));
    }
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate/batch",
        serde_json::json!({ "texts": ["他"], "mode": "hanviet" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate",
        serde_json::json!({ "text": "他", "mode": "bogus" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("invalid mode: bogus"));
}

#[tokio::test]
async fn lite_lists_one_mode_and_has_no_heavy_routes() {
    let resp = get_path(build_lite_router(lite_state()), "/modes").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, r#"{"modes":["vietphrase-one"]}"#);
    let resp = get_path(build_lite_router(lite_state()), "/health").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = get_path(build_lite_router(lite_state()), "/dictionaries/defaults").await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    for uri in ["/names/filter", "/meanings"] {
        let resp = post_json(
            build_lite_router(lite_state()),
            uri,
            serde_json::json!({ "text": "张先生" }),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{uri}");
    }
}

#[tokio::test]
async fn lite_rejects_invalid_request_luat_nhan() {
    let resp = post_json(
        build_lite_router(lite_state()),
        "/translate",
        serde_json::json!({ "text": "他", "dictionaries": { "luatNhan": "([{n}=invalid" } }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(body_string(resp).await.contains("invalid LuatNhan rule"));
}

#[tokio::test]
async fn body_limit_is_configurable_per_router() {
    assert_eq!(DEFAULT_MAX_BODY_BYTES, 32 * 1024 * 1024);
    let text = "很好".repeat(400); // ~2,4 KB JSON
    let body = serde_json::json!({ "text": text });
    let resp = post_json(
        build_router_with_body_limit(test_state(), 1024),
        "/translate",
        body.clone(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let resp = post_json(
        build_lite_router_with_body_limit(lite_state(), 1024),
        "/translate",
        body.clone(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let resp = post_json(build_router(test_state()), "/translate", body.clone()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = post_json(build_lite_router(lite_state()), "/translate", body).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
