# Windows 3.2.7 formal release evidence

PixelDone Windows 3.2.7 is the formal successor to the immutable 3.2.6 release. It incorporates the approved checklist, conflict-recovery, localization, layout, release automation, and updater-fallback changes on `main`.

- Checklist deletion uses a dedicated localized confirmation dialog and preserves focus and modal keyboard isolation.
- UI mutations and synchronization recover from common revision races and transient network failures while keeping actionable status in the relevant surface.
- The checklist sidebar supports persisted widths from 200 through 720 pixels and dynamically preserves at least 440 pixels for the workspace.
- The updater keeps GitHub as the primary signed manifest source and resolves the same or newest strict stable version from the Gitee Release API only when GitHub discovery or installation fails.
- The release workflow builds and signs one NSIS artifact, verifies its signature and SHA-256, then publishes byte-identical assets and notes to immutable GitHub and Gitee Releases.

## Release verification

- Local verification passed Svelte diagnostics with zero errors and warnings, 30 Bun tests with 246 expectations, the production build, and the parity gate at 85.71%.
- Rust verification passed formatting, Clippy with warnings denied, 37 unit tests, 5 migration tests, and 7 Windows productization tests. The two ignored tests require a deployed database copy or installed notification identity, and the ignored release-integrity test is reserved for the signed CI artifact.
- The complete binary WebView2 suite passed all 24 scenarios across 11 specs, including checklist deletion, revision recovery, the 720-pixel sidebar bound, RTL/LTR layout, authentication dialogs, synchronization, reminders, storage, and the Rust update version.
- The tag-triggered GitHub Actions workflow repeats the quality gate before building the signed x64 NSIS package and publishing it to both release providers.
- The parity gate continues to preserve all authorized cross-device cloud scenarios as incomplete; they are not represented as verified.
- Exact public installer size, SHA-256, updater signature, manifest URLs, and provider-side asset hashes are verified by CI and the publishing scripts rather than predeclared in source evidence.
