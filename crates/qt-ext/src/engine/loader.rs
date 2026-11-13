//! Module JS chỉ lấy từ file của gói (đọc sẵn trong bộ nhớ). Chỉ nhận đường dẫn tương đối (`./`, `../`) từ một
//! module khác của gói, và `main.js` ở gốc; không cho thoát ra ngoài gói, không có module hệ thống hay tên trần.

use rquickjs::loader::{ImportAttributes, Loader, Resolver};
use rquickjs::module::Declared;
use rquickjs::{Ctx, Error, Module, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// `base`: module đang import (rỗng nếu là điểm vào). Trả `None` nếu đường dẫn không hợp lệ hoặc thoát gói.
pub(crate) fn normalize(base: &str, name: &str) -> Option<String> {
    if base.is_empty() {
        return (name == crate::package::ENTRY).then(|| name.to_string());
    }
    if !(name.starts_with("./") || name.starts_with("../")) {
        return None;
    }
    let mut parts: Vec<&str> = base.split('/').collect();
    parts.pop();
    for segment in name.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

pub(crate) struct MemResolver(pub Arc<HashMap<String, String>>);

impl Resolver for MemResolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, base: &str, name: &str, _attributes: Option<ImportAttributes<'js>>) -> Result<String> {
        match normalize(base, name) {
            Some(path) if self.0.contains_key(&path) => Ok(path),
            _ => Err(Error::new_resolving_message(base, name, "chỉ import được file trong gói extension")),
        }
    }
}

pub(crate) struct MemLoader(pub Arc<HashMap<String, String>>);

impl Loader for MemLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str, _attributes: Option<ImportAttributes<'js>>) -> Result<Module<'js, Declared>> {
        let source = self.0.get(name).ok_or_else(|| Error::new_loading(name))?.clone();
        Module::declare(ctx.clone(), name, source)
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn duong_dan_module_khong_thoat_goi() {
        assert_eq!(normalize("", "main.js").as_deref(), Some("main.js"));
        assert_eq!(normalize("", "lib/a.js"), None, "điểm vào chỉ là main.js");
        assert_eq!(normalize("main.js", "./lib/a.js").as_deref(), Some("lib/a.js"));
        assert_eq!(normalize("lib/a.js", "../b.js").as_deref(), Some("b.js"));
        assert_eq!(normalize("lib/a.js", "./c/../d.js").as_deref(), Some("lib/d.js"));
        assert_eq!(normalize("main.js", "../x.js"), None);
        assert_eq!(normalize("lib/a.js", "../../x.js"), None);
        assert_eq!(normalize("main.js", "fs"), None);
        assert_eq!(normalize("main.js", "/etc/passwd"), None);
        assert_eq!(normalize("main.js", "C:/Windows/x.js"), None);
    }
}
