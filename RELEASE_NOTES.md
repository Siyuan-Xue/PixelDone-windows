# PixelDone for Windows 3.2.7

PixelDone for Windows 3.2.7 improves checklist safety, synchronization recovery, large-window layout, and release resilience. It also establishes Gitee as a verified secondary Release and updater source without creating a second build authority.

## Checklist and layout

- Adds a dedicated localized checklist-deletion confirmation dialog with focus containment, Escape/backdrop behavior, and pending-request isolation.
- Keeps ordinary checklist navigation compact while preserving Trash counts and clearer conflict/retry feedback.
- Expands the persisted checklist sidebar range to 200–720 pixels and dynamically reserves at least 440 pixels for the main workspace.
- Refreshes Windows parity screenshots and layout coverage for the updated interface.

## Synchronization reliability

- Silently refreshes and retries normal revision races once, while repeated conflicts and remote deletion receive non-destructive guidance.
- Prevents delayed or gapped mutations from regressing the displayed snapshot.
- Keeps transient network recovery, authentication expiry, server compatibility, local-storage, remote-data, and unknown failures in structured Sync status instead of the global alert.
- Preserves the focused password-change flow and converges successful authentication changes onto the latest local snapshot.

## Release and updater resilience

- Runs the complete Bun, Svelte, parity, formatting, Clippy, and Rust test gates in CI before packaging.
- Builds and updater-signs one x64 NSIS installer, then publishes byte-identical artifacts and release notes to GitHub and Gitee Releases.
- Adds a recovery dispatch that can rebuild a missing Gitee Release from the immutable verified GitHub Release without rebuilding the application.
- Keeps GitHub as the primary updater source and falls back through the Gitee Releases API when GitHub discovery fails.
- If a GitHub download or installation fails, restricts Gitee fallback to the exact same version to prevent an unintended version switch during installation.
- Verifies every downloaded installer with the embedded Tauri updater public key; Gitee is a transport fallback, not a trust fallback.

## Release status

- Version metadata, tag, updater manifests, and the x64 NSIS artifact target 3.2.7.
- Authenticode remains intentionally disabled, so Windows can identify the installer as an unknown publisher. The Tauri updater signature and published SHA-256 protect artifact integrity.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
