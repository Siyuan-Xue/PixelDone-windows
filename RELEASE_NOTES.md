# PixelDone for Windows 3.2.0

PixelDone for Windows 3.2.0 adds authenticated password changes and private cross-device todo image synchronization.

## Highlights

- Synchronizes one original JPEG, PNG, or WebP image up to 10 MiB per todo through native private Supabase Storage.
- Downloads remote images only when opened, validates image signatures, decoding and SHA-256, and retries object cleanup independently from ordinary todo synchronization.
- Verifies the current password before updating it, requests global Supabase logout, clears the local Windows credential, and requires a fresh sign-in.
- Subscribes to attachment metadata alongside the four existing Realtime tables while keeping the transactional cursor pull as the sole merge path.
- Runs no fixed-interval synchronization polling loop.
- Keeps standard Windows notifications for every priority by default; enhanced XHIGH alarm behavior remains opt-in.

## Installation and update

- Run `PixelDone_3.2.0_x64-setup.exe` to install or overwrite an existing 3.1.x installation for the current Windows user.
- The installer intentionally has no Authenticode publisher signature, so Windows may display an unknown-publisher warning.
- The detached Tauri updater signature and `latest.json` provide update-download integrity checking; they do not provide Windows publisher identity authentication.

## Cloud prerequisite and risk

- PixelDone 3.2 is a hard protocol cutover and does not fall back to schema 3.1.
- The operator applied the Storage-owner policy script and public 3.2 migration, then confirmed every consolidated schema, bucket, RPC, Realtime, policy, and cron check on 2026-07-13.
- PixelDone intentionally connects to the configured direct-IP Supabase deployment over cleartext HTTP/WS. This transport does not provide confidentiality, integrity, or server identity protection, and no HTTPS migration is planned.
- The operator explicitly authorized formal publication before installed two-device image, password/global-logout, Realtime, and notification regression verification was completed. Those checks remain pending and are not represented as passed.
