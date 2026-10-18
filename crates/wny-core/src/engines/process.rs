use crate::error::{Error, Result};
use crate::integrity::{verify_binary, EnginesLock};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

pub type ChildCell = Arc<Mutex<Option<Child>>>;

pub fn engine_bin_dir() -> PathBuf {
    if let Ok(p) = std::env::var("WNY_ENGINES_DIR") {
        return PathBuf::from(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("engines");
            if bundled.is_dir() {
                return bundled;
            }
        }
    }
    PathBuf::from("engines/bin")
}

pub fn binary_path(name: &str) -> PathBuf {
    let dir = engine_bin_dir();
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    dir.join(exe)
}

pub fn binary_exists(name: &str) -> bool {
    binary_path(name).is_file()
}

pub fn spawn_headless(engine_id: &str, bin: &str, args: &[String]) -> Result<Child> {
    let path = binary_path(bin);
    if !path.is_file() {
        return Err(Error::Engine {
            id: engine_id.into(),
            detail: format!("missing binary {}", path.display()),
        });
    }
    if let Ok(lock) = EnginesLock::bundled() {
        verify_binary(&lock, engine_id, &path)?;
    }
    spawn_path(engine_id, &path, args)
}

pub fn spawn_path(engine_id: &str, path: &Path, args: &[String]) -> Result<Child> {
    // Child must never show a window. Windows CREATE_NO_WINDOW = 0x08000000.
    let mut cmd = Command::new(path);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map_err(|e| Error::Engine {
        id: engine_id.into(),
        detail: e.to_string(),
    })
}

pub fn stop_child(cell: &ChildCell) -> Result<()> {
    let mut g = cell.lock();
    if let Some(mut c) = g.take() {
        let _ = c.kill();
        let _ = c.wait();
    }
    Ok(())
}

pub fn child_up(cell: &ChildCell) -> bool {
    let mut g = cell.lock();
    match g.as_mut() {
        None => false,
        Some(c) => match c.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => {
                *g = None;
                false
            }
            Err(_) => false,
        },
    }
}
