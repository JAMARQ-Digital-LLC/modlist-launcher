use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};

use modlist_core::launch::LaunchSpec;

/// Starts the process described by `spec` with its own working directory.
/// The child is detached from the launcher's lifetime.
/// Under scratch it starts nothing outside the scratch folders, so a test or
/// smoke run can never start a real MO2 or game.
pub fn spawn(spec: &LaunchSpec) -> std::io::Result<Child> {
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    super::check_scratch_path("start", &spec.program)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;
    if !spec.working_directory.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("working directory {} does not exist", spec.working_directory.display()),
        ));
    }
    Command::new(&spec.program)
        .args(&spec.args)
        .current_dir(&spec.working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NEW_PROCESS_GROUP)
        .spawn()
}

/// Watches a freshly started MO2 for a few seconds. MO2 exits right away with
/// a non-zero code when it refuses to start (for example another instance).
pub fn early_exit(child: &mut Child, window: std::time::Duration) -> Option<std::process::ExitStatus> {
    let deadline = std::time::Instant::now() + window;
    while std::time::Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(150)),
            Err(_) => return None,
        }
    }
    None
}
