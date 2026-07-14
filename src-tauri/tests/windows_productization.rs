use std::fs;

use sha2::{Digest, Sha384};

#[test]
fn formal_config_uses_professional_identity_and_protocol() {
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("tauri.conf.json").unwrap()).unwrap();
    assert_eq!(config["productName"], "PixelDone");
    assert_eq!(config["version"], "3.2.3");
    assert_eq!(config["mainBinaryName"], "PixelDone");
    assert_eq!(
        config["plugins"]["deep-link"]["desktop"]["schemes"][0],
        "pixeldone-reminder"
    );
}

#[test]
fn windows_platform_config_keeps_nsis_current_user_installation() {
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("tauri.windows.conf.json").unwrap()).unwrap();
    assert_eq!(config["bundle"]["targets"][0], "nsis");
    assert_eq!(
        config["bundle"]["windows"]["nsis"]["installMode"],
        "currentUser"
    );
}

#[test]
fn windows_icon_source_is_transparent_and_preserves_android_subject() {
    let icon = fs::read_to_string("../assets/pixeldone-icon.svg").unwrap();
    for token in [
        "#4B463E",
        "#FAF9F5",
        "#D97757",
        "#141413",
        "#6A9BCC",
        "#629987",
        "M34 28h40v52H34z",
        "M44 24h20v10H44z",
        "scale(1.42)",
    ] {
        assert!(icon.contains(token), "missing Android icon token: {token}");
    }
    assert!(!icon.contains("<rect width=\"108\" height=\"108\""));
}

#[test]
fn formal_release_matches_the_3_1_0_unsigned_publisher_policy() {
    let workflow = fs::read_to_string("../.github/workflows/release-windows.yml").unwrap();
    assert!(workflow.contains("TAURI_SIGNING_PRIVATE_KEY"));
    assert!(workflow.contains("args: --bundles nsis --target x86_64-pc-windows-msvc"));
    assert!(!workflow.contains("--config"));
    assert!(!workflow.contains("WINDOWS_CERTIFICATE_BASE64"));
    assert!(!workflow.contains("certificateThumbprint"));
}

#[test]
fn notification_identity_uses_stable_aumid_and_stub_clsid() {
    assert_eq!(
        pixeldone_windows_lib::platform::windows::notification::APP_USER_MODEL_ID,
        "com.milesxue.pixeldone.windows"
    );
    assert_eq!(
        pixeldone_windows_lib::platform::windows::identity::TOAST_ACTIVATOR_STUB_CLSID,
        windows::core::GUID::from_u128(0x8c0e9d6b_47af_4b53_9c1e_1c477842b2da)
    );
}

#[test]
fn runtime_preserves_only_the_current_installed_shortcut_target() {
    let source = fs::read_to_string("src/platform/windows/identity.rs").unwrap();
    assert!(source.contains("persist.Load"));
    assert!(source.contains("existing_shortcut_target"));
    assert!(source.contains("executable_paths_match"));
    assert!(source.contains("if !preserve_target"));
}

#[test]
fn sqlite_migrations_use_the_deployed_windows_line_endings() {
    let attributes = fs::read_to_string("../.gitattributes").unwrap();
    assert!(attributes.contains("src-tauri/migrations/*.sql text eol=crlf"));
    assert!(attributes.contains("src-tauri/migrations/0007_attachment_sync.sql text eol=lf"));
    for version in (1..=6).chain(std::iter::once(8)) {
        let prefix = format!("{version:04}_");
        let migration = fs::read_dir("migrations")
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .expect("migration file should exist");
        let bytes = fs::read(migration.path()).unwrap();
        assert!(
            bytes.windows(2).any(|window| window == b"\r\n"),
            "migration {version} must use CRLF to preserve SQLx checksums"
        );
        assert!(
            !bytes.windows(2).enumerate().any(
                |(index, _)| bytes[index] == b'\n' && (index == 0 || bytes[index - 1] != b'\r')
            ),
            "migration {version} contains a bare LF"
        );
    }

    let deployed_attachment_sync = fs::read("migrations/0007_attachment_sync.sql").unwrap();
    assert!(
        deployed_attachment_sync
            .windows(2)
            .any(|window| window == b"\n\n")
    );
    assert!(
        !deployed_attachment_sync
            .windows(2)
            .any(|window| window == b"\r\n"),
        "migration 7 must retain the LF checksum deployed by PixelDone 3.2.0"
    );
    assert_eq!(
        format!("{:X}", Sha384::digest(&deployed_attachment_sync)),
        "9606CFB487A71F9661010578B3E0D527A5A44B0B9D554650F1E6D524D086594560669F8548893FC6E85DECC900BE1CC6"
    );
}

#[test]
#[ignore = "requires an installed PixelDone notification identity"]
fn installed_notification_queue_reconcile_is_idempotent() {
    let installed = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .expect("LOCALAPPDATA should exist")
        .join("PixelDone")
        .join("PixelDone.exe");
    assert!(
        installed.is_file(),
        "installed PixelDone executable is required"
    );
    pixeldone_windows_lib::platform::windows::identity::ensure_notification_identity(&installed)
        .expect("installed notification identity should be valid");
    pixeldone_windows_lib::platform::windows::notification::replace_scheduled_toasts(&[])
        .expect("first empty queue reconcile should succeed");
    pixeldone_windows_lib::platform::windows::notification::replace_scheduled_toasts(&[])
        .expect("second empty queue reconcile should remain idempotent");
}
