mod common;

use qt_ext::testing;
use serde_json::json;

#[test]
fn vector_chuan_va_khu_hoi() {
    let src = r#"export async function run() {
      const key = "0123456789abcdef", iv = "fedcba9876543210";
      const cbc = crypto.aes.encrypt({ mode: "cbc", key, iv, data: "第一章" });
      let wrongKey = null;
      try { crypto.aes.decrypt({ mode: "cbc", key: "fedcba9876543210", iv, data: cbc }); } catch (e) { wrongKey = e.message; }
      let badIv = null;
      try { crypto.aes.encrypt({ mode: "cbc", key, iv: "short", data: "x" }); } catch (e) { badIv = e.message; }
      const k256 = encoding.hex.decode("603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4");
      return [
        crypto.md5("abc"),
        crypto.sha1("abc"),
        crypto.sha256("abc"),
        crypto.hmac("sha256", "key", "The quick brown fox jumps over the lazy dog"),
        encoding.hex.encode(crypto.aes.encrypt({ mode: "ecb", key: encoding.hex.decode("2b7e151628aed2a6abf7158809cf4f3c"),
            data: encoding.hex.decode("6bc1bee22e409f96e93d7e117393172a"), padding: "none" })),
        encoding.hex.encode(crypto.aes.encrypt({ mode: "ecb", key: k256, data: encoding.hex.decode("6bc1bee22e409f96e93d7e117393172a"), padding: "none" })),
        encoding.utf8.decode(crypto.aes.decrypt({ mode: "cbc", key, iv, data: cbc })),
        encoding.utf8.decode(crypto.aes.decrypt({ mode: "gcm", key, iv: "012345678901",
            data: crypto.aes.encrypt({ mode: "gcm", key, iv: "012345678901", data: "天下" }) })),
        encoding.base64.encode("天"),
        encoding.utf8.decode(encoding.base64.decode("5aSp")),
        encoding.base64.encode(encoding.hex.decode("fbff"), { urlSafe: true }),
        encoding.decode(encoding.encode("天下", "gbk"), "gbk"),
        encoding.hex.encode(encoding.encode("天", "gbk")),
        crypto.randomBytes(16).length,
        wrongKey !== null, badIv,
      ];
    }"#;
    let pkg = common::package(&[("main.js", src)]);
    let v = testing::call(&pkg, "run", vec![], Default::default()).unwrap();
    assert_eq!(
        v,
        json!([
            "900150983cd24fb0d6963f7d28e17f72",
            "a9993e364706816aba3e25717850c26c9cd0d89d",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8",
            "3ad77bb40d7a3660a89ecaf32466ef97",
            "f3eed1bdb5d2a03c064b5a7e3db181f8",
            "第一章",
            "天下",
            "5aSp",
            "天",
            "-_8",
            "天下",
            "ccec",
            16,
            true,
            "AES-CBC cần iv 16 byte (đang 5)",
        ])
    );
}
