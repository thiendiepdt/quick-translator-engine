//! Extension tải raw: manifest, sandbox QuickJS và các API app cấp cho extension (http, html, crypto,
//! encoding, log, sleep, progress). Extension là code lạ: không đọc/ghi file, mạng chỉ tới `hosts` đã khai.
//! Thiết kế: docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md.

pub mod error;
pub mod manifest;
pub mod policy;
pub mod text;
pub mod net;
pub mod package;
mod engine;
pub mod session;
pub mod store;
#[doc(hidden)]
pub mod testkit;

pub use error::ExtError;
pub use manifest::{check_setting, parse_manifest, AuthField, AuthScheme, AuthSpec, Developer, FieldKind, Limits, Manifest, Permission, Scope, SettingField, SettingKind};
pub use policy::HostPolicy;
pub use net::{HttpClient, HttpRequest, HttpResponse};
pub use package::Package;
pub use store::{AuthState, AuthValues, ExtStore, SecretBackend};
pub use session::{ChapterContent, ChapterFormat, ChapterOutcome, Session, SessionOptions, TocItem};
pub use engine::{redact, EngineOptions, LogLevel, LogSink, ProgressSink};
#[doc(hidden)]
pub use engine::testing;
