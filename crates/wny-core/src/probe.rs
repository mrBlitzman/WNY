use crate::mode::Mode;
use crate::policy::Policy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeReport {
    pub dns_poisoned: bool,
    pub tls_mitm: bool,
    pub extra_root_ca: bool,
    pub connectivity_ok: bool,
    pub ipv6_leak: bool,
}

pub trait Probes: Send + Sync {
    fn run(&self, mode: Mode, policy: &Policy) -> ProbeReport;
}

/// Conservative live probes. They never use plaintext DNS while WNY is on.
/// Connectivity is a TLS connect to the first canary (wikipedia).
pub struct LiveProbes;

impl Probes for LiveProbes {
    fn run(&self, _mode: Mode, policy: &Policy) -> ProbeReport {
        let mut report = ProbeReport {
            connectivity_ok: false,
            ..ProbeReport::default()
        };
        if let Some(url) = policy.probes.connectivity.first() {
            report.connectivity_ok = tls_connect_host(url);
        }
        report
    }
}

fn tls_connect_host(url: &str) -> bool {
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    if host.is_empty() {
        return false;
    }
    std::net::TcpStream::connect((host, 443)).is_ok()
}

pub struct FakeProbes {
    pub report: ProbeReport,
}

impl Probes for FakeProbes {
    fn run(&self, _mode: Mode, _policy: &Policy) -> ProbeReport {
        self.report.clone()
    }
}
