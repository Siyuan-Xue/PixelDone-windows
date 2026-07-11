use std::{ffi::c_void, ptr};

use windows::{
    Win32::Security::Credentials::{
        CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree,
        CredReadW, CredWriteW,
    },
    core::{PCWSTR, PWSTR},
};

use crate::{domain::AppError, infrastructure::auth::AuthSession};

const TARGET: &str = "com.milesxue.pixeldone.windows/supabase-session";

#[derive(Clone)]
pub struct CredentialStore {
    target: String,
}

impl Default for CredentialStore {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let target =
            std::env::var("PIXELDONE_CREDENTIAL_TARGET").unwrap_or_else(|_| TARGET.to_owned());
        #[cfg(not(debug_assertions))]
        let target = TARGET.to_owned();
        Self { target }
    }
}

impl CredentialStore {
    pub fn load(&self) -> Result<Option<AuthSession>, AppError> {
        let target = wide(&self.target);
        let mut pointer: *mut CREDENTIALW = ptr::null_mut();
        // SAFETY: target is a null-terminated UTF-16 string and Windows owns the returned buffer.
        if unsafe {
            CredReadW(
                PCWSTR(target.as_ptr()),
                CRED_TYPE_GENERIC,
                None,
                &mut pointer,
            )
        }
        .is_err()
        {
            return Ok(None);
        }
        if pointer.is_null() {
            return Ok(None);
        }
        // SAFETY: CredReadW returned a valid CREDENTIALW and blob for the duration of this block.
        let bytes = unsafe {
            let credential = &*pointer;
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
            .to_vec()
        };
        // SAFETY: the pointer was allocated by CredReadW and must be released by CredFree.
        unsafe { CredFree(pointer.cast::<c_void>()) };
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| AppError::Platform(format!("Credential Manager 数据无效：{error}")))
    }

    pub fn save(&self, session: &AuthSession) -> Result<(), AppError> {
        let mut target = wide(&self.target);
        let mut username = wide(session.email.as_deref().unwrap_or("PixelDone"));
        let mut blob =
            serde_json::to_vec(session).map_err(|error| AppError::Platform(error.to_string()))?;
        let credential = CREDENTIALW {
            Type: CRED_TYPE_GENERIC,
            TargetName: PWSTR(target.as_mut_ptr()),
            CredentialBlobSize: u32::try_from(blob.len())
                .map_err(|_| AppError::Platform("Credential blob 过大".to_owned()))?,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            UserName: PWSTR(username.as_mut_ptr()),
            ..Default::default()
        };
        // SAFETY: every pointer in CREDENTIALW remains valid for the synchronous CredWriteW call.
        unsafe { CredWriteW(&credential, 0) }.map_err(|error| AppError::Platform(error.to_string()))
    }

    pub fn clear(&self) -> Result<(), AppError> {
        let target = wide(&self.target);
        // Missing credentials are already the desired state.
        match unsafe { CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, None) } {
            Ok(()) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
