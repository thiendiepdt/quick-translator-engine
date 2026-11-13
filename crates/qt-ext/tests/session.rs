mod common;

use qt_ext::{ChapterFormat, ExtError, Session, TocItem};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn numbered(toc: Vec<TocItem>) -> Vec<(usize, TocItem)> {
    toc.into_iter().enumerate().map(|(i, t)| (i + 1, t)).collect()
}

#[test]
fn toc_nhieu_trang_gbk_va_chapters_song_song() {
    let s = common::demo_server();
    let session = Session::start(common::demo_package(), common::session_opts()).unwrap();
    let ctx = json!({ "rawUrl": s.url("/book/1"), "story": { "name": "Demo", "current": null } });
    let toc = session.toc(&ctx).unwrap();
    assert_eq!(toc.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(), ["第1章 起", "第2章 承", "第3章 转", "第4章 合", "第5章 VIP", "第6章 坏"]);
    assert!(toc[4].locked && !toc[0].locked);
    assert_eq!(toc[0].data, Some(json!({ "href": "/c/1" })));
    let items = numbered(toc);
    let mut got = Vec::new();
    let t = Instant::now();
    session.chapters(&ctx, &items[..4], &mut |o| got.push(o));
    assert!(t.elapsed().as_millis() < 1500, "song song 3 luồng");
    got.sort_by_key(|o| o.index);
    assert_eq!(got.iter().map(|o| o.index).collect::<Vec<_>>(), [1, 2, 3, 4]);
    let c1 = got[0].result.as_ref().unwrap();
    assert_eq!(c1.format, ChapterFormat::Html);
    assert_eq!(c1.title.as_deref(), Some("第1章"));
    assert!(c1.content.contains("天下大势"));
    let mut bad = Vec::new();
    session.chapters(&ctx, &items[5..6], &mut |o| bad.push(o));
    assert_eq!(bad[0].result, Err(ExtError::Extension { message: "hỏng".into(), retry: true }));
    assert_eq!(bad[0].index, 6);
}

#[test]
fn toc_sai_dang_bi_tu_choi() {
    let pkg = common::package(&[("main.js", r#"export async function toc() { return [{ id: "1", title: "a" }, { id: "1", title: "b" }]; }
                                             export async function chapter() { return { content: "" }; }"#)]);
    let session = Session::start(pkg, common::session_opts()).unwrap();
    assert!(matches!(session.toc(&json!({})), Err(ExtError::BadResult(m)) if m.contains("trùng")));
}

#[test]
fn ham_chapters_gop_nhom() {
    let pkg = common::package(&[("main.js", r#"
        globalThis.calls = 0;
        export async function toc() { return Array.from({ length: 45 }, (_, i) => ({ id: String(i + 1), title: "c" + (i + 1) })); }
        export async function chapter() { throw new Error("không được gọi"); }
        export async function chapters(ctx, list) {
            calls++;
            const n = calls;
            await sleep(10);
            return list.filter(c => c.id !== "7").map(c => ({ id: c.id, content: "nội dung " + c.id + " nhóm " + n, error: c.id === "9" ? "hết lượt" : null }));
        }"#)]);
    let session = Session::start(pkg, common::session_opts()).unwrap();
    let items = numbered(session.toc(&json!({})).unwrap());
    let mut got = Vec::new();
    session.chapters(&json!({}), &items, &mut |o| got.push(o));
    assert_eq!(got.len(), 45);
    got.sort_by_key(|o| o.index);
    assert!(matches!(&got[6].result, Err(ExtError::Extension { retry: true, .. })), "mục 7 thiếu trong kết quả");
    assert_eq!(got[8].result, Err(ExtError::Extension { message: "hết lượt".into(), retry: true }));
    let groups: std::collections::BTreeSet<String> = got.iter().filter_map(|o| o.result.as_ref().ok()).map(|c| c.content.rsplit(' ').next().unwrap().to_string()).collect();
    assert_eq!(groups.len(), 3, "45 mục = 3 nhóm 20/20/5");
}

#[test]
fn huy_giua_chung() {
    let pkg = common::package(&[("main.js", r#"
        export async function toc() { return Array.from({ length: 10 }, (_, i) => ({ id: String(i), title: "c" })); }
        export async function chapter(ctx, ch) { await sleep(300); return { content: "x" }; }"#)]);
    let session = Arc::new(Session::start(pkg, common::session_opts()).unwrap());
    let items = numbered(session.toc(&json!({})).unwrap());
    let s2 = session.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(450));
        s2.cancel();
    });
    let mut got = Vec::new();
    let t = Instant::now();
    session.chapters(&json!({}), &items, &mut |o| got.push(o));
    assert!(t.elapsed().as_millis() < 1500);
    assert_eq!(got.len(), 10);
    assert!(got.iter().any(|o| o.result.is_ok()));
    assert!(got.iter().any(|o| o.result == Err(ExtError::Cancelled)));
}

#[test]
fn loi_nap_goi_bao_ngay() {
    let pkg = common::package(&[("main.js", "export async function toc() {} throw new Error('hỏng lúc nạp');")]);
    assert!(matches!(Session::start(pkg, common::session_opts()), Err(ExtError::Script(m)) if m.contains("hỏng lúc nạp")));
}

#[test]
fn het_gio_tung_chuong() {
    let pkg = common::package(&[("main.js", r#"
        export async function toc() { return [{ id: "a", title: "a" }, { id: "b", title: "b" }]; }
        export async function chapter(ctx, ch) { if (ch.id === "a") await sleep(5000); return { content: ch.id }; }"#)]);
    let mut opts = common::session_opts();
    opts.chapter_limit = Duration::from_millis(300);
    let session = Session::start(pkg, opts).unwrap();
    let items = numbered(session.toc(&json!({})).unwrap());
    let mut got = Vec::new();
    session.chapters(&json!({}), &items, &mut |o| got.push(o));
    got.sort_by_key(|o| o.index);
    assert_eq!(got[0].result, Err(ExtError::Timeout("chapter")));
    assert_eq!(got[1].result.as_ref().unwrap().content, "b");
}
