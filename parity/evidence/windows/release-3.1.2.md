# Windows 3.1.2 verification evidence

## Implemented build-log cleanup

- The Windows release workflow no longer passes `--config src-tauri/tauri.windows.conf.json` to `tauri-action`; Tauri continues to merge the platform config automatically, avoiding the action's misleading inline-JSON parse message.
- Production and E2E frontend builds now run `svelte-kit sync` before Vite so a clean workspace generates `.svelte-kit/tsconfig.json` before `tsconfig.json` is resolved.
- Rust productization tests verify that the workflow has no `--config` argument and that `tauri.windows.conf.json` still selects NSIS with `currentUser` installation.
- Clean-workspace local production and E2E builds no longer contain either original warning. A tagged GitHub Actions run is still required to confirm the hosted `tauri-action` log.

## Verified static and build gates

- On 2026-07-13, `bun run check` passed with 0 errors and 0 warnings.
- On 2026-07-13, `bun test` passed 10 tests across the parity target, version contract, Dock ordering, and editor reconciliation suites.
- Clean-workspace production and E2E frontend builds were each started after moving the ignored `.svelte-kit` directory out of the repository. Both regenerated the SvelteKit TypeScript base config and completed without the missing-base-config warning.
- The production build transformed 159 SSR and 173 client modules; the E2E frontend build transformed 159 SSR and 177 client modules.
- On 2026-07-13, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed. Rust tests comprised 18 library tests, 1 migration test, and 5 Windows productization tests; the formal updater-signature test is run after the protected release job publishes the signed 3.1.2 artifact.
- The full binary WebDriver suite passed all 10 spec files and 16 tests. It covered the two permanent regions, sidebar-only checklist creation, task-only Dock `+`, all three plus placements, 1000×680 / 1180×780 / 1440×900 CSS viewports at 150% Windows scaling, empty lists, very long task titles, physical Dock edges in RTL, modal shortcut isolation, reminders, seven language labels, task completion grouping, trash, and the Rust update version.
- Version inspection and Rust compilation agree on `3.1.2` in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`.
- The current parity input is `parity/pixeldone-3.1.2.yaml`. It preserves the Android 3.1.0 functional baseline and verifies all 41 required Windows rows at 100%.

## Accepted visual baseline

- The target desktop structure is a checklist sidebar plus one main task workspace, with no permanent Inspector column.
- Checklist creation belongs to a dedicated sidebar button; the floating main `+` remains task-only, and the other Dock actions remain separate square controls with small gaps.
- The main workspace must include a compact status bar, task list, floating Dock, centered rectangular task editor, in-workspace settings, and in-workspace trash.
- `parity/evidence/windows/main-1180x780.png` is the accepted 1180×780 light-theme empty-state structural baseline. It confirms the two permanent regions, sidebar checklist button, workspace status bar, in-workspace empty state, separate square Dock controls, and absence of the Inspector.
- Interactive visual inspection covered the implemented task rows, centered editor, Settings, light/dark themes, and 1000×680 / 1180×780 / 1440×900 layouts without horizontal overflow. `e2e/specs/layout.e2e.ts` independently verifies the three CSS viewports, RTL physical edges, separate square Dock controls, and modal shortcut isolation.
- `parity/evidence/windows/settings-arabic.png` records the Arabic RTL settings state; the Settings E2E verifies all seven native language labels and the two-column language grid.

## Local NSIS artifact

- Local x64 bundle: `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/PixelDone_3.1.2_x64-setup.exe`.
- Local validation size: `5,304,260` bytes.
- Local validation SHA-256: `FB07827B7AF298A5613F58745132FD1462B7EC85725CB3A59A557728169FDF2E`.
- Authenticode status: `NotSigned`, matching the accepted unknown-publisher distribution policy.
- The protected GitHub Actions release job receives the encrypted updater-key password from repository secrets and publishes the Tauri `.sig` plus `latest.json`; Authenticode remains intentionally disabled.

## Installed upgrade verification

- The published unsigned `PixelDone_3.1.1_x64-setup.exe` was installed silently as the current-user baseline, with registry version `3.1.1` and installation directory `C:\Users\Miles\AppData\Local\PixelDone`.
- The local 3.1.2 validation installer completed with exit code 0 over that installation. The same uninstall identity (`PixelDone`), installation directory, executable name, and uninstall path were retained; the registry, file version, and product version changed in place to `3.1.2`.
- No uninstall was required and no parallel application directory was created.

## Formal release acceptance

- GitHub Actions run `29213244302` completed successfully: parity gate in 1m14s and the protected release job in 11m18s.
- The hosted build log contains `tauri build --bundles nsis --target "x86_64-pc-windows-msvc"` and `svelte-kit sync && vite build`; it contains neither the misleading inline-JSON `--config` message nor the missing `.svelte-kit/tsconfig.json` warning.
- The public `v3.1.2` Release contains `PixelDone_3.1.2_x64-setup.exe`, its `.sig`, and `latest.json` with version `3.1.2` and non-empty update notes.
- Formal installer size: `5,308,210` bytes. Formal SHA-256: `D2521F14364A10AE8F9ECDF23412CB87DCE1AD6C3711514C2007BCDC3F0D72A4`. Authenticode remains `NotSigned` by policy.
- The downloaded formal installer and `.sig` passed `formal_nsis_signature_matches_embedded_public_key` using the public key embedded in the repository.
- Installing the downloaded formal artifact over the validation installation returned exit code 0, retained the current-user `PixelDone` identity and `C:\Users\Miles\AppData\Local\PixelDone` path, reported file/product/registry version `3.1.2`, and retained the Start Menu shortcut.
- Release URL: `https://github.com/Siyuan-Xue/PixelDone-windows/releases/tag/v3.1.2`.
