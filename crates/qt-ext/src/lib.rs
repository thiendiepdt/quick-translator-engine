//! Extension tải raw: manifest, sandbox QuickJS và các API app cấp cho extension (http, html, crypto,
//! encoding, log, sleep, progress). Extension là code lạ: không đọc/ghi file, mạng chỉ tới `hosts` đã khai.
//! Thiết kế: docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md.

pub mod error;
pub mod manifest;
pub mod policy;

pub use error::ExtError;
pub use manifest::{parse_manifest, AuthField, AuthScheme, AuthSpec, Developer, FieldKind, Limits, Manifest, Permission, Scope, SettingField, SettingKind};
pub use policy::HostPolicy;
