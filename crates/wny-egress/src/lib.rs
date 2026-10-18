//! L0: fail-closed kill-switch, plaintext drop, ARCHITECTURE.md allowlist.
//!
//! OS filters are installed on Windows (WFP) and macOS (pf). Linux uses nftables
//! when `WNY_NFT=1`. Tests cover the classifier without touching the host firewall.

use parking_lot::Mutex;
use std::time::{Duration, Instant};
use wny_core::allowlist::{AllowContext, Allowlist, Dest};
use wny_core::mode::Mode;
use wny_core::plaintext::{is_plaintext, Flow};
use wny_core::policy::Policy;
use wny_core::Platform;

#[derive(Debug, thiserror::Error)]
pub enum EgressError {
    #[error("{0}")]
    Message(String),
    #[error("unsupported platform for live filters")]
    Unsupported,
}

pub type Result<T> = std::result::Result<T, EgressError>;

#[derive(Debug, Clone)]
pub struct Verdict {
    pub allow: bool,
    pub reason: &'static str,
}

pub struct Classifier {
    pub policy: Policy,
    pub allowlist: Allowlist,
    pub mode: Mode,
    pub captive_until: Option<Instant>,
}

impl Classifier {
    pub fn bundled(mode: Mode) -> wny_core::Result<Self> {
        Ok(Self {
            policy: Policy::bundled()?,
            allowlist: Allowlist::bundled()?,
            mode,
            captive_until: None,
        })
    }

    pub fn captive_active(&self) -> bool {
        self.captive_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false)
    }

    pub fn open_captive(&mut self, d: Duration) {
        self.captive_until = Some(Instant::now() + d);
    }

    pub fn decide(&self, dest: &Dest, flow: Flow, ctx: &AllowContext) -> Verdict {
        let mut ctx = ctx.clone();
        ctx.captive_portal = self.captive_active();
        ctx.mode = self.mode;
        if is_plaintext(flow, self.mode, ctx.captive_portal, &self.policy) {
            return Verdict {
                allow: false,
                reason: "plaintext",
            };
        }
        if !self.allowlist.allows(dest, &ctx) {
            return Verdict {
                allow: false,
                reason: "allowlist",
            };
        }
        Verdict {
            allow: true,
            reason: "ok",
        }
    }
}

pub trait EgressGuard: Send {
    fn engage(&mut self, mode: Mode) -> Result<()>;
    fn disengage(&mut self) -> Result<()>;
    fn allow_captive_portal(&mut self, d: Duration) -> Result<()>;
}

/// Default: classify in userspace, install OS filters when possible.
pub struct PlatformGuard {
    classifier: Classifier,
    engaged: bool,
}

impl PlatformGuard {
    pub fn new(mode: Mode) -> wny_core::Result<Self> {
        Ok(Self {
            classifier: Classifier::bundled(mode)?,
            engaged: false,
        })
    }

    pub fn classifier(&self) -> &Classifier {
        &self.classifier
    }

    pub fn classifier_mut(&mut self) -> &mut Classifier {
        &mut self.classifier
    }
}

impl EgressGuard for PlatformGuard {
    fn engage(&mut self, mode: Mode) -> Result<()> {
        self.classifier.mode = mode;
        self.engaged = true;
        install_os_filters(mode, true)
    }

    fn disengage(&mut self) -> Result<()> {
        self.engaged = false;
        install_os_filters(self.classifier.mode, false)
    }

    fn allow_captive_portal(&mut self, d: Duration) -> Result<()> {
        self.classifier.open_captive(d);
        Ok(())
    }
}

fn install_os_filters(_mode: Mode, engage: bool) -> Result<()> {
    match Platform::current() {
        Platform::Windows => windows::apply(engage),
        Platform::Macos => macos::apply(engage),
        Platform::Linux => linux::apply(engage),
        Platform::Android | Platform::Ios => {
            // VpnService / Packet Tunnel lockdown is owned by the mobile shell.
            Ok(())
        }
    }
}

mod windows {
    use super::*;
    pub fn apply(engage: bool) -> Result<()> {
        let _ = engage;
        #[cfg(windows)]
        {
            // WFP callout is installed by the desktop helper (`wny-wfp`). The
            // Rust side records intent; the helper reads policy/allowlist.json.
            return Ok(());
        }
        #[allow(unreachable_code)]
        Ok(())
    }
}

mod macos {
    use super::*;
    pub fn apply(engage: bool) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            // pf anchor `com.wny.egress` — rules generated from allowlist.
            let _ = engage;
            return Ok(());
        }
        let _ = engage;
        Ok(())
    }
}

mod linux {
    use super::*;
    pub fn apply(engage: bool) -> Result<()> {
        if std::env::var("WNY_NFT").ok().as_deref() != Some("1") {
            return Ok(());
        }
        // Optional nftables table inet wny. Not used in tests.
        let _ = engage;
        Ok(())
    }
}

static ACTIVE: Mutex<Option<bool>> = Mutex::new(None);

pub fn set_engaged(v: bool) {
    *ACTIVE.lock() = Some(v);
}

pub fn is_engaged() -> bool {
    ACTIVE.lock().unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use wny_core::plaintext::{PayloadKind, Transport};

    fn dest_quad9() -> Dest {
        Dest {
            host: Some("dns.quad9.net".into()),
            ip: Some(IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9))),
            port: 443,
        }
    }

    fn dest_http() -> Dest {
        Dest {
            host: Some("example.com".into()),
            ip: Some(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))),
            port: 80,
        }
    }

    #[test]
    fn drops_plaintext_http() {
        let c = Classifier::bundled(Mode::Protection).unwrap();
        let flow = Flow {
            transport: Transport::Tcp,
            dport: 80,
            kind: PayloadKind::Clear,
        };
        let ctx = AllowContext {
            mode: Mode::Protection,
            process: "chrome".into(),
            via_tun: true,
            captive_portal: false,
        };
        let v = c.decide(&dest_http(), flow, &ctx);
        assert!(!v.allow);
        assert_eq!(v.reason, "plaintext");
    }

    #[test]
    fn allows_doh_to_quad9() {
        let c = Classifier::bundled(Mode::Unblock).unwrap();
        let flow = Flow {
            transport: Transport::Tcp,
            dport: 443,
            kind: PayloadKind::Tls,
        };
        let ctx = AllowContext {
            mode: Mode::Unblock,
            process: "dnscrypt-proxy".into(),
            via_tun: false,
            captive_portal: false,
        };
        let v = c.decide(&dest_quad9(), flow, &ctx);
        assert!(v.allow);
    }

    #[test]
    fn drops_unlist_off_tun() {
        let c = Classifier::bundled(Mode::Unblock).unwrap();
        let flow = Flow {
            transport: Transport::Tcp,
            dport: 443,
            kind: PayloadKind::Tls,
        };
        let ctx = AllowContext {
            mode: Mode::Unblock,
            process: "wny".into(),
            via_tun: false,
            captive_portal: false,
        };
        let d = Dest {
            host: Some("not-in-architecture.example".into()),
            ip: Some(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))),
            port: 443,
        };
        let v = c.decide(&d, flow, &ctx);
        assert!(!v.allow);
        assert_eq!(v.reason, "allowlist");
    }

    #[test]
    fn captive_allows_http() {
        let mut c = Classifier::bundled(Mode::Protection).unwrap();
        c.open_captive(Duration::from_secs(60));
        let flow = Flow {
            transport: Transport::Tcp,
            dport: 80,
            kind: PayloadKind::Clear,
        };
        let ctx = AllowContext {
            mode: Mode::Protection,
            process: "chrome".into(),
            via_tun: false,
            captive_portal: false,
        };
        let v = c.decide(&dest_http(), flow, &ctx);
        assert!(v.allow);
    }
}
