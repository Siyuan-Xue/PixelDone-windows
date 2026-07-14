# PixelDone for Windows 3.2.4

PixelDone for Windows 3.2.4 starts from the published 3.2.3 baseline and focuses on synchronization correctness, conflict clarity, and cross-platform presentation.

## Synchronization

- Stops a todo edit from falsely marking its parent checklist as changed.
- Adds durable pristine cloud records and mutation payloads to SQLite.
- Uses field-level three-way merge: independent local and cloud edits merge automatically; only a different edit to the same semantic field opens Review.
- Reuses the same mutation UUID and payload after a transient failure.
- Excludes the local-only Trash and Settings destinations from checklist upload, tombstones, and conflict review.
- Keeps Supabase Realtime subscriptions for checklists, todos, attachments, settings, and tombstones. There is no fixed-interval cloud polling loop; Realtime events trigger a debounced transactional cursor pull.

## Interface

- Replaces raw conflict JSON and ambiguous numeric summaries with named fields, human-readable values, and explicit “This device” / “Cloud version” choices.
- Removes the separator, label, and action controls between active and completed todos.
- Bundles OFL-licensed Source and Noto families. Serif faces are used for significant headings and sans-serif faces for compact UI text, including Chinese and Arabic-specific families.

## Release status

- Version metadata, tag, updater manifest, and the x64 NSIS artifact target 3.2.4.
- The formal NSIS package carries the required Tauri updater integrity signature. Authenticode remains intentionally disabled, so Windows can still identify the installer as an unknown publisher.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
