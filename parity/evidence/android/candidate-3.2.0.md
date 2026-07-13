# Android 3.2.0 candidate verification evidence

PixelDone Android 3.2.0 source was pushed to `main`. On 2026-07-13, the operator explicitly authorized formal publication while the installed/two-device checks below remained incomplete.

- Room schema 6 persists attachment identifiers, private Storage paths, SHA-256, MIME type, byte size, attachment timestamps/versions, transfer state, and per-image errors.
- Settings verifies the current password, updates it through Supabase Auth, requests global logout, clears the local Keystore-backed session, and requires a fresh sign-in.
- JPEG, PNG, and WebP originals are limited to 10 MiB, validated by signature and decodable bounds, uploaded to private Supabase Storage, and downloaded only when previewed.
- Realtime subscribes to `todo_attachments` in addition to the four existing sync tables; no periodic sync job is registered.
- `lintDebug`, `testDebugUnitTest`, `assembleDebug`, and `assembleRelease` passed on 2026-07-13.
- `PixelDone-3.2.0-debug.apk` is 18,136,709 bytes with SHA-256 `4B91346B2D511EAE3286ECC160086F0C93D0263A7AD6DFBDD48037CA49417A0B`; APK Signature Scheme v2 verifies with the Android debug certificate.
- The final `PixelDone-3.2.0-release.apk` is 13,126,291 bytes with SHA-256 `F33EE30A0F181FD94B019721D8AF4544492CEB664E00325A97E82352B56D5438`; APK Signature Scheme v2 verifies with the long-lived PixelDone 4096-bit release certificate (`6d146e63d8f96d383fd9bbcfd61c61c343d7d7ecb6c98d33db5bd7dbf56d2317`).
- On 2026-07-13, the operator applied the private Storage policies as `supabase_storage_admin`, then completed the 3.2 public-schema cutover. The consolidated gate returned `true` for schema version, bucket limits/MIME types, RPC overloads, five-table Realtime publication, all four owner-scoped Storage policies, and the active daily trash cron.
- Pixel 10 installation and two-device 3.2 Storage, password/global-logout, Realtime, and notification regression verification remain pending. The related parity rows intentionally remain `in_progress`.
