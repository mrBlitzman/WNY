//! macOS Filter uses SpoofDPI / zapret tpws (see wny-core engines).
//! L0 uses pf anchor `com.wny`. The desktop shell never launches SpoofDPI's TUI.

#[cfg(target_os = "macos")]
pub const FILTER_NOTE: &str = "Filter slot: spoofdpi then tpws. Road: utun + hev-socks5-tunnel.";
