# PixelDone for Windows 3.2.1

PixelDone for Windows 3.2.1 is a desktop polish and notification reliability release.

## Highlights

- Uses the localized `Options` name consistently across Android and Windows.
- Expands the resizable sidebar range to 200–560 px and uses 320 px as its default and reset width.
- Matches the Android Dock action geometry, button sizes, add button, spacing, and state presentation.
- Replaces Settings action labels with fixed 44 px icons and prevents translated text buttons from growing vertically.
- Reduces todo completion controls to 28 px and localizes priority labels in todo rows.
- Removes terminal sentence periods from system UI presentation.
- Repairs stale Start menu shortcut targets before registering the Windows notification identity.

## Installation and update

- Run `PixelDone_3.2.1_x64-setup.exe` to install or overwrite an existing 3.1.x or 3.2.0 installation for the current Windows user.
- The installer intentionally has no Authenticode publisher signature, so Windows may display an unknown-publisher warning.
- The detached Tauri updater signature and `latest.json` provide update-download integrity checking; they do not provide Windows publisher identity authentication.

## Cloud prerequisite and risk

- PixelDone 3.2 remains a hard protocol cutover and does not fall back to schema 3.1.
- The Storage-owner policy script and public 3.2 migration were already applied and their consolidated checks passed on 2026-07-13.
- PixelDone intentionally connects to the configured direct-IP Supabase deployment over cleartext HTTP/WS. This transport does not provide confidentiality, integrity, or server identity protection, and no HTTPS migration is planned.
- The operator explicitly authorized formal publication while installed two-device image, password/global-logout, and Realtime verification remains incomplete. The notification shortcut repair is automated and tested, but installed notification presentation still requires separate manual regression and is not represented as passed.
