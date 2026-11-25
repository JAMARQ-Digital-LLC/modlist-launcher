use std::path::PathBuf;

use modlist_core::scratch::SteamSource;
use modlist_core::steam::SteamRoot;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
use windows::core::w;

/// The Steam folder this run uses: the real one from the registry only in a
/// release build (or with `MODLIST_LAUNCHER_STEAM=live`), a named folder, or
/// none (see `modlist_core::scratch`).
pub fn locate() -> Option<SteamRoot> {
    let path = match &super::settings().as_ref().ok()?.steam {
        SteamSource::Registry => registry_path()?,
        SteamSource::Folder(p) => p.clone(),
        SteamSource::Off => return None,
    };
    path.is_dir().then(|| SteamRoot::new(path))
}

/// `HKCU\Software\Valve\Steam\SteamPath`. Read-only; also used to refuse
/// the real folder in dev and scratch runs.
pub fn registry_path() -> Option<PathBuf> {
    let mut buf = vec![0u16; 1024];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer size in bytes is passed and updated by the call.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\Valve\Steam"),
            w!("SteamPath"),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    if status.is_err() {
        return None;
    }
    Some(PathBuf::from(super::from_wide(&buf).replace('/', "\\")))
}

pub fn is_running() -> bool {
    !super::processes::running(|n| n.eq_ignore_ascii_case("steam.exe")).is_empty()
}
