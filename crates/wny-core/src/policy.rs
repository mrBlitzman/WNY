use crate::error::{Error, Result};
use crate::mode::{Mode, Slot};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const POLICY_JSON: &str = include_str!("../../../policy/policy.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Policy {
    pub version: u32,
    pub modes: BTreeMap<String, ModePolicy>,
    pub probes: ProbePolicy,
    pub captive_portal_secs: u64,
    pub plaintext_drop: PlaintextDrop,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModePolicy {
    pub names: SlotPolicy,
    pub filter: SlotPolicy,
    pub road: SlotPolicy,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SlotPolicy {
    #[serde(default = "default_auto")]
    pub choice: String,
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(default)]
    pub on_mitm: Vec<String>,
    #[serde(default)]
    pub only_if_road_bootstrap_fails: bool,
    #[serde(default)]
    pub fail: Option<String>,
}

fn default_auto() -> String {
    "auto".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProbePolicy {
    pub dns: Vec<String>,
    pub tls_issuers: Vec<String>,
    pub connectivity: Vec<String>,
    pub jitter_ms: [u64; 2],
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaintextDrop {
    pub tcp_ports: Vec<u16>,
    pub udp_ports: Vec<u16>,
    pub private_also_drop_udp_443: bool,
}

impl Policy {
    pub fn bundled() -> Result<Self> {
        serde_json::from_str(POLICY_JSON).map_err(|e| Error::Policy(e.to_string()))
    }

    pub fn mode(&self, mode: Mode) -> Result<&ModePolicy> {
        self.modes
            .get(mode.as_str())
            .ok_or_else(|| Error::Policy(format!("missing mode {}", mode.as_str())))
    }

    pub fn order(&self, mode: Mode, slot: Slot) -> Result<&[String]> {
        let m = self.mode(mode)?;
        Ok(match slot {
            Slot::Names => m.names.order.as_slice(),
            Slot::Filter => m.filter.order.as_slice(),
            Slot::Road => m.road.order.as_slice(),
        })
    }

    pub fn fail_closed(&self, mode: Mode) -> bool {
        self.mode(mode)
            .ok()
            .and_then(|m| m.road.fail.as_deref())
            .map(|s| s.eq_ignore_ascii_case("closed"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_policy_parses() {
        let p = Policy::bundled().unwrap();
        assert_eq!(p.version, 1);
        assert!(p.mode(Mode::Private).unwrap().road.order.contains(&"tor".into()));
        assert!(p.fail_closed(Mode::Private));
        assert!(!p.fail_closed(Mode::Protection));
    }
}
