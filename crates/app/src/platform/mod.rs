//! Windows-specific operations. Everything that touches the OS lives here so
//! the rest of the app stays testable.

pub mod acl;
pub mod clipboard;
pub mod dialogs;
pub mod icons;
pub mod launch;
pub mod other_windows;
pub mod processes;
pub mod shell;
pub mod steam;

use std::path::PathBuf;
use std::sync::OnceLock;

use modlist_core::scratch::{self, Settings};

/// True for a debug build, or any build `scripts/package.ps1` did not make
/// for a release: worktree and local builds are dev builds.
pub fn dev_build() -> bool {
    cfg!(debug_assertions) || option_env!("MODLIST_LAUNCHER_RELEASE_BUILD") != Some("1")
}

/// The dev identity, scratch mode, data folder and Steam source, resolved
/// once from the environment. `main` refuses to start when this is an error.
/// Tests always run in scratch, with a per-process data folder under temp.
pub fn settings() -> &'static Result<Settings, String> {
    static SETTINGS: OnceLock<Result<Settings, String>> = OnceLock::new();
    SETTINGS.get_or_init(|| {
        let mut env = scratch::Env::from_vars(|n| std::env::var_os(n), std::env::temp_dir());
        if cfg!(test) {
            env.scratch = true;
            env.data_dir.get_or_insert_with(|| {
                std::env::temp_dir().join(format!("modlist-launcher-test-{}", std::process::id()))
            });
        }
        scratch::resolve(&env, dev_build(), steam::registry_path().as_deref())
    })
}

fn checked() -> &'static Settings {
    settings().as_ref().expect("settings are checked when the launcher starts")
}

/// The launcher-owned data folder: `%LOCALAPPDATA%\Modlist Launcher` for a
/// release build, `%LOCALAPPDATA%\Modlist Launcher Dev` for a dev build, or
/// `MODLIST_LAUNCHER_DATA_DIR` (old name `MODLIST_LAUNCHER_DATA`).
pub fn data_dir() -> PathBuf {
    checked().data_dir.clone()
}

/// Under scratch, refuses a path outside the scratch data folder and temp.
pub fn check_scratch_path(what: &str, path: &std::path::Path) -> Result<(), String> {
    match settings() {
        Ok(s) => s.check_scratch_path(what, path),
        Err(e) => Err(e.clone()),
    }
}

pub fn shift_held() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
    // SAFETY: plain query of the async key state.
    unsafe { GetAsyncKeyState(i32::from(VK_SHIFT.0)) as u16 & 0x8000 != 0 }
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

#[cfg(test)]
mod tests;
