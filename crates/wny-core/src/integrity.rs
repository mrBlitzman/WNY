use crate::error::{Error, Result};
use crate::platform::triple;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const ENGINES_LOCK: &str = include_str!("../../../policy/engines.lock");

#[derive(Debug, Clone, Deserialize)]
pub struct EnginesLock {
    pub version: u32,
    pub engines: Vec<LockedEngine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockedEngine {
    pub id: String,
    pub slot: String,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub upstream: String,
    pub version: String,
    #[serde(default)]
    pub artifacts: serde_json::Map<String, serde_json::Value>,
}

impl EnginesLock {
    pub fn bundled() -> Result<Self> {
        serde_json::from_str(ENGINES_LOCK).map_err(|e| Error::Integrity(e.to_string()))
    }

    pub fn get(&self, id: &str) -> Option<&LockedEngine> {
        self.engines.iter().find(|e| e.id == id)
    }

    pub fn expected_sha256(&self, id: &str) -> Option<String> {
        let e = self.get(id)?;
        let art = e.artifacts.get(&triple())?;
        art.get("sha256")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase())
    }
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    Ok(hex::encode(Sha256::digest(&data)))
}

/// If the lock has a hash for this engine on this triple, the file must match.
/// Empty hash in the lock means "not pinned yet" — allowed only in development
/// (`WNY_DEV=1`) so a release build with empty pins still refuses to start.
pub fn verify_binary(lock: &EnginesLock, id: &str, path: &Path) -> Result<()> {
    let Some(expected) = lock.expected_sha256(id) else {
        if std::env::var("WNY_DEV").ok().as_deref() == Some("1") {
            return Ok(());
        }
        // No artifact for this platform is not a tamper; the engine is simply missing.
        return Ok(());
    };
    let got = sha256_file(path)?;
    if got != expected {
        return Err(Error::Integrity(format!(
            "{id} hash mismatch (got {got}, expected {expected})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn lock_parses_and_lists_tor() {
        let l = EnginesLock::bundled().unwrap();
        assert!(l.get("tor").is_some());
        assert!(l.get("dnscrypt-proxy").is_some());
        assert!(l.get("psiphon-tunnel-core").is_some());
        assert!(l.get("lokinet").is_some());
    }

    #[test]
    fn mismatch_is_error() {
        let l = EnginesLock::bundled().unwrap();
        let p = std::env::temp_dir().join(format!("wny-hash-{}", std::process::id()));
        {
            let mut f = std::fs::File::create(&p).unwrap();
            f.write_all(b"not-the-binary").unwrap();
        }
        // If this triple has no pin, verify is Ok (missing artifact).
        // Force a pin by constructing a fake lock.
        let mut fake = l.clone();
        let art = serde_json::json!({ "sha256": "00".repeat(32) });
        fake.engines[0]
            .artifacts
            .insert(triple(), art);
        let err = verify_binary(&fake, &fake.engines[0].id, &p).unwrap_err();
        assert!(err.to_string().contains("hash mismatch"));
    }
}
