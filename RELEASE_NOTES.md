# PixelDone for Windows 3.3.0

PixelDone for Windows 3.3.0 adds Markdown checklist export to the configurable Dock.

## Markdown export

- Adds `EXPORT MARKDOWN` to the Dock action picker and task-list Dock.
- Asks for confirmation before copying and offers simple or detailed Markdown.
- Exports every task in the current checklist, including completed tasks hidden by the current view, while preserving the current sort order.
- Simple export contains a checklist heading and Markdown task checkboxes. Detailed export also includes localized priority, due-date, and repeat metadata.
- Uses the scoped Tauri clipboard write permission and confirms a successful copy in the workspace.
- Supports Cancel, Escape, backdrop dismissal, focus containment, RTL layout, and all seven existing interface languages.

## Quality and release status

- Adds Bun regression coverage for Markdown escaping, ordering, completion state, and optional metadata.
- Version metadata, tag, updater manifests, and the x64 NSIS artifact target 3.3.0.
- GitHub Actions builds and signs one installer, publishes it to GitHub Release, and mirrors the same bytes to Gitee Release.
- Authenticode remains intentionally disabled, so Windows can identify the installer as an unknown publisher. The Tauri updater signature and published SHA-256 protect artifact integrity.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
