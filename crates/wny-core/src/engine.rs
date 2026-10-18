use crate::error::Result;
use crate::mode::Slot;
use crate::platform::Platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Health {
    Up,
    Down,
    Missing,
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub id: String,
    pub slot: Slot,
    pub platforms: Vec<Platform>,
}

pub trait Engine: Send + Sync {
    fn info(&self) -> EngineInfo;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn health(&self) -> Health;
    /// Tor NEWNYM / new circuit. Default no-op.
    fn new_identity(&self) -> Result<()> {
        Ok(())
    }
}

pub fn supported_here(info: &EngineInfo) -> bool {
    let p = Platform::current();
    info.platforms.is_empty() || info.platforms.contains(&p)
}
