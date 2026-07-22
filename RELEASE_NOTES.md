# PixelDone for Windows 3.2.9

PixelDone for Windows 3.2.9 fixes task deletion from the editor so the confirmation dialog stays visible and clickable above the editing dialog.

## Task deletion confirmation

- Places the destructive confirmation backdrop above the task editor with an explicit modal stacking level.
- Keeps the editor and its draft state intact while the confirmation is open.
- Preserves Cancel, Escape, focus restoration, and confirmed move-to-Trash behavior.

## Quality and release status

- Adds a real WebView2 regression check for computed stacking order and pointer hit-testing at the confirmation button.
- Version metadata, tag, updater manifests, and the x64 NSIS artifact target 3.2.9.
- GitHub Actions builds and signs one installer, publishes it to GitHub Release, and mirrors the same bytes to Gitee Release.
- Authenticode remains intentionally disabled, so Windows can identify the installer as an unknown publisher. The Tauri updater signature and published SHA-256 protect artifact integrity.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
