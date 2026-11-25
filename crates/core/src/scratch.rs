//! Dev identity and scratch mode (agentic-instructions
//! `docs/desktop-app-standard.md`, section 5).
//!
//! A dev build (debug, or any build not made by `scripts/package.ps1` in CI)
//! keeps its data in its own folder and never sees the real Steam unless
//! `MODLIST_LAUNCHER_STEAM=live` asks for it by name. With
//! `MODLIST_LAUNCHER_SCRATCH=1` the launcher refuses, rather than warns about,
//! anything real: the release or dev data folder, `%APPDATA%`, the real Steam
//! folder, and starting or writing anything outside the scratch folders.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::paths::{normalize, strip_verbatim};

pub const RELEASE_FOLDER: &str = "Modlist Launcher";
pub const DEV_FOLDER: &str = "Modlist Launcher Dev";

pub const SCRATCH: &str = "MODLIST_LAUNCHER_SCRATCH";
pub const DATA_DIR: &str = "MODLIST_LAUNCHER_DATA_DIR";
/// The older name for [`DATA_DIR`], still read when the new one is unset.
pub const DATA_DIR_ALIAS: &str = "MODLIST_LAUNCHER_DATA";
pub const STEAM: &str = "MODLIST_LAUNCHER_STEAM";
pub const NETWORK: &str = "MODLIST_LAUNCHER_NETWORK";
pub const SECRETS: &str = "MODLIST_LAUNCHER_SECRETS";

/// What the process environment says, read once.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub scratch: bool,
    pub data_dir: Option<PathBuf>,
    pub steam: Option<String>,
    pub network: Option<String>,
    pub secrets: Option<String>,
    pub local_app_data: Option<PathBuf>,
    pub app_data: Option<PathBuf>,
    pub temp: PathBuf,
}

impl Env {
    pub fn from_vars(get: impl Fn(&str) -> Option<OsString>, temp: PathBuf) -> Self {
        let get = |name: &str| get(name).filter(|v| !v.is_empty());
        let text = |name: &str| get(name).map(|v| v.to_string_lossy().trim().to_string());
        Self {
            scratch: text(SCRATCH).as_deref() == Some("1"),
            data_dir: get(DATA_DIR).or_else(|| get(DATA_DIR_ALIAS)).map(PathBuf::from),
            steam: text(STEAM),
            network: text(NETWORK),
            secrets: text(SECRETS),
            local_app_data: get("LOCALAPPDATA").map(PathBuf::from),
            app_data: get("APPDATA").map(PathBuf::from),
            temp,
        }
    }
}

/// Where Steam's files come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamSource {
    /// The real Steam, from `HKCU\Software\Valve\Steam\SteamPath`.
    Registry,
    /// A folder named by `MODLIST_LAUNCHER_STEAM` (a fake Steam in tests).
    Folder(PathBuf),
    /// No Steam: a dev or scratch run that did not name one.
    Off,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub scratch: bool,
    pub dev: bool,
    pub data_dir: PathBuf,
    pub steam: SteamSource,
    /// Under scratch, the only places a program may be started from or a
    /// shortcut written to.
    pub scratch_roots: Vec<PathBuf>,
}

/// Resolves the settings. `real_steam` is the registry's SteamPath, read
/// only to refuse it.
pub fn resolve(env: &Env, dev: bool, real_steam: Option<&Path>) -> Result<Settings, String> {
    match env.network.as_deref() {
        None | Some("offline") => {}
        Some("live") if env.scratch => {
            return Err(format!("{SCRATCH}=1 refuses {NETWORK}=live."));
        }
        Some("live") => {}
        Some(other) => return Err(format!("{NETWORK} must be live or offline, not \"{other}\".")),
    }
    match env.secrets.as_deref() {
        None | Some("memory") => {}
        Some("keychain") if env.scratch => {
            return Err(format!("{SCRATCH}=1 refuses {SECRETS}=keychain (Credential Manager)."));
        }
        Some("keychain") => {}
        Some(other) => return Err(format!("{SECRETS} must be keychain or memory, not \"{other}\".")),
    }

    let base = env.local_app_data.clone().unwrap_or_else(|| env.temp.clone());
    let release_dir = base.join(RELEASE_FOLDER);
    let dev_dir = base.join(DEV_FOLDER);

    let data_dir = match &env.data_dir {
        Some(dir) if env.scratch => {
            if !dir.is_absolute() {
                return Err(format!("{SCRATCH}=1 needs an absolute {DATA_DIR}, not {}.", dir.display()));
            }
            let mut real: Vec<&Path> = vec![&release_dir, &dev_dir];
            if let Some(app_data) = &env.app_data {
                real.push(app_data);
            }
            if let Some(steam) = real_steam {
                real.push(steam);
            }
            if let Some(hit) = real.iter().find(|r| is_within(dir, r)) {
                return Err(format!(
                    "{SCRATCH}=1 refuses the data folder {}: it is inside {}.",
                    dir.display(),
                    hit.display()
                ));
            }
            dir.clone()
        }
        Some(dir) => dir.clone(),
        None if env.scratch => return Err(format!("{SCRATCH}=1 needs {DATA_DIR} set to a scratch folder.")),
        None if dev => dev_dir,
        None => release_dir,
    };

    let refuse_real = |path: &Path| -> Result<(), String> {
        match real_steam {
            Some(real) if is_within(path, real) => Err(format!(
                "{STEAM} names the real Steam folder {}. Use a fake Steam folder, or {STEAM}=live outside scratch.",
                real.display()
            )),
            _ => Ok(()),
        }
    };
    let steam = match env.steam.as_deref() {
        Some("live") if env.scratch => return Err(format!("{SCRATCH}=1 refuses {STEAM}=live.")),
        Some("live") => SteamSource::Registry,
        Some(folder) => {
            let folder = PathBuf::from(folder);
            if env.scratch || dev {
                refuse_real(&folder)?;
            }
            SteamSource::Folder(folder)
        }
        None if env.scratch || dev => SteamSource::Off,
        None => SteamSource::Registry,
    };

    let scratch_roots = if env.scratch { vec![data_dir.clone(), env.temp.clone()] } else { Vec::new() };
    Ok(Settings { scratch: env.scratch, dev, data_dir, steam, scratch_roots })
}

impl Settings {
    /// Under scratch, refuses `path` unless it is inside the scratch data
    /// folder or the temp folder. `what` reads like "start" or "write".
    pub fn check_scratch_path(&self, what: &str, path: &Path) -> Result<(), String> {
        if !self.scratch || self.scratch_roots.iter().any(|r| is_within(path, r)) {
            return Ok(());
        }
        Err(format!(
            "{SCRATCH}=1 refuses to {what} {}: only the scratch data folder and the temp folder are allowed.",
            path.display()
        ))
    }
}

/// True when `path` is `root` or inside it. Both are resolved through the
/// file system where they exist, so an 8.3 short name or a junction does not
/// slip past.
pub fn is_within(path: &Path, root: &Path) -> bool {
    let p = normalize(&resolved(path));
    let r = normalize(&resolved(root));
    p == r || p.strip_prefix(&r).is_some_and(|rest| rest.starts_with('\\') || r.ends_with('\\'))
}

/// `path` with its longest existing ancestor canonicalized.
fn resolved(path: &Path) -> PathBuf {
    let mut rest = Vec::new();
    let mut cur = path;
    loop {
        if let Ok(c) = std::fs::canonicalize(cur) {
            let mut out = strip_verbatim(c);
            for part in rest.iter().rev() {
                out.push(part);
            }
            return out;
        }
        match (cur.parent(), cur.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_os_string());
                cur = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(tmp: &Path) -> Env {
        Env {
            local_app_data: Some(tmp.join("Local")),
            app_data: Some(tmp.join("Roaming")),
            temp: tmp.join("Temp"),
            ..Env::default()
        }
    }

    fn scratch(tmp: &Path) -> Env {
        Env { scratch: true, data_dir: Some(tmp.join("Temp").join("scratch")), ..env(tmp) }
    }

    #[test]
    fn data_dir_alias_and_new_name() {
        let vars = |pairs: &'static [(&'static str, &'static str)]| {
            move |n: &str| pairs.iter().find(|(k, _)| *k == n).map(|(_, v)| OsString::from(v))
        };
        let e = Env::from_vars(vars(&[(DATA_DIR_ALIAS, r"C:\old")]), PathBuf::from(r"C:\t"));
        assert_eq!(e.data_dir, Some(PathBuf::from(r"C:\old")));
        let e = Env::from_vars(vars(&[(DATA_DIR_ALIAS, r"C:\old"), (DATA_DIR, r"C:\new")]), PathBuf::from(r"C:\t"));
        assert_eq!(e.data_dir, Some(PathBuf::from(r"C:\new")));
        let e = Env::from_vars(vars(&[(SCRATCH, "1"), (DATA_DIR, "")]), PathBuf::from(r"C:\t"));
        assert!(e.scratch);
        assert_eq!(e.data_dir, None);
    }

    #[test]
    fn release_and_dev_builds_use_their_own_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let e = env(tmp.path());
        let release = resolve(&e, false, None).unwrap();
        assert_eq!(release.data_dir, tmp.path().join("Local").join(RELEASE_FOLDER));
        assert_eq!(release.steam, SteamSource::Registry);
        let dev = resolve(&e, true, None).unwrap();
        assert_eq!(dev.data_dir, tmp.path().join("Local").join(DEV_FOLDER));
        assert_eq!(dev.steam, SteamSource::Off);
    }

    #[test]
    fn dev_build_needs_live_by_name_for_the_real_steam() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("Steam");
        std::fs::create_dir_all(&real).unwrap();
        let mut e = env(tmp.path());
        e.steam = Some(real.to_string_lossy().into_owned());
        assert!(resolve(&e, true, Some(&real)).is_err());
        e.steam = Some(real.join("userdata").to_string_lossy().into_owned());
        assert!(resolve(&e, true, Some(&real)).is_err());
        e.steam = Some("live".into());
        assert_eq!(resolve(&e, true, Some(&real)).unwrap().steam, SteamSource::Registry);
        let fake = tmp.path().join("FakeSteam");
        e.steam = Some(fake.to_string_lossy().into_owned());
        assert_eq!(resolve(&e, true, Some(&real)).unwrap().steam, SteamSource::Folder(fake));
    }

    #[test]
    fn scratch_requires_a_data_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let e = Env { scratch: true, ..env(tmp.path()) };
        assert!(resolve(&e, false, None).unwrap_err().contains(DATA_DIR));
        let e = Env { data_dir: Some(PathBuf::from("relative")), ..scratch(tmp.path()) };
        assert!(resolve(&e, false, None).is_err());
    }

    #[test]
    fn scratch_refuses_the_real_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let real_steam = tmp.path().join("Steam");
        let local = tmp.path().join("Local");
        for bad in [
            local.join(RELEASE_FOLDER),
            local.join(RELEASE_FOLDER).join("sub"),
            local.join(DEV_FOLDER),
            tmp.path().join("Roaming").join("x"),
            real_steam.join("userdata"),
        ] {
            let e = Env { data_dir: Some(bad.clone()), ..scratch(tmp.path()) };
            assert!(resolve(&e, false, Some(&real_steam)).is_err(), "{} was allowed", bad.display());
            assert!(resolve(&e, true, Some(&real_steam)).is_err(), "{} was allowed", bad.display());
        }
        // A sibling whose name only starts the same is not inside.
        let e = Env { data_dir: Some(local.join("Modlist Launcher Scratch")), ..scratch(tmp.path()) };
        assert!(resolve(&e, false, Some(&real_steam)).is_ok());
    }

    #[test]
    fn scratch_refuses_real_steam_network_and_secrets() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("Steam");
        std::fs::create_dir_all(&real).unwrap();
        let ok = resolve(&scratch(tmp.path()), false, Some(&real)).unwrap();
        assert_eq!(ok.steam, SteamSource::Off);
        for (steam, network, secrets) in [
            (Some("live"), None, None),
            (Some(real.to_string_lossy().as_ref()), None, None),
            (None, Some("live"), None),
            (None, Some("sometimes"), None),
            (None, None, Some("keychain")),
        ] {
            let e = Env {
                steam: steam.map(str::to_string),
                network: network.map(str::to_string),
                secrets: secrets.map(str::to_string),
                ..scratch(tmp.path())
            };
            assert!(resolve(&e, false, Some(&real)).is_err(), "{steam:?} {network:?} {secrets:?} was allowed");
        }
        let fake = tmp.path().join("Temp").join("fake-steam");
        let e = Env {
            steam: Some(fake.to_string_lossy().into_owned()),
            network: Some("offline".into()),
            secrets: Some("memory".into()),
            ..scratch(tmp.path())
        };
        assert_eq!(resolve(&e, false, Some(&real)).unwrap().steam, SteamSource::Folder(fake));
    }

    #[test]
    fn scratch_only_starts_and_writes_inside_its_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let s = resolve(&scratch(tmp.path()), false, None).unwrap();
        assert!(s.check_scratch_path("start", &tmp.path().join("Temp").join("x").join("ModOrganizer.exe")).is_ok());
        assert!(s.check_scratch_path("start", &tmp.path().join("Temp").join("scratch").join("a.lnk")).is_ok());
        assert!(
            s.check_scratch_path("start", Path::new(r"C:\Games\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe")).is_err()
        );
        assert!(s.check_scratch_path("write", &tmp.path().join("Desktop").join("List.lnk")).is_err());
        let not_scratch = resolve(&env(tmp.path()), false, None).unwrap();
        assert!(not_scratch.check_scratch_path("start", Path::new(r"C:\Games\x.exe")).is_ok());
    }

    #[test]
    fn is_within_matches_whole_components() {
        assert!(is_within(Path::new(r"C:\A\B"), Path::new(r"c:\a")));
        assert!(is_within(Path::new(r"C:\A"), Path::new(r"C:\A\")));
        assert!(!is_within(Path::new(r"C:\AB"), Path::new(r"C:\A")));
        assert!(is_within(Path::new(r"C:\A"), Path::new(r"C:\")));
    }
}
