# Windows 3.3.0 formal release evidence

PixelDone Windows 3.3.0 adds Markdown export to the configurable Dock while preserving the existing database, cloud, reminder, image, and installer contracts.

- The export dialog offers simple and detailed copies, includes completed and unfinished tasks in the current sort order, and copies a Markdown heading plus task checkboxes.
- Detailed export adds localized priority, due-date, and repeat metadata.
- Clipboard access uses the scoped Tauri clipboard write permission; the modal supports Cancel, Escape, backdrop dismissal, and keyboard focus containment.
- Bun tests cover Markdown escaping, ordering, completion state, and optional metadata.
- The native WebView2 flow verifies the Dock action, modal visibility, Escape dismissal with focus restoration, detailed-copy clipboard call, success notice, and modal closure.
- The Android launcher widget is an explicit platform-only feature; Windows has no launcher widget surface.

## Release verification

- Local verification covers Svelte diagnostics, Bun tests, the production build, the parity gate, Rust formatting, Clippy with warnings denied, Rust tests, and the complete binary WebView2 suite.
- The parity gate continues to preserve all authorized cross-device cloud scenarios as incomplete; they are not represented as verified.
- Exact public installer size, SHA-256, updater signature, manifest URLs, and provider-side asset hashes are verified by CI and the publishing scripts rather than predeclared in source evidence.
