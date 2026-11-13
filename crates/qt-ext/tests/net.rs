mod common;

use common::{Route, TestServer};
use qt_ext::{ExtError, HostPolicy, HttpClient, HttpRequest, Limits};

fn client(allow: &[&str], limits: Limits) -> HttpClient {
    let hosts: Vec<String> = allow.iter().map(|s| s.to_string()).collect();
    HttpClient::new(HostPolicy::new(&hosts, true), limits).unwrap()
}

fn get(url: String) -> HttpRequest {
    HttpRequest { method: "GET".into(), url, ..Default::default() }
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
}

#[test]
fn redirect_kiem_host_tung_buoc() {
    let s = TestServer::start(vec![Route::redirect("/a", "/b"), Route::ok("/b", "xong"), Route::redirect("/out", "http://evil.test/x"), Route::redirect("/loop", "/loop")]);
    let c = client(&["127.0.0.1"], Limits::default());
    let r = rt().block_on(c.send(get(s.url("/a")))).unwrap();
    assert_eq!((r.status, r.body.as_slice()), (200, b"xong".as_slice()));
    assert!(r.url.ends_with("/b"));
    let e = rt().block_on(c.send(get(s.url("/out")))).unwrap_err();
    assert_eq!(e, ExtError::HostDenied("evil.test".into()));
    assert!(matches!(rt().block_on(c.send(get(s.url("/loop")))), Err(ExtError::Network(m)) if m.contains("chuyển hướng")));
}

#[test]
fn chan_host_ngoai_danh_sach_va_qua_kich_thuoc() {
    let s = TestServer::start(vec![Route::ok("/big", &"x".repeat(21 * 1024 * 1024))]);
    let c = client(&["127.0.0.1"], Limits::default());
    assert!(matches!(rt().block_on(c.send(get("https://evil.test/".into()))), Err(ExtError::HostDenied(_))));
    assert!(matches!(rt().block_on(c.send(get(s.url("/big")))), Err(ExtError::TooLarge(_))));
}

#[test]
fn cho_429_theo_retry_after() {
    let s = TestServer::start(vec![Route::sequence("/r", vec![(429, vec![("Retry-After".into(), "1".into())], "".into()), (200, vec![], "ok".into())])]);
    let c = client(&["127.0.0.1"], Limits::default());
    let t = std::time::Instant::now();
    let r = rt().block_on(c.send(get(s.url("/r")))).unwrap();
    assert_eq!(r.status, 200);
    assert!(t.elapsed().as_millis() >= 900);
    assert_eq!(s.hits("/r"), 2);
}

#[test]
fn query_header_body_va_charset() {
    let s = TestServer::start(vec![Route::echo("/echo")]);
    let c = client(&["127.0.0.1"], Limits::default());
    let mut req = get(s.url("/echo"));
    req.method = "post".into();
    req.query = vec![("q".into(), "天下".into())];
    req.headers = vec![("X-Test".into(), "1".into())];
    req.body = Some(b"abc".to_vec());
    let r = rt().block_on(c.send(req)).unwrap();
    let echo = String::from_utf8(r.body).unwrap();
    assert!(echo.contains("POST /echo?q=%E5%A4%A9%E4%B8%8B"), "{echo}");
    assert!(echo.to_lowercase().contains("x-test: 1"));
    assert!(echo.ends_with("abc"));
    assert_eq!(r.charset.as_deref(), Some("utf-8"));
}

#[test]
fn gian_cach_va_so_luong_theo_host() {
    let s = TestServer::start(vec![Route::ok("/d", "x").slow(200), Route::ok("/fast", "x")]);
    let c = client(&["127.0.0.1"], Limits { concurrency: 2, delay_ms: 0 });
    let t = std::time::Instant::now();
    rt().block_on(async {
        let calls: Vec<_> = (0..4).map(|_| c.send(get(s.url("/d")))).collect();
        for r in futures_util::future::join_all(calls).await {
            r.unwrap();
        }
    });
    let ms = t.elapsed().as_millis();
    assert!((390..800).contains(&ms), "4 lượt, 2 luồng, mỗi lượt 200 ms: {ms} ms");

    let gap = client(&["127.0.0.1"], Limits { concurrency: 3, delay_ms: 150 });
    let t = std::time::Instant::now();
    rt().block_on(async {
        for _ in 0..3 {
            gap.send(get(s.url("/fast"))).await.unwrap();
        }
    });
    let ms = t.elapsed().as_millis();
    assert!(ms >= 290, "3 lượt cách nhau 150 ms tính từ lúc bắt đầu lượt trước: {ms} ms");
}

#[test]
fn loi_khong_lo_query() {
    let c = client(&["127.0.0.1"], Limits::default());
    // cổng 1 không có gì nghe
    let e = rt().block_on(c.send(get("http://127.0.0.1:1/x?key=sk-SECRET".into()))).unwrap_err().to_string();
    assert!(!e.contains("sk-SECRET"), "{e}");
}
