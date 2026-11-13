//! `crypto` (md5, sha1, sha256, sha512, hmac, AES cbc/ecb/gcm, randomBytes) và `encoding` (decode/encode theo
//! bảng mã, utf8, base64, hex). Dữ liệu vào là chuỗi (lấy byte UTF-8) hoặc `Uint8Array`; byte ra là `Uint8Array`.

use super::HostState;
use crate::text;
use aes::cipher::block_padding::{NoPadding, Pkcs7};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit};
use aes_gcm::aead::Aead;
use base64::Engine as _;
use hmac::Mac;
use rquickjs::{Ctx, Exception, FromJs, Function, Result, TypedArray, Value};
use sha2::Digest;
use std::rc::Rc;

pub(crate) const JS: &str = r#"
  {
    const opt = (v) => (v == null ? "" : v);
    const aes = (op) => (o) => {
      if (!o || o.key == null || o.data == null) throw new TypeError("crypto.aes: cần key và data");
      return host.aes(op, String(o.mode || "cbc"), o.key, opt(o.iv), o.data, String(o.padding || "pkcs7"));
    };
    define("crypto", freeze({
      md5: (data) => host.digest("md5", data),
      sha1: (data) => host.digest("sha1", data),
      sha256: (data) => host.digest("sha256", data),
      sha512: (data) => host.digest("sha512", data),
      hmac: (algorithm, key, data) => host.hmac(String(algorithm), key, data),
      aes: freeze({ encrypt: aes("encrypt"), decrypt: aes("decrypt") }),
      randomBytes: (n) => host.random(Math.floor(Number(n) || 0)),
    }));
    define("encoding", freeze({
      decode: (bytes, label) => host.text_decode(bytes, label == null ? "auto" : String(label)),
      encode: (value, label) => host.text_encode(String(value), label == null ? "utf-8" : String(label)),
      utf8: freeze({
        decode: (bytes) => host.text_decode(bytes, "utf-8"),
        encode: (value) => host.text_encode(String(value), "utf-8"),
      }),
      base64: freeze({
        encode: (data, o) => host.b64_encode(data, !!(o && o.urlSafe)),
        decode: (value) => host.b64_decode(String(value)),
      }),
      hex: freeze({
        encode: (data) => host.hex_encode(data),
        decode: (value) => host.hex_decode(String(value)),
      }),
    }));
  }
"#;

const MAX_RANDOM: usize = 1024;

fn type_error(ctx: &Ctx<'_>, message: &str) -> rquickjs::Error {
    Exception::throw_type(ctx, message)
}

/// Chuỗi → byte UTF-8; `Uint8Array` → chép byte.
fn bytes_of<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Vec<u8>> {
    if let Some(s) = value.as_string() {
        return Ok(s.to_string()?.into_bytes());
    }
    if let Ok(array) = TypedArray::<u8>::from_js(ctx, value) {
        // SAFETY: chép ra Vec ngay, không chạy JS nào trong lúc slice còn sống.
        return Ok(unsafe { array.as_bytes() }.map(<[u8]>::to_vec).unwrap_or_default());
    }
    Err(type_error(ctx, "cần chuỗi hoặc Uint8Array"))
}

fn digest(algorithm: &str, data: &[u8]) -> Option<String> {
    Some(match algorithm {
        "md5" => hex::encode(md5::Md5::digest(data)),
        "sha1" => hex::encode(sha1::Sha1::digest(data)),
        "sha256" => hex::encode(sha2::Sha256::digest(data)),
        "sha512" => hex::encode(sha2::Sha512::digest(data)),
        _ => return None,
    })
}

fn hmac_hex(algorithm: &str, key: &[u8], data: &[u8]) -> Option<String> {
    macro_rules! mac {
        ($hash:ty) => {{
            let mut mac = <hmac::Hmac<$hash> as Mac>::new_from_slice(key).ok()?;
            mac.update(data);
            hex::encode(mac.finalize().into_bytes())
        }};
    }
    Some(match algorithm {
        "md5" => mac!(md5::Md5),
        "sha1" => mac!(sha1::Sha1),
        "sha256" => mac!(sha2::Sha256),
        "sha512" => mac!(sha2::Sha512),
        _ => return None,
    })
}

pub(crate) fn aes(op: &str, mode: &str, key: &[u8], iv: &[u8], data: &[u8], padding: &str) -> std::result::Result<Vec<u8>, String> {
    let encrypt = match op {
        "encrypt" => true,
        "decrypt" => false,
        _ => return Err(format!("thao tác lạ {op}")),
    };
    let pad = match padding {
        "pkcs7" => true,
        "none" => false,
        other => return Err(format!("padding \"{other}\": chỉ có pkcs7 hoặc none")),
    };
    let bad_key = || format!("khoá AES phải dài 16, 24 hoặc 32 byte (đang {})", key.len());
    macro_rules! block_mode {
        ($enc:ident, $dec:ident, $init:ident, $($arg:expr),+) => {{
            macro_rules! run {
                ($cipher:ty) => {{
                    if encrypt {
                        let c = $enc::<$cipher>::$init($($arg),+).map_err(|e| e.to_string())?;
                        Ok(if pad { c.encrypt_padded_vec_mut::<Pkcs7>(data) } else { c.encrypt_padded_vec_mut::<NoPadding>(data) })
                    } else {
                        let c = $dec::<$cipher>::$init($($arg),+).map_err(|e| e.to_string())?;
                        if pad { c.decrypt_padded_vec_mut::<Pkcs7>(data) } else { c.decrypt_padded_vec_mut::<NoPadding>(data) }
                            .map_err(|_| "giải mã AES thất bại (sai khoá, iv hoặc padding)".to_string())
                    }
                }};
            }
            match key.len() {
                16 => run!(aes::Aes128),
                24 => run!(aes::Aes192),
                32 => run!(aes::Aes256),
                _ => Err(bad_key()),
            }
        }};
    }
    use cbc::{Decryptor as CbcDec, Encryptor as CbcEnc};
    use ecb::{Decryptor as EcbDec, Encryptor as EcbEnc};
    match mode {
        "cbc" => {
            if iv.len() != 16 {
                return Err(format!("AES-CBC cần iv 16 byte (đang {})", iv.len()));
            }
            block_mode!(CbcEnc, CbcDec, new_from_slices, key, iv)
        }
        "ecb" => block_mode!(EcbEnc, EcbDec, new_from_slice, key),
        "gcm" => {
            if iv.len() != 12 {
                return Err(format!("AES-GCM cần iv 12 byte (đang {})", iv.len()));
            }
            let nonce = aes_gcm::Nonce::from_slice(iv);
            macro_rules! gcm {
                ($cipher:ty) => {{
                    let c = <$cipher>::new_from_slice(key).map_err(|_| bad_key())?;
                    if encrypt { c.encrypt(nonce, data) } else { c.decrypt(nonce, data) }
                        .map_err(|_| "AES-GCM thất bại (sai khoá, iv hoặc dữ liệu bị sửa)".to_string())
                }};
            }
            match key.len() {
                16 => gcm!(aes_gcm::Aes128Gcm),
                24 => gcm!(aes_gcm::AesGcm<aes::Aes192, aes_gcm::aead::consts::U12>),
                32 => gcm!(aes_gcm::Aes256Gcm),
                _ => Err(bad_key()),
            }
        }
        other => Err(format!("mode \"{other}\": chỉ có cbc, ecb hoặc gcm")),
    }
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD};
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    [STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD].iter().find_map(|engine| engine.decode(&clean).ok())
}

pub(crate) fn install<'js>(ctx: &Ctx<'js>, state: &Rc<HostState>) -> Result<()> {
    let globals = ctx.globals();
    let touch = {
        let st = state.clone();
        move || st.touch()
    };

    let t = touch.clone();
    globals.set(
        "__host_digest",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, algorithm: String, data: Value<'js>| -> Result<String> {
            t();
            let bytes = bytes_of(&ctx, data)?;
            digest(&algorithm, &bytes).ok_or_else(|| type_error(&ctx, &format!("thuật toán băm lạ \"{algorithm}\"")))
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_hmac",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, algorithm: String, key: Value<'js>, data: Value<'js>| -> Result<String> {
            t();
            let key = bytes_of(&ctx, key)?;
            let data = bytes_of(&ctx, data)?;
            hmac_hex(&algorithm, &key, &data).ok_or_else(|| type_error(&ctx, &format!("thuật toán hmac lạ \"{algorithm}\"")))
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_aes",
        Function::new(
            ctx.clone(),
            move |ctx: Ctx<'js>, op: String, mode: String, key: Value<'js>, iv: Value<'js>, data: Value<'js>, padding: String| -> Result<TypedArray<'js, u8>> {
                t();
                let key = bytes_of(&ctx, key)?;
                let iv = bytes_of(&ctx, iv)?;
                let data = bytes_of(&ctx, data)?;
                let out = aes(&op, &mode, &key, &iv, &data, &padding).map_err(|m| Exception::throw_message(&ctx, &m))?;
                TypedArray::new(ctx, out)
            },
        )?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_random",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, n: i64| -> Result<TypedArray<'js, u8>> {
            t();
            if !(0..=MAX_RANDOM as i64).contains(&n) {
                return Err(type_error(&ctx, &format!("randomBytes: n từ 0 tới {MAX_RANDOM}")));
            }
            let mut buf = vec![0u8; n as usize];
            getrandom::fill(&mut buf).map_err(|e| Exception::throw_message(&ctx, &e.to_string()))?;
            TypedArray::new(ctx, buf)
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_text_decode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, bytes: Value<'js>, label: String| -> Result<String> {
            t();
            let bytes = bytes_of(&ctx, bytes)?;
            text::decode(&bytes, Some(&label), None).map_err(|e| Exception::throw_message(&ctx, &e.to_string()))
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_text_encode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, value: String, label: String| -> Result<TypedArray<'js, u8>> {
            t();
            let bytes = text::encode(&value, &label).map_err(|e| Exception::throw_message(&ctx, &e.to_string()))?;
            TypedArray::new(ctx, bytes)
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_b64_encode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, data: Value<'js>, url_safe: bool| -> Result<String> {
            t();
            let bytes = bytes_of(&ctx, data)?;
            Ok(if url_safe {
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
            } else {
                base64::engine::general_purpose::STANDARD.encode(bytes)
            })
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_b64_decode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, value: String| -> Result<TypedArray<'js, u8>> {
            t();
            let bytes = base64_decode(&value).ok_or_else(|| type_error(&ctx, "chuỗi base64 không hợp lệ"))?;
            TypedArray::new(ctx, bytes)
        })?,
    )?;

    let t = touch.clone();
    globals.set(
        "__host_hex_encode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, data: Value<'js>| -> Result<String> {
            t();
            Ok(hex::encode(bytes_of(&ctx, data)?))
        })?,
    )?;

    let t = touch;
    globals.set(
        "__host_hex_decode",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, value: String| -> Result<TypedArray<'js, u8>> {
            t();
            let bytes = hex::decode(value.trim()).map_err(|_| type_error(&ctx, "chuỗi hex không hợp lệ"))?;
            TypedArray::new(ctx, bytes)
        })?,
    )?;
    Ok(())
}
