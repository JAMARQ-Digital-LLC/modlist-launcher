//! The data folder's access list: only the signed-in user, applied when the
//! folder is created and again at every launch (the Windows form of 0700).

use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::Authorization::{SE_FILE_OBJECT, SetNamedSecurityInfoW};
use windows::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACL, ACL_REVISION, AddAccessAllowedAceEx, CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION,
    GetLengthSid, GetTokenInformation, InitializeAcl, OBJECT_INHERIT_ACE, PROTECTED_DACL_SECURITY_INFORMATION, PSID,
    TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::HSTRING;

/// The current user's TOKEN_USER, kept in a buffer that owns the SID.
fn current_user() -> windows::core::Result<Vec<u8>> {
    let mut token = HANDLE::default();
    // SAFETY: query-only token of this process, closed below.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)? };
    let mut len = 0u32;
    // SAFETY: the first call only reports the size needed.
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut len) };
    let mut buf = vec![0u8; len as usize];
    // SAFETY: the buffer has the size the first call reported.
    let got = unsafe { GetTokenInformation(token, TokenUser, Some(buf.as_mut_ptr().cast()), len, &mut len) };
    // SAFETY: the handle came from OpenProcessToken.
    let _ = unsafe { CloseHandle(token) };
    got.map(|()| buf)
}

fn sid_of(user: &[u8]) -> PSID {
    // SAFETY: `user` holds a TOKEN_USER filled in by GetTokenInformation.
    unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid }
}

/// Replaces the folder's access list with one entry: full control for the
/// current user, inherited by everything inside, nothing inherited from the
/// parent. Creates the folder first when it is missing.
pub fn restrict_to_current_user(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    let fail = |e: windows::core::Error| format!("could not restrict {}: {}", dir.display(), e.message());
    let user = current_user().map_err(fail)?;
    let sid = sid_of(&user);
    // SAFETY: `sid` points into `user`, which outlives every use below.
    let sid_len = unsafe { GetLengthSid(sid) } as usize;
    let size = std::mem::size_of::<ACL>() + std::mem::size_of::<ACCESS_ALLOWED_ACE>() + sid_len;
    // u32 storage keeps the ACL aligned.
    let mut storage = vec![0u32; size.div_ceil(4)];
    let acl = storage.as_mut_ptr().cast::<ACL>();
    // SAFETY: `acl` points to `size` writable bytes.
    unsafe {
        InitializeAcl(acl, size as u32, ACL_REVISION).map_err(fail)?;
        AddAccessAllowedAceEx(acl, ACL_REVISION, OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE, FILE_ALL_ACCESS.0, sid)
            .map_err(fail)?;
    }
    // SAFETY: a valid path string and the ACL built above.
    let status = unsafe {
        SetNamedSecurityInfoW(
            &HSTRING::from(dir.as_os_str()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(acl),
            None,
        )
    };
    status.ok().map_err(fail)
}

/// Reads the folder's access list back: true when it is protected from
/// inheritance and grants access to exactly one account, the current user.
#[cfg(test)]
pub fn only_current_user(dir: &Path) -> Result<bool, String> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::GetNamedSecurityInfoW;
    use windows::Win32::Security::{
        ACE_HEADER, EqualSid, GetAce, GetSecurityDescriptorControl, PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
    };
    const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;

    let fail =
        |e: windows::core::Error| format!("could not read the access list of {}: {}", dir.display(), e.message());
    let user = current_user().map_err(fail)?;
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: out-pointers to locals; `sd` is freed below.
    let status = unsafe {
        GetNamedSecurityInfoW(
            &HSTRING::from(dir.as_os_str()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut dacl),
            None,
            &mut sd,
        )
    };
    status.ok().map_err(fail)?;
    // SAFETY: `sd` and `dacl` come from GetNamedSecurityInfoW and stay valid
    // until LocalFree.
    let result = (|| unsafe {
        let mut control = 0u16;
        let mut revision = 0u32;
        GetSecurityDescriptorControl(sd, &mut control, &mut revision).map_err(fail)?;
        let protected = control & SE_DACL_PROTECTED.0 != 0;
        let count = if dacl.is_null() { 0 } else { u32::from((*dacl).AceCount) };
        let mut only_user = count == 1;
        for i in 0..count {
            let mut ace = std::ptr::null_mut();
            GetAce(dacl, i, &mut ace).map_err(fail)?;
            let header = &*ace.cast::<ACE_HEADER>();
            let allowed = ace.cast::<ACCESS_ALLOWED_ACE>();
            let sid = PSID(std::ptr::addr_of_mut!((*allowed).SidStart).cast());
            if header.AceType != ACCESS_ALLOWED_ACE_TYPE || EqualSid(sid, sid_of(&user)).is_err() {
                only_user = false;
            }
        }
        Ok(protected && only_user)
    })();
    // SAFETY: allocated by GetNamedSecurityInfoW.
    unsafe {
        let _ = LocalFree(Some(HLOCAL(sd.0)));
    }
    result
}
