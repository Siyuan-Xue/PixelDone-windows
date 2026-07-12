use std::fs;

#[test]
fn formal_config_uses_professional_identity_and_protocol() {
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("tauri.conf.json").unwrap()).unwrap();
    assert_eq!(config["productName"], "PixelDone");
    assert_eq!(config["version"], "3.1.1");
    assert_eq!(config["mainBinaryName"], "PixelDone");
    assert_eq!(
        config["bundle"]["windows"]["nsis"]["installMode"],
        "currentUser"
    );
    assert_eq!(
        config["plugins"]["deep-link"]["desktop"]["schemes"][0],
        "pixeldone-reminder"
    );
}

#[test]
fn windows_icon_source_preserves_android_geometry_and_colors() {
    let icon = fs::read_to_string("../assets/pixeldone-icon.svg").unwrap();
    for token in [
        "#262624",
        "#4B463E",
        "#FAF9F5",
        "#D97757",
        "#141413",
        "#6A9BCC",
        "#629987",
        "M34 28h40v52H34z",
        "M44 24h20v10H44z",
    ] {
        assert!(icon.contains(token), "missing Android icon token: {token}");
    }
}

#[test]
fn formal_release_matches_the_3_1_0_unsigned_publisher_policy() {
    let workflow = fs::read_to_string("../.github/workflows/release-windows.yml").unwrap();
    assert!(workflow.contains("TAURI_SIGNING_PRIVATE_KEY"));
    assert!(workflow.contains("tauri.windows.conf.json"));
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
