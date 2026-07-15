# Windows 3.2.6 formal release evidence

PixelDone Windows 3.2.6 is the formal successor to the published, immutable 3.2.5 baseline plus the source-only checklist alignment fix on `main`.

- Password changes now use the same focused rectangular dialog model as sign-in and registration, including localized validation, password visibility, focus containment, dismissal isolation, and signed-out success feedback.
- The manual Sync action is located in the Sync row, ordinary checklist navigation no longer displays todo counts, and Trash retains its item count.
- UI mutations silently refresh and retry once after a normal revision race; repeated races and remote deletion use non-red guidance, while old and gapped deltas cannot regress the WebView snapshot.
- Authentication commands validate the expected revision before cloud side effects and converge successful cloud authentication changes onto the latest local snapshot.
- Automatic sync retries transient network failures with capped exponential backoff and publishes structured issue codes for network, authentication, server compatibility, local storage, remote data, and unknown failures.

## Release verification

- Bun unit tests passed 29 tests with 182 expectations; Svelte diagnostics, the production build, the parity gate, and the complete Windows WebView2 E2E suite passed. The E2E result covers 22 scenarios across all 11 specs.
- Rust passed 32 unit tests, 3 migration tests, 7 Windows productization tests, formatting, and Clippy with warnings denied. The formal updater-signature integrity test also passed against the generated 3.2.6 package.
- The parity gate authorizes the formal release at 85.71% while preserving every incomplete dual-device row as unverified. Android remains unchanged by this release.
- The local formal x64 NSIS package is 29,073,035 bytes with SHA-256 `98C9F86F9023EA08199A45734037F0751B26C6E9D1DDFB2D1A9F5AC2110A2651`; its 420-byte Tauri updater signature has SHA-256 `9BECA792DAEF4EEF9BA0304F00E5EBE8841C2DB5BBE21FC5D90D439219AA9A08`.
- Executable metadata reports product and file version 3.2.6. `latest.json` reports 3.2.6, points to the immutable `v3.2.6` installer URL, and embeds the exact generated updater signature. Public artifacts are checked separately after the immutable GitHub Release upload.
- Native installer overwrite, installed-app smoke testing, and the six dual-device cloud scenarios are not represented as verified unless they are actually reproduced. They remain explicitly authorized release exceptions.
