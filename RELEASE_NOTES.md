# PixelDone for Windows 3.2.8

PixelDone for Windows 3.2.8 improves deletion safety, makes Trash easier to search and manage, and fixes a misleading notification warning when testing a development build beside an installed release.

## Safer destructive actions

- Uses one localized in-app alert dialog for deleting a todo, clearing completed todos, deleting a checklist, permanently deleting one Trash item, emptying all of Trash, and removing the legacy Roaming database.
- Defaults focus to Cancel, traps keyboard focus, supports Escape and backdrop cancellation, restores focus to the triggering control, and prevents duplicate submission while an action is running.
- Shows the affected item count for clearing completed todos and emptying Trash, while revision checks continue to protect every confirmed mutation.
- Keeps quick delete as an explicitly enabled one-click move into recoverable Trash.

## Trash navigation and layout

- Replaces the textual restore and permanent-delete actions with accessible borderless icon buttons.
- Displays the original checklist name without the broken `%1$s` placeholder or a redundant prefix.
- Adds title search plus priority and source-checklist filters, with combined matching and an independent no-results state.
- Keeps search, both filters, and Empty Trash on one responsive toolbar; Empty Trash still clears the entire Trash rather than only filtered results.

## Windows notification identity

- Allows a direct development build to reuse the installed PixelDone executable for Windows notification shortcut identity.
- Avoids showing “Notifications need attention” merely because the currently running executable came from `target/debug`, while still reporting a real identity error when no installed build is available.

## Quality and release status

- Expands unit and WebView2 coverage for destructive confirmations, Trash filtering, focus restoration, responsive and RTL layouts, stale-revision retry behavior, and notification identity selection.
- Version metadata, tag, updater manifests, and the x64 NSIS artifact target 3.2.8.
- GitHub Actions builds and signs one installer, publishes it to GitHub Release, and mirrors the same bytes to Gitee Release.
- Authenticode remains intentionally disabled, so Windows can identify the installer as an unknown publisher. The Tauri updater signature and published SHA-256 protect artifact integrity.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
