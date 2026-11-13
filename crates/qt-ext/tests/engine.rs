mod common;

use qt_ext::{testing, EngineOptions, ExtError};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[test]
fn goi_ham_export_tra_json_va_ctx_bi_dong_bang() {
    let pkg = common::package(&[
        ("main.js", r#"
            import { twice } from "./lib/u.js";
            export async function toc(ctx) {
                let frozen = false;
                try { ctx.story.name = "x"; } catch (e) { frozen = true; }
                return [{ id: "1", title: ctx.story.name + twice(2), frozen }];
            }
            export function plain(a, b) { return a + b; }"#),
        ("lib/u.js", "export const twice = (x) => x * 2;"),
    ]);
    let v = testing::call(&pkg, "toc", vec![json!({"story": {"name": "A"}})], Default::default()).unwrap();
    assert_eq!(v, json!([{ "id": "1", "title": "A4", "frozen": true }]));
    assert_eq!(testing::call(&pkg, "plain", vec![json!(2), json!(3)], Default::default()).unwrap(), json!(5), "hàm thường, nhiều tham số");
    assert!(matches!(testing::call(&pkg, "nope", vec![], Default::default()), Err(ExtError::Script(m)) if m.contains("nope")));
}

#[test]
fn doi_loi_cua_extension() {
    let pkg = common::package(&[("main.js", r#"
        export async function a() { throw new AuthExpired("hết phiên"); }
        export async function b() { throw new Locked("VIP"); }
        export async function c() { throw new ExtensionError("tạm lỗi", { retry: true }); }
        export async function d() { throw new NeedsInteraction("cần đăng nhập"); }
        export async function e() { null.x; }
        export async function f() { throw "chuỗi"; }
        export async function g() { throw new ExtensionError("hỏng hẳn"); }"#)]);
    let call = |f| testing::call(&pkg, f, vec![], Default::default()).unwrap_err();
    assert_eq!(call("a"), ExtError::AuthExpired("hết phiên".into()));
    assert_eq!(call("b"), ExtError::Locked("VIP".into()));
    assert_eq!(call("c"), ExtError::Extension { message: "tạm lỗi".into(), retry: true });
    assert_eq!(call("d"), ExtError::NeedsInteraction("cần đăng nhập".into()));
    assert!(matches!(call("e"), ExtError::Script(m) if m.contains("TypeError")));
    assert!(matches!(call("f"), ExtError::Script(m) if m.contains("chuỗi")));
    assert_eq!(call("g"), ExtError::Extension { message: "hỏng hẳn".into(), retry: false });
}

#[test]
fn ngat_vong_lap_vo_han_va_vuot_ram() {
    let pkg = common::package(&[("main.js", r#"
        export async function spin() { while (true) {} }
        export async function spinCatch() { try { while (true) {} } catch (e) { return "bắt được"; } }
        export async function hog() { const a = []; while (true) a.push(new Array(1e5).fill(1)); }"#)]);
    let mut opts = EngineOptions::default();
    opts.cpu_slice = Duration::from_millis(300);
    opts.memory_limit = 32 * 1024 * 1024;
    let t = Instant::now();
    assert_eq!(testing::call(&pkg, "spin", vec![], opts.clone()).unwrap_err(), ExtError::Interrupted);
    assert!(t.elapsed().as_millis() < 2000);
    assert_eq!(testing::call(&pkg, "spinCatch", vec![], opts.clone()).unwrap_err(), ExtError::Interrupted, "try/catch không nuốt được lệnh ngắt");
    assert_eq!(testing::call(&pkg, "hog", vec![], opts).unwrap_err(), ExtError::OutOfMemory);
}

#[test]
fn cho_io_khong_tinh_vao_thoi_gian_chay_lien_tuc() {
    let pkg = common::package(&[("main.js", "export async function wait() { await sleep(500); await sleep(500); return 1; }")]);
    let mut opts = EngineOptions::default();
    opts.cpu_slice = Duration::from_millis(300);
    assert_eq!(testing::call(&pkg, "wait", vec![], opts).unwrap(), json!(1));
}

#[test]
fn khong_co_api_he_thong_va_che_khoa_trong_log() {
    let pkg = common::package(&[("main.js", r#"
        export async function probe() {
            log.info("khoá là sk-SECRET-123", { a: 1 });
            let overwritten = true;
            try { log = null; } catch (e) {}
            try { globalThis.log = null; } catch (e) {}
            overwritten = log === null;
            return [typeof require, typeof process, typeof std, typeof os, typeof __host_log, typeof fetch, overwritten];
        }
        export async function boom() { throw new Error("lộ sk-SECRET-123"); }"#)]);
    let logs = Arc::new(Mutex::new(Vec::new()));
    let sink = logs.clone();
    let mut opts = EngineOptions::default();
    opts.secrets = vec!["sk-SECRET-123".into()];
    opts.log = Arc::new(move |_lvl, msg| sink.lock().unwrap().push(msg));
    let v = testing::call(&pkg, "probe", vec![], opts.clone()).unwrap();
    assert_eq!(v, json!(["undefined", "undefined", "undefined", "undefined", "undefined", "undefined", false]));
    assert_eq!(logs.lock().unwrap().as_slice(), ["khoá là *** {\"a\":1}"]);
    let e = testing::call(&pkg, "boom", vec![], opts).unwrap_err().to_string();
    assert!(e.contains("***") && !e.contains("SECRET"), "{e}");
}

#[test]
fn het_han_va_huy() {
    let pkg = common::package(&[("main.js", "export async function slow() { await sleep(5000); return 1; }")]);
    let e = testing::call_with_limit(&pkg, "slow", vec![], Default::default(), Duration::from_millis(200)).unwrap_err();
    assert_eq!(e, ExtError::Timeout("call"));

    let opts = EngineOptions::default();
    let cancel = opts.cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    let t = Instant::now();
    assert_eq!(testing::call(&pkg, "slow", vec![], opts).unwrap_err(), ExtError::Cancelled);
    assert!(t.elapsed().as_millis() < 1500);
}

#[test]
fn import_ra_ngoai_goi_bi_chan() {
    let pkg = common::package(&[("main.js", r#"export async function esc() { return await import("../secret.js"); }
                                              export async function bare() { return await import("os"); }"#)]);
    assert!(matches!(testing::call(&pkg, "esc", vec![], Default::default()), Err(ExtError::Script(m)) if m.contains("trong gói")));
    assert!(matches!(testing::call(&pkg, "bare", vec![], Default::default()), Err(ExtError::Script(m)) if m.contains("trong gói")));
}
