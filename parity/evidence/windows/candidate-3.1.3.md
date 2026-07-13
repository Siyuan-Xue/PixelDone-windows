# Windows 3.1.3 candidate verification evidence

PixelDone Windows 3.1.3 is an implementation candidate and has not been tagged, pushed, or published as a Release.

## Scope

- The window title carries the `CODEX & XUE` identity while the installer, executable, AUMID, and data identity remain unchanged.
- Sidebar and workspace headers share a 64 px height; redundant in-page product labels and the ordinary workspace sync chip are removed.
- The checklist sidebar is resizable from 220 to 420 px and persists its device-local width in SQLite migration `0006_layout.sql`.
- Language option anatomy follows the app direction while each native language label chooses its own text direction.
- The signed-out Android synchronization hint is rendered by the Windows localization layer in English, Simplified Chinese, Arabic, French, Russian, and Spanish instead of exposing the Chinese backend message.
- Dock controls align at their bottom edge, settings switches retain a fixed 44 by 26 px footprint, and checklist creation uses a borderless clay plus.
- Scheduled notification replacement snapshots the Windows queue before deletion, treats an already-removed item as idempotent, retries identity registration once, and preserves an installer-created Start Menu shortcut target.
- Windows icon outputs use a transparent background with a larger PixelDone checklist subject.

## Verification

- On 2026-07-13, `bun run check` completed with 0 errors and 0 warnings, all 16 Bun tests passed, and the production Svelte build completed without the previous missing `.svelte-kit/tsconfig.json` warning. The six added localization assertions cover every supported language.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed. Rust coverage comprises 20 library tests, 1 migration test, and 7 ordinary Windows productization tests; formal updater signature verification remains intentionally ignored until a protected release artifact exists.
- The real Tauri WebDriver suite passed all 10 spec files and 18 tests. It verifies the 64 px header rhythm, exact window title, absence of redundant page identity/sync controls, LTR/RTL sidebar sizing, width persistence and clamping, language-option anatomy, fixed switches, bottom-aligned Dock buttons, and the existing checklist, todo, trash, reminder, image, cloud, and update flows. The settings spec was rerun after the localization fix and confirmed the signed-out synchronization hint changes between Arabic and English at runtime.
- The parity gate reports 41 of 41 required rows verified (100%).
- The unsigned local x64 NSIS candidate built successfully with Tauri's explicit `--no-sign` option at `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/PixelDone_3.1.3_x64-setup.exe`; SHA-256 is `631DCF97B5690ED4EDF7F83365BD1A640D906B3271AB48BAF944EF0B48FBD6CB`.
- A silent in-place install over the existing 3.1.2 installation returned exit code 0. File and registry versions are 3.1.3, the installation remains `%LOCALAPPDATA%\PixelDone`, and the Start Menu shortcut target is `%LOCALAPPDATA%\PixelDone\PixelDone.exe` rather than the stale sandbox path.
- The localization-patched 3.1.3 candidate also completed a silent same-version in-place install with exit code 0 and restarted minimized from the same installation directory.
- Existing 3.1.2 SQLite migration checksums match the now-pinned CRLF migration files exactly. The installed 3.1.3 app applied `0006_layout.sql`, retained the existing checklist/todo/dirty/tombstone counts, and continues running minimized without stderr.
- The installed AUMID and Stub CLSID passed two consecutive notification queue reconciliations. `ToastNotifier.Setting()` returning `0x80070490` before Windows creates an unpackaged desktop settings entry no longer blocks authoritative scheduled-queue operations; real disabled settings and all other failures still surface.
