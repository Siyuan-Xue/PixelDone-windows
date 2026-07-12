# PixelDone for Windows 3.1.2

## Planned delivery

- Replace the permanent three-column Inspector layout with a checklist sidebar and one main workspace.
- Align light and dark UI tokens, task rows, controls, typography, and the floating action dock with the PixelDone Android 3.1.1 design language.
- Give checklist creation its own sidebar button and reserve the main `+` button for task creation.
- Use a centered rectangular modal for task creation and editing while preserving all existing IPC, sync, reminder, image, update, and storage behavior.
- Move checklist counts, sync state, conflicts, notification failures, and update availability into a compact main status bar.
- Remove the redundant release-workflow `--config` argument that tauri-action first reports as invalid inline JSON.
- Run `svelte-kit sync` before each Vite production/E2E build so clean workspaces do not warn about the generated SvelteKit TypeScript base config.
- Continue the 3.1.1 distribution contract: unsigned Authenticode publisher, current-user NSIS identity, and signed Tauri updater artifacts.

## Compatibility

- Product identifier, executable name, installation directory, SQLite schema, application data roots, Credential Manager target, protocol activation, updater public key, and Supabase sync protocol remain unchanged.
- PixelDone 3.1.1 can be upgraded in place without uninstalling or creating a second formal installation.
- Android is used as the visual and interaction authority but is not changed by this Windows iteration.

## Verification gates

- [x] `bun run check`
- [x] `bun test`
- [x] `bun run build` from a workspace without `.svelte-kit`
- [x] `bun run build:e2e` from a workspace without `.svelte-kit`
- [x] `cargo fmt --check`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo test`
- [x] Full desktop WebDriver E2E suite (10/10 spec files, 16 tests)
- [x] `bun run parity:check` (100%)
- [x] Two-pane UI at 1000x680, 1180x780, and 1440x900, plus light/dark theme inspection
- [x] English, Simplified Chinese, Arabic RTL, and native French/Russian labels
- [x] Empty-list and very-long-task-title visual edge cases
- [x] Sidebar-only checklist creation, task-only `+`, all three Dock plus placements, modal shortcut isolation, focus, and Esc
- [x] Unsigned local x64 NSIS validation bundle
- [x] Installed in-place upgrade from 3.1.1
- [x] Signed Tauri updater artifact and embedded-public-key verification

The formal `v3.1.2` tag and GitHub Release were published after all gates passed. Authenticode remains intentionally disabled; the protected GitHub Actions release job produced the Tauri updater signature, and the downloaded artifact passed embedded-public-key verification.
