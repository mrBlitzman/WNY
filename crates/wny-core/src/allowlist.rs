use crate::error::{Error, Result};
use crate::mode::Mode;
use ipnet::IpNet;
use serde::Deserialize;
use std::net::IpAddr;
use std::str::FromStr;

pub const ALLOWLIST_JSON: &str = include_str!("../../../policy/allowlist.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Allowlist {
    pub version: u32,
    pub static_endpoints: Vec<StaticEndpoint>,
    pub named_classes: Vec<NamedClass>,
    pub loopback: Vec<String>,
    pub tun_prefixes: Vec<String>,
    pub forbidden: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaticEndpoint {
    pub id: String,
    pub class: String,
    #[serde(default)]
    pub hosts: Vec<String>,
    #[serde(default)]
    pub ips: Vec<String>,
    #[serde(default)]
    pub cidrs: Vec<String>,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub modes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamedClass {
    pub id: String,
    #[serde(default)]
    pub process: serde_json::Value,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Dest {
    pub host: Option<String>,
    pub ip: Option<IpAddr>,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct AllowContext {
    pub mode: Mode,
    pub process: String,
    pub via_tun: bool,
    pub captive_portal: bool,
}

impl Allowlist {
    pub fn bundled() -> Result<Self> {
        serde_json::from_str(ALLOWLIST_JSON).map_err(|e| Error::Policy(e.to_string()))
    }

    pub fn is_forbidden_host(&self, host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        self.forbidden.iter().any(|f| h == *f || h.ends_with(&format!(".{f}")))
    }

    pub fn is_loopback_or_tun(&self, ip: IpAddr) -> bool {
        for cidr in self.loopback.iter().chain(self.tun_prefixes.iter()) {
            if let Ok(net) = IpNet::from_str(cidr) {
                if net.contains(&ip) {
                    return true;
                }
            }
        }
        ip.is_loopback()
    }

    pub fn class_allows_process(&self, class_id: &str, process: &str) -> bool {
        let Some(c) = self.named_classes.iter().find(|c| c.id == class_id) else {
            return false;
        };
        match &c.process {
            serde_json::Value::String(s) => s == process,
            serde_json::Value::Array(arr) => arr.iter().any(|v| v.as_str() == Some(process)),
            _ => false,
        }
    }

    /// User-app packets on the TUN are allowed: L0 already captured them.
    /// Direct off-TUN packets must match a static endpoint or a named class for *this* process.
    pub fn allows(&self, dest: &Dest, ctx: &AllowContext) -> bool {
        if ctx.captive_portal {
            return true;
        }
        if let Some(h) = &dest.host {
            if self.is_forbidden_host(h) {
                return false;
            }
        }
        if let Some(ip) = dest.ip {
            if self.is_loopback_or_tun(ip) {
                return true;
            }
        }
        if ctx.via_tun {
            // Apps inside the tunnel do not need the static allowlist; the
            // tunnel process itself is still bound by named classes / static.
            return true;
        }
        self.static_hit(dest, ctx)
    }

    fn static_hit(&self, dest: &Dest, ctx: &AllowContext) -> bool {
        let mode = ctx.mode.as_str();
        for ep in &self.static_endpoints {
            if !ep.modes.is_empty() && !ep.modes.iter().any(|m| m == mode) {
                continue;
            }
            if !ep.ports.is_empty() && !ep.ports.contains(&dest.port) {
                continue;
            }
            if let Some(h) = &dest.host {
                let hl = h.to_ascii_lowercase();
                if ep.hosts.iter().any(|eh| eh.eq_ignore_ascii_case(&hl)) {
                    return true;
                }
            }
            if let Some(ip) = dest.ip {
                if ep.ips.iter().any(|s| s.parse::<IpAddr>().ok() == Some(ip)) {
                    return true;
                }
                for c in &ep.cidrs {
                    if let Ok(net) = IpNet::from_str(c) {
                        if net.contains(&ip) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn ctx(process: &str, via_tun: bool) -> AllowContext {
        AllowContext {
            mode: Mode::Unblock,
            process: process.into(),
            via_tun,
            captive_portal: false,
        }
    }

    #[test]
    fn bundled_parses() {
        let a = Allowlist::bundled().unwrap();
        assert_eq!(a.version, 1);
        assert!(a.static_endpoints.iter().any(|e| e.id == "quad9-doh"));
    }

    #[test]
    fn quad9_allowed_for_resolver() {
        let a = Allowlist::bundled().unwrap();
        let d = Dest {
            host: Some("dns.quad9.net".into()),
            ip: Some(IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9))),
            port: 443,
        };
        assert!(a.allows(&d, &ctx("dnscrypt-proxy", false)));
    }

    #[test]
    fn random_host_denied_off_tun() {
        let a = Allowlist::bundled().unwrap();
        let d = Dest {
            host: Some("evil.example".into()),
            ip: Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            port: 443,
        };
        assert!(!a.allows(&d, &ctx("wny", false)));
        assert!(a.allows(&d, &ctx("chrome", true)));
    }

    #[test]
    fn forbidden_nextdns() {
        let a = Allowlist::bundled().unwrap();
        assert!(a.is_forbidden_host("dns.nextdns.io"));
    }

    #[test]
    fn odoh_not_in_private() {
        let a = Allowlist::bundled().unwrap();
        let d = Dest {
            host: Some("odoh.cloudflare-dns.com".into()),
            ip: None,
            port: 443,
        };
        let mut c = ctx("dnscrypt-proxy", false);
        c.mode = Mode::Private;
        assert!(!a.allows(&d, &c));
        c.mode = Mode::Protection;
        assert!(a.allows(&d, &c));
    }

    #[test]
    fn tor_class_process() {
        let a = Allowlist::bundled().unwrap();
        assert!(a.class_allows_process("tor-relays", "tor"));
        assert!(!a.class_allows_process("tor-relays", "chrome"));
        assert!(a.class_allows_process("tor-bridges", "lyrebird"));
    }
}
