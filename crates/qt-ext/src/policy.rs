//! Domain extension được gọi tới. Kiểm ở mọi request và mọi bước redirect, cho cả `http` lẫn webview.
//! Mạng nội bộ (IP riêng, loopback, link-local, đuôi tên miền nội bộ) luôn bị chặn, trừ chế độ developer
//! (`allow_local`) với đúng host đã khai — để chạy thử extension với máy chủ giả trên máy.

use crate::ExtError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::{Host, Url};

/// Tên miền chỉ phân giải được trong mạng nội bộ (như `qt-ai-core::source_http`).
const LOCAL_DOMAIN_SUFFIXES: [&str; 7] = [".localhost", ".local", ".internal", ".intranet", ".lan", ".home", ".corp"];

pub(crate) fn is_local_name(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost" || LOCAL_DOMAIN_SUFFIXES.iter().any(|suffix| host.ends_with(suffix))
}

fn is_private_v4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast() || ip.octets()[0] == 0
        // 100.64.0.0/10 (CGNAT)
        || (ip.octets()[0] == 100 && (ip.octets()[1] & 0xc0) == 64)
}

fn is_private_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_private_v4(v4);
    }
    let first = ip.segments()[0];
    ip.is_loopback() || ip.is_unspecified() || (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80
}

pub(crate) fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_v4(v4),
        IpAddr::V6(v6) => is_private_v6(v6),
    }
}

#[derive(Debug, Clone)]
pub struct HostPolicy {
    patterns: Vec<String>,
    allow_local: bool,
}

impl HostPolicy {
    pub fn new(hosts: &[String], allow_local: bool) -> Self {
        HostPolicy { patterns: hosts.iter().map(|h| h.trim().to_ascii_lowercase()).collect(), allow_local }
    }

    fn listed(&self, host: &str) -> bool {
        self.patterns.iter().any(|pattern| match pattern.strip_prefix("*.") {
            Some(root) => host.len() > root.len() + 1 && host.ends_with(root) && host[..host.len() - root.len()].ends_with('.'),
            None => host == pattern,
        })
    }

    pub fn check(&self, url: &Url) -> Result<(), ExtError> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ExtError::HostDenied(format!("{}://", url.scheme())));
        }
        let (name, local) = match url.host() {
            Some(Host::Domain(domain)) => {
                let domain = domain.trim_end_matches('.').to_ascii_lowercase();
                let local = is_local_name(&domain);
                (domain, local)
            }
            Some(Host::Ipv4(ip)) => (ip.to_string(), is_private_v4(ip)),
            Some(Host::Ipv6(ip)) => (ip.to_string(), is_private_v6(ip)),
            None => return Err(ExtError::HostDenied(String::new())),
        };
        if local && !self.allow_local {
            return Err(ExtError::HostDenied(name));
        }
        if self.listed(&name) {
            Ok(())
        } else {
            Err(ExtError::HostDenied(name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chi_cho_host_da_khai() {
        let p = HostPolicy::new(&["demo.test".into(), "*.cdn.test".into()], false);
        let ok = |u: &str| p.check(&Url::parse(u).unwrap()).is_ok();
        assert!(ok("https://demo.test/a"));
        assert!(ok("http://demo.test/a"));
        assert!(ok("https://DEMO.test./a"));
        assert!(ok("https://img.cdn.test/a.jpg"));
        assert!(ok("https://a.b.cdn.test/x"));
        assert!(!ok("https://cdn.test/x"), "*. không gồm chính tên gốc");
        assert!(!ok("https://xcdn.test/x"));
        assert!(!ok("https://www.demo.test/a"), "không tự gồm www");
        assert!(!ok("https://evil.test/a"));
        assert!(!ok("https://demo.test.evil.com/a"));
        assert!(!ok("ftp://demo.test/a"));
        assert!(ok("https://user:pw@demo.test/a"), "userinfo không đổi host");
    }

    #[test]
    fn chan_noi_bo_tru_khi_allow_local() {
        let p = HostPolicy::new(&["localhost".into(), "127.0.0.1".into(), "box.lan".into()], false);
        for u in ["http://localhost:8080/", "http://127.0.0.1/", "http://box.lan/", "http://10.0.0.2/", "http://[::1]/", "http://169.254.1.1/", "http://[::ffff:192.168.1.1]/"] {
            assert!(p.check(&Url::parse(u).unwrap()).is_err(), "{u}");
        }
        let dev = HostPolicy::new(&["localhost".into(), "127.0.0.1".into()], true);
        assert!(dev.check(&Url::parse("http://127.0.0.1:9000/x").unwrap()).is_ok());
        assert!(dev.check(&Url::parse("http://localhost:9000/x").unwrap()).is_ok());
        assert!(dev.check(&Url::parse("http://10.0.0.2/x").unwrap()).is_err(), "allow_local vẫn chỉ host đã khai");
    }
}
