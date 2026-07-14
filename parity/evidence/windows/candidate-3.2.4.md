# Windows 3.2.4 formal release evidence

PixelDone Windows 3.2.4 is the formal successor to the published, immutable 3.2.3 baseline.

- Local todo or checklist edits no longer dirty the parent checklist merely because its embedded item collection changed.
- Synchronization persists pristine cloud records and durable mutation payloads, performs field-level three-way merges, and only creates a review conflict when both sides changed the same semantic field differently.
- Mutation retries reuse the original mutation UUID and payload so network recovery remains idempotent.
- Trash and Settings are local synthetic destinations and are excluded from cloud checklist pushes, tombstones, and conflict review.
- Conflict review presents semantic fields and human-readable device/cloud values instead of raw JSON or ambiguous numeric summaries.
- The completed-task separator, label, and group actions were removed; completed rows now follow active rows directly.
- Bundled OFL fonts provide a shared serif-title and sans-serif-UI hierarchy across supported languages.
- Supabase Realtime remains the change trigger for five synchronized tables. No fixed-interval cloud polling was added; reminder and update timers remain purpose-specific local maintenance.

## Release verification

- Bun passed 19 tests with 68 expectations; Svelte reported 0 errors and 0 warnings; the production frontend build completed.
- All 10 native Windows E2E spec files passed against the Tauri/WebView2 binary, including checklist, todo, settings/RTL, layout, reminder, image, Trash, cloud-boundary, and update scenarios.
- Rust passed 28 unit tests, 3 migration tests, 7 Windows productization tests, formatting, and Clippy with warnings denied. The deployed-database, formal-signature, and installed-notification tests remain explicitly ignored because they require external artifacts or installed identity.
- Android companion 3.2.2 passed its full JVM unit suite, `lintDebug`, and `assembleDebug`.
- The local formal x64 NSIS package is 29,045,594 bytes with SHA-256 `C8DF636DFAB2AB5DE7F45C2852FEB659D050BF676E20B7D93C605273E4DB4700`; its 420-byte Tauri updater signature has SHA-256 `F1AFF6673DCAA4CF5FDD22B5D1C7E0F43D7480D9644B8EDAE3DAA56DF327BE96`, and executable metadata reports version 3.2.4. Public artifacts are checked separately after the immutable GitHub Release upload.
- Native installer overwrite, installed-app smoke testing, and six dual-device cloud scenarios were not reproduced locally. They remain explicitly authorized release exceptions and must not be inferred from this document.
