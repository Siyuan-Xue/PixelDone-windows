# Windows 3.2.1 candidate verification evidence

PixelDone Windows 3.2.1 is a patch release for the desktop UI and Windows notification identity. The operator explicitly requested formal publication on 2026-07-13. The six pre-existing cross-device cloud rows remain unverified and are not represented as passed.

- The settings destination is consistently presented as the localized `Options` label.
- Sidebar persistence now supports 200–560 px and uses 320 px as the default and reset width.
- Dock action geometry, the 56 px add button, spacing, and state presentation match the Android reference implementation.
- Settings actions use fixed 44 px icon buttons, and remaining text buttons do not grow vertically when labels are long.
- Todo completion controls are 28 px, and todo priority labels follow the active locale.
- User-interface presentation removes terminal sentence periods without changing question marks or user-authored content.
- Windows notification identity setup now validates the existing Start menu shortcut target and rewrites stale targets before registering scheduled toast activation.
- `bun test` passed 18 tests with 64 expectations; `bun run check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and the production frontend build passed.
- Rust verification passed 23 unit tests, 1 migration test, and 7 Windows productization tests; the single formal-artifact test remained intentionally ignored until the installer existed.
- Native Windows E2E verification passed all 10 spec files, including the 3.2.1 update-version assertion and notification-shortcut identity coverage.
- The formal x64 NSIS build produced `PixelDone_3.2.1_x64-setup.exe` at 5,399,926 bytes with SHA-256 `ADC2AA22D21F6434E32FBFC0602D160DA8A52A4E7C1D84E21C173E4312108A0C`.
- The detached Tauri updater signature is 420 bytes. `cargo test --test release_integrity -- --ignored` verified it against the embedded public key.
- Generated `latest.json` reports version 3.2.1, the v3.2.1 GitHub installer URL, and the 420-character updater signature.
- The installer remains `NotSigned` under Windows Authenticode, matching the documented project release posture; updater authenticity is provided by the verified Tauri signature.
- Installed-notification display remains a manual post-install observation and is not represented as verified by the automated shortcut and toast-registration coverage.

## Published release verification

- GitHub Actions run `29263815905` completed successfully for source commit `e3e5f77e07eb332c629814bb10edf03d406d078f` and immutable tag `v3.2.1`.
- The public, non-draft, non-prerelease `PixelDone v3.2.1` release was published on 2026-07-13 with four expected assets: the x64 NSIS installer, detached updater signature, SHA-256 file, and `latest.json`.
- The cloud-built installer is 5,397,428 bytes with SHA-256 `349F2E852E32C5C3CB3F722F42944F55AC2A21CAB8AB5C0A3468946FDC7EB7AC`.
- The uploaded SHA-256 file matches GitHub's installer asset digest exactly. The public `latest.json` reports version 3.2.1, the v3.2.1 installer URL, and a 420-character updater signature.
