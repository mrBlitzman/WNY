//! WNY core: local settings, allowlist, orchestrator. No network protocol.

pub mod allowlist;
pub mod engine;
pub mod engines;
pub mod error;
pub mod i18n;
pub mod integrity;
pub mod mode;
pub mod orchestrator;
pub mod plaintext;
pub mod platform;
pub mod policy;
pub mod probe;
pub mod settings;

pub use allowlist::{AllowContext, Allowlist, Dest};
pub use engine::{Engine, EngineInfo, Health};
pub use error::{Error, Result};
pub use mode::{Language, Mode, Slot, SlotChoice};
pub use orchestrator::{Orchestrator, UiSnapshot};
pub use plaintext::{classify_payload, is_plaintext, Flow, PayloadKind, Transport};
pub use platform::Platform;
pub use policy::Policy;
pub use settings::Settings;
