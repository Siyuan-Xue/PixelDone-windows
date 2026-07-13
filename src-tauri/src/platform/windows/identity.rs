use std::{
    mem::ManuallyDrop,
    path::{Path, PathBuf},
};

use windows::{
    Win32::{
        Foundation::{PROPERTYKEY, RPC_E_CHANGED_MODE},
        System::{
            Com::StructuredStorage::{
                PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0, PropVariantClear,
            },
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoTaskMemAlloc, CoTaskMemFree, CoUninitialize, IPersistFile, STGM_READWRITE,
            },
            Variant::{VT_CLSID, VT_LPWSTR},
        },
        UI::Shell::{
            FOLDERID_Programs, IShellLinkW, KF_FLAG_DEFAULT, PropertiesSystem::IPropertyStore,
            SHGetKnownFolderPath, SHStrDupW, SetCurrentProcessExplicitAppUserModelID, ShellLink,
        },
    },
    core::{GUID, Interface, PCWSTR},
};

use crate::{domain::AppError, platform::windows::notification::APP_USER_MODEL_ID};

pub const TOAST_ACTIVATOR_STUB_CLSID: GUID =
    GUID::from_u128(0x8c0e9d6b_47af_4b53_9c1e_1c477842b2da);

const APP_USER_MODEL_PROPERTY: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
    pid: 5,
};
const TOAST_ACTIVATOR_PROPERTY: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3),
    pid: 26,
};

pub fn ensure_notification_identity(executable: &Path) -> Result<PathBuf, AppError> {
    let app_id = wide(APP_USER_MODEL_ID);
    // SAFETY: app_id is a valid null-terminated UTF-16 string for this synchronous call.
    unsafe { SetCurrentProcessExplicitAppUserModelID(PCWSTR(app_id.as_ptr())) }
        .map_err(platform_error)?;

    if is_direct_development_binary(executable) {
        return Err(AppError::Platform(
            "Windows notifications require an installed PixelDone build".to_owned(),
        ));
    }

    let initialized = match unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok() {
        Ok(()) => true,
        Err(error) if error.code() == RPC_E_CHANGED_MODE => false,
        Err(error) => return Err(platform_error(error)),
    };
    let _guard = ComGuard(initialized);
    create_start_menu_shortcut(executable)
}

fn create_start_menu_shortcut(executable: &Path) -> Result<PathBuf, AppError> {
    // SAFETY: the known-folder API returns a CoTaskMem-allocated UTF-16 string.
    let programs_raw = unsafe { SHGetKnownFolderPath(&FOLDERID_Programs, KF_FLAG_DEFAULT, None) }
        .map_err(platform_error)?;
    let programs =
        unsafe { programs_raw.to_string() }.map_err(|error| AppError::Platform(error.to_string()));
    unsafe { CoTaskMemFree(Some(programs_raw.0.cast())) };
    let programs = programs?;
    let shortcut_path = PathBuf::from(programs).join("PixelDone.lnk");

    // SAFETY: COM is initialized on this thread and ShellLink is an in-process COM class.
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(platform_error)?;
    let persist: IPersistFile = link.cast().map_err(platform_error)?;
    let shortcut_wide = wide(&shortcut_path.display().to_string());
    let loaded_existing = shortcut_path.exists()
        && unsafe { persist.Load(PCWSTR(shortcut_wide.as_ptr()), STGM_READWRITE) }.is_ok();
    let preserve_target = loaded_existing
        && existing_shortcut_target(&link)
            .is_some_and(|target| executable_paths_match(&target, executable));
    let executable_wide = wide(&executable.display().to_string());
    let working_directory = executable.parent().unwrap_or_else(|| Path::new(""));
    let working_directory_wide = wide(&working_directory.display().to_string());
    let description = wide("PixelDone for Windows");
    unsafe {
        if !preserve_target {
            link.SetPath(PCWSTR(executable_wide.as_ptr()))
                .map_err(platform_error)?;
            link.SetWorkingDirectory(PCWSTR(working_directory_wide.as_ptr()))
                .map_err(platform_error)?;
        }
        link.SetDescription(PCWSTR(description.as_ptr()))
            .map_err(platform_error)?;
        link.SetIconLocation(PCWSTR(executable_wide.as_ptr()), 0)
            .map_err(platform_error)?;
    }

    let store: IPropertyStore = link.cast().map_err(platform_error)?;
    set_string_property(&store, &APP_USER_MODEL_PROPERTY, APP_USER_MODEL_ID)?;
    set_guid_property(
        &store,
        &TOAST_ACTIVATOR_PROPERTY,
        TOAST_ACTIVATOR_STUB_CLSID,
    )?;
    unsafe { store.Commit() }.map_err(platform_error)?;

    unsafe { persist.Save(PCWSTR(shortcut_wide.as_ptr()), true) }.map_err(platform_error)?;
    Ok(shortcut_path)
}

fn existing_shortcut_target(link: &IShellLinkW) -> Option<PathBuf> {
    let mut value = vec![0_u16; 32_768];
    unsafe { link.GetPath(&mut value, std::ptr::null_mut(), 0) }.ok()?;
    let length = value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len());
    (length > 0).then(|| PathBuf::from(String::from_utf16_lossy(&value[..length])))
}

fn executable_paths_match(existing: &Path, current: &Path) -> bool {
    if !existing.is_file() || !current.is_file() {
        return false;
    }
    let existing = existing
        .canonicalize()
        .unwrap_or_else(|_| existing.to_path_buf());
    let current = current
        .canonicalize()
        .unwrap_or_else(|_| current.to_path_buf());
    normalize_windows_path(&existing) == normalize_windows_path(&current)
}

fn normalize_windows_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_start_matches(r"\\?\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn set_string_property(
    store: &IPropertyStore,
    key: &PROPERTYKEY,
    value: &str,
) -> Result<(), AppError> {
    let value = wide(value);
    let duplicated = unsafe { SHStrDupW(PCWSTR(value.as_ptr())) }.map_err(platform_error)?;
    let mut variant = prop_variant(
        VT_LPWSTR,
        PROPVARIANT_0_0_0 {
            pwszVal: duplicated,
        },
    );
    let result = unsafe { store.SetValue(key, &variant) }.map_err(platform_error);
    unsafe { PropVariantClear(&mut variant) }.map_err(platform_error)?;
    result
}

fn set_guid_property(
    store: &IPropertyStore,
    key: &PROPERTYKEY,
    value: GUID,
) -> Result<(), AppError> {
    let pointer = unsafe { CoTaskMemAlloc(std::mem::size_of::<GUID>()) }.cast::<GUID>();
    if pointer.is_null() {
        return Err(AppError::Platform(
            "Could not allocate notification identity property".to_owned(),
        ));
    }
    unsafe { pointer.write(value) };
    let mut variant = prop_variant(VT_CLSID, PROPVARIANT_0_0_0 { puuid: pointer });
    let result = unsafe { store.SetValue(key, &variant) }.map_err(platform_error);
    unsafe { PropVariantClear(&mut variant) }.map_err(platform_error)?;
    result
}

fn prop_variant(
    kind: windows::Win32::System::Variant::VARENUM,
    value: PROPVARIANT_0_0_0,
) -> PROPVARIANT {
    PROPVARIANT {
        Anonymous: PROPVARIANT_0 {
            Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
                vt: kind,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: value,
            }),
        },
    }
}

fn is_direct_development_binary(executable: &Path) -> bool {
    let normalized = executable
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase();
    normalized.contains("\\src-tauri\\target\\debug\\")
        || normalized.contains("\\src-tauri\\target\\release\\")
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn platform_error(error: windows::core::Error) -> AppError {
    AppError::Platform(error.to_string())
}

struct ComGuard(bool);

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_binaries_are_detected_before_shortcut_updates() {
        assert!(is_direct_development_binary(Path::new(
            r"C:\repo\src-tauri\target\debug\PixelDone.exe"
        )));
        assert!(!is_direct_development_binary(Path::new(
            r"C:\Users\Miles\AppData\Local\PixelDone\PixelDone.exe"
        )));
    }

    #[test]
    fn shortcut_targets_are_preserved_only_for_the_current_executable() {
        let root = std::env::temp_dir();
        let current = root.join(format!("pixeldone-current-{}.exe", std::process::id()));
        let stale = root.join(format!("pixeldone-stale-{}.exe", std::process::id()));
        std::fs::write(&current, b"current").unwrap();
        std::fs::write(&stale, b"stale").unwrap();

        assert!(executable_paths_match(&current, &current));
        assert!(!executable_paths_match(&stale, &current));
        assert!(!executable_paths_match(
            &root.join("pixeldone-missing.exe"),
            &current
        ));

        let _ = std::fs::remove_file(current);
        let _ = std::fs::remove_file(stale);
    }
}
