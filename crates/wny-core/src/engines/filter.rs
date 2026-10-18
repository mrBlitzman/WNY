use super::process::{binary_exists, child_up, spawn_headless, stop_child, ChildCell};
use crate::engine::{Engine, EngineInfo, Health};
use crate::error::Result;
use crate::mode::Slot;
use crate::platform::Platform;
use parking_lot::Mutex;
use std::sync::Arc;

struct FilterProc {
    id: &'static str,
    bin: &'static str,
    args: Vec<String>,
    platforms: Vec<Platform>,
    child: ChildCell,
}

impl FilterProc {
    fn new(id: &'static str, bin: &'static str, args: &[&str], platforms: Vec<Platform>) -> Self {
        Self {
            id,
            bin,
            args: args.iter().map(|s| s.to_string()).collect(),
            platforms,
            child: Arc::new(Mutex::new(None)),
        }
    }
}

impl Engine for FilterProc {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: self.id.into(),
            slot: Slot::Filter,
            platforms: self.platforms.clone(),
        }
    }
    fn start(&self) -> Result<()> {
        let child = spawn_headless(self.id, self.bin, &self.args)?;
        *self.child.lock() = Some(child);
        Ok(())
    }
    fn stop(&self) -> Result<()> {
        stop_child(&self.child)
    }
    fn health(&self) -> Health {
        if child_up(&self.child) {
            Health::Up
        } else if binary_exists(self.bin) {
            Health::Down
        } else {
            Health::Missing
        }
    }
}

pub struct GoodbyeDpi {
    inner: FilterProc,
}
impl Default for GoodbyeDpi {
    fn default() -> Self {
        Self {
            inner: FilterProc::new(
                "goodbyedpi",
                "goodbyedpi",
                &["-5"],
                vec![Platform::Windows],
            ),
        }
    }
}
impl Engine for GoodbyeDpi {
    fn info(&self) -> EngineInfo {
        self.inner.info()
    }
    fn start(&self) -> Result<()> {
        self.inner.start()
    }
    fn stop(&self) -> Result<()> {
        self.inner.stop()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
}

pub struct Zapret {
    inner: FilterProc,
}
impl Default for Zapret {
    fn default() -> Self {
        let plat = vec![Platform::Windows, Platform::Macos, Platform::Linux];
        let bin = if cfg!(windows) { "winws" } else { "tpws" };
        Self {
            inner: FilterProc::new("zapret", bin, &["--dpi-desync=fake,disorder"], plat),
        }
    }
}
impl Engine for Zapret {
    fn info(&self) -> EngineInfo {
        self.inner.info()
    }
    fn start(&self) -> Result<()> {
        self.inner.start()
    }
    fn stop(&self) -> Result<()> {
        self.inner.stop()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
}

pub struct SpoofDpi {
    inner: FilterProc,
}
impl Default for SpoofDpi {
    fn default() -> Self {
        Self {
            inner: FilterProc::new(
                "spoofdpi",
                "spoofdpi",
                &[],
                vec![Platform::Macos, Platform::Linux, Platform::Windows],
            ),
        }
    }
}
impl Engine for SpoofDpi {
    fn info(&self) -> EngineInfo {
        self.inner.info()
    }
    fn start(&self) -> Result<()> {
        self.inner.start()
    }
    fn stop(&self) -> Result<()> {
        self.inner.stop()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
}

pub struct ByeDpi {
    inner: FilterProc,
}
impl Default for ByeDpi {
    fn default() -> Self {
        Self {
            inner: FilterProc::new(
                "byedpi",
                "ciadpi",
                &["-d", "1"],
                vec![
                    Platform::Windows,
                    Platform::Macos,
                    Platform::Linux,
                    Platform::Android,
                ],
            ),
        }
    }
}
impl Engine for ByeDpi {
    fn info(&self) -> EngineInfo {
        self.inner.info()
    }
    fn start(&self) -> Result<()> {
        self.inner.start()
    }
    fn stop(&self) -> Result<()> {
        self.inner.stop()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
}
