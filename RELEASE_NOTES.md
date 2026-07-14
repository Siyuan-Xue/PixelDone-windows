# PixelDone for Windows 3.2.2

PixelDone for Windows 3.2.2 is an emergency startup hotfix for 3.2.1.

## Fix

- Fixes the immediate startup exit seen after installing 3.2.1 over a database previously used by 3.2.0.
- Preserves the exact migration 7 checksum deployed by 3.2.0 instead of allowing the GitHub Windows checkout to change LF bytes to CRLF.
- Adds Bun and Rust checksum regression tests and runs the Bun suite as a required release gate.
- Preserves existing todos, settings, authentication state, attachments, and the complete 3.2.1 UI update.

## Installation and update

- Run `PixelDone_3.2.2_x64-setup.exe` to overwrite 3.2.1 for the current Windows user. Do not delete the PixelDone data directory.
- The installer intentionally has no Authenticode publisher signature, so Windows may display an unknown-publisher warning.
- The detached Tauri updater signature and `latest.json` provide update-download integrity checking; they do not provide Windows publisher identity authentication.

## Verification and existing scope

- The 3.2.1 failure was reproduced with exit code 101 and an explicit SQLx migration checksum error.
- A disposable copy of the affected installed database upgraded through migration 8 with SQLite integrity `ok` and its existing sidebar width preserved.
- PixelDone 3.2 remains a hard protocol cutover and intentionally uses the configured direct-IP Supabase deployment over cleartext HTTP/WS.
- The six previously documented cross-device cloud rows and installed notification presentation remain incomplete and are not represented as passed by this hotfix.
