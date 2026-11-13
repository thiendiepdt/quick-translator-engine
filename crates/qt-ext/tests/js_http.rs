mod common;

use common::{Route, TestServer};
use qt_ext::testing;
use serde_json::json;

#[test]
fn http_get_post_ma_hoa_va_chan_host() {
    let (gbk, _, _) = encoding_rs::GBK.encode("<div>第一章 天下大势</div>");
    let s = TestServer::start(vec![
        Route::bytes("/gbk", "text/html; charset=gbk", gbk.into_owned()),
        Route::bytes("/nolabel", "text/html", encoding_rs::GBK.encode("<meta charset=gbk>天下").0.into_owned()),
        Route::json("/json", r#"{"list":[1,2,3],"name":"书"}"#),
        Route::echo("/echo"),
        Route::sequence("/status", vec![(404, vec![], "không có".into())]),
    ]);
    let src = r#"
        export async function run(ctx) {
            const a = await http.get(ctx.base + "/gbk");
            const b = await http.get(ctx.base + "/json");
            const c = await http.post(ctx.base + "/echo", { a: 1 }, { headers: { "X-Key": "v" }, query: { page: 2 } });
            const d = await http.request({ url: ctx.base + "/nolabel" });
            const e = await http.get(ctx.base + "/status");
            let denied = null;
            try { await http.get("https://evil.test/x"); } catch (err) { denied = err.message; }
            const raw = await http.get(ctx.base + "/gbk", { encoding: "latin1" });
            return {
                a: a.text(), aType: a.headers["content-type"], b: b.json(), echo: c.text(),
                d: d.text(), notFound: [e.status, e.ok, e.text()], denied,
                forced: raw.text() !== a.text(), bytes: a.bytes().length, url: a.url.endsWith("/gbk")
            };
        }"#;
    let pkg = common::package(&[("main.js", src)]);
    let v = testing::call(&pkg, "run", vec![json!({ "base": s.url("") })], Default::default()).unwrap();
    assert_eq!(v["a"], "<div>第一章 天下大势</div>");
    assert_eq!(v["aType"], "text/html; charset=gbk");
    assert_eq!(v["b"], json!({"list": [1, 2, 3], "name": "书"}));
    let echo = v["echo"].as_str().unwrap().to_lowercase();
    assert!(echo.starts_with("post /echo?page=2"), "{echo}");
    assert!(echo.contains("content-type: application/json") && echo.contains("x-key: v") && echo.ends_with("{\"a\":1}"), "{echo}");
    assert_eq!(v["d"], "<meta charset=gbk>天下");
    assert_eq!(v["notFound"], json!([404, false, "không có"]));
    assert!(v["denied"].as_str().unwrap().contains("evil.test"));
    assert_eq!(v["forced"], true);
    assert_eq!(v["url"], true);
    assert!(v["bytes"].as_u64().unwrap() > 10);
}

#[test]
fn body_chuoi_va_bytes() {
    let s = TestServer::start(vec![Route::echo("/echo")]);
    let src = r#"
        export async function run(ctx) {
            const t = await http.post(ctx.base + "/echo", "a=1&b=2", { headers: [["Content-Type", "application/x-www-form-urlencoded"]] });
            const b = await http.post(ctx.base + "/echo", (await http.post(ctx.base + "/echo", "xyz")).bytes());
            return [t.text(), b.text()];
        }"#;
    let pkg = common::package(&[("main.js", src)]);
    let v = testing::call(&pkg, "run", vec![json!({ "base": s.url("") })], Default::default()).unwrap();
    let t = v[0].as_str().unwrap();
    assert!(t.ends_with("a=1&b=2") && t.to_lowercase().contains("application/x-www-form-urlencoded"));
    assert!(v[1].as_str().unwrap().contains("POST /echo HTTP/1.1"), "bytes của phản hồi trước được gửi nguyên");
}
