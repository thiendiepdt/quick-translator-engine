//! Máy chủ HTTP giả trên 127.0.0.1 và trang nguồn mẫu — dùng chung cho test của qt-ext và qt-ai-core.
//! Không dùng trong app.
#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// (status, header, body) của một lượt phản hồi.
pub type Step = (u16, Vec<(String, String)>, String);

#[derive(Clone)]
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct Route {
    /// Khớp đúng path + query trước, rồi tới path không query.
    pub path: String,
    pub responses: Vec<Resp>,
    pub delay_ms: u64,
    pub echo: bool,
}

impl Route {
    pub fn bytes(path: &str, content_type: &str, body: Vec<u8>) -> Route {
        Route { path: path.into(), responses: vec![Resp { status: 200, headers: vec![("Content-Type".into(), content_type.into())], body }], delay_ms: 0, echo: false }
    }
    pub fn ok(path: &str, body: &str) -> Route {
        Route::bytes(path, "text/html; charset=utf-8", body.as_bytes().to_vec())
    }
    pub fn json(path: &str, body: &str) -> Route {
        Route::bytes(path, "application/json", body.as_bytes().to_vec())
    }
    pub fn redirect(path: &str, to: &str) -> Route {
        Route { path: path.into(), responses: vec![Resp { status: 302, headers: vec![("Location".into(), to.into())], body: vec![] }], delay_ms: 0, echo: false }
    }
    pub fn sequence(path: &str, list: Vec<Step>) -> Route {
        let responses = list.into_iter().map(|(status, headers, body)| Resp { status, headers, body: body.into_bytes() }).collect();
        Route { path: path.into(), responses, delay_ms: 0, echo: false }
    }
    pub fn echo(path: &str) -> Route {
        Route { path: path.into(), responses: vec![], delay_ms: 0, echo: true }
    }
    pub fn slow(mut self, ms: u64) -> Route {
        self.delay_ms = ms;
        self
    }
}

pub struct TestServer {
    port: u16,
    hits: Arc<Mutex<HashMap<String, usize>>>,
}

impl TestServer {
    pub fn start(routes: Vec<Route>) -> TestServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let hits: Arc<Mutex<HashMap<String, usize>>> = Arc::default();
        let routes = Arc::new(routes);
        let counter = hits.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let routes = routes.clone();
                let counter = counter.clone();
                std::thread::spawn(move || {
                    let _ = handle(stream, &routes, &counter);
                });
            }
        });
        TestServer { port, hits }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    pub fn hits(&self, path: &str) -> usize {
        self.hits.lock().unwrap().get(path).copied().unwrap_or(0)
    }
}

fn handle(stream: TcpStream, routes: &[Route], hits: &Mutex<HashMap<String, usize>>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut head = String::new();
    let mut length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
        head.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let target = head.split_whitespace().nth(1).unwrap_or("/").to_string();
    let path = target.split('?').next().unwrap_or("/").to_string();
    let route = routes.iter().find(|r| r.path == target).or_else(|| routes.iter().find(|r| r.path == path));
    let mut stream = stream;
    let Some(route) = route else {
        return write(&mut stream, &Resp { status: 404, headers: vec![], body: b"not found".to_vec() });
    };
    let n = {
        let mut map = hits.lock().unwrap();
        let entry = map.entry(route.path.clone()).or_insert(0);
        *entry += 1;
        *entry
    };
    if route.delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(route.delay_ms));
    }
    if route.echo {
        let mut echoed = head.into_bytes();
        echoed.extend_from_slice(&body);
        return write(&mut stream, &Resp { status: 200, headers: vec![("Content-Type".into(), "text/plain; charset=utf-8".into())], body: echoed });
    }
    let resp = &route.responses[(n - 1).min(route.responses.len() - 1)];
    write(&mut stream, resp)
}

fn write(stream: &mut TcpStream, resp: &Resp) -> std::io::Result<()> {
    let mut out = format!("HTTP/1.1 {} X\r\nContent-Length: {}\r\nConnection: close\r\n", resp.status, resp.body.len());
    let has_type = resp.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
    if !has_type {
        out.push_str("Content-Type: text/html; charset=utf-8\r\n");
    }
    for (key, value) in &resp.headers {
        out.push_str(&format!("{key}: {value}\r\n"));
    }
    out.push_str("\r\n");
    stream.write_all(out.as_bytes())?;
    stream.write_all(&resp.body)?;
    stream.flush()
}

pub fn gbk(text: &str) -> Vec<u8> {
    encoding_rs::GBK.encode(text).0.into_owned()
}

/// Trang nguồn giả: mục lục GBK 2 trang (6 chương: chương 5 khoá, chương 6 link /c/err), chương 3 chậm 400 ms.
pub fn demo_server() -> TestServer {
    let page1 = r#"<div id="list"><a href="/c/1">第1章 起</a><a href="/c/2">第2章 承</a><a href="/c/3">第3章 转</a></div><a class="next" href="/book/1/list?page=2">下一页</a>"#;
    let page2 = r#"<div id="list"><a href="/c/4">第4章 合</a><a class="vip" href="/c/5">第5章 VIP</a><a href="/c/err">第6章 坏</a></div>"#;
    let mut routes = vec![
        Route::bytes("/book/1/list?page=1", "text/html; charset=gbk", gbk(page1)),
        Route::bytes("/book/1/list?page=2", "text/html", gbk(&format!("<meta charset=\"gbk\">{page2}"))),
    ];
    for n in 1..=5 {
        let body = format!("<h1>第{n}章</h1><div id=\"content\"><p>天下大势，分久必合。</p><p>第{n}段</p></div>");
        let route = Route::bytes(&format!("/c/{n}"), "text/html; charset=gbk", gbk(&body));
        routes.push(if n == 3 { route.slow(400) } else { route });
    }
    TestServer::start(routes)
}

/// Thư mục extension mẫu (manifest hosts 127.0.0.1, nạp ở chế độ developer) đi cùng `demo_server`.
pub fn demo_extension_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo")
}
