# Android 3.2.0 candidate verification evidence

PixelDone Android 3.2.0 is a local candidate only. It has not been pushed, tagged, or published.

- Room schema 6 persists attachment identifiers, private Storage paths, SHA-256, MIME type, byte size, attachment timestamps/versions, transfer state, and per-image errors.
- Settings verifies the current password, updates it through Supabase Auth, requests global logout, clears the local Keystore-backed session, and requires a fresh sign-in.
- JPEG, PNG, and WebP originals are limited to 10 MiB, validated by signature and decodable bounds, uploaded to private Supabase Storage, and downloaded only when previewed.
- Realtime subscribes to `todo_attachments` in addition to the four existing sync tables; no periodic sync job is registered.
- `lintDebug`, `testDebugUnitTest`, `assembleDebug`, and `assembleRelease` passed on 2026-07-13.
- `PixelDone-3.2.0-debug.apk` is 18,136,709 bytes with SHA-256 `4B91346B2D511EAE3286ECC160086F0C93D0263A7AD6DFBDD48037CA49417A0B`; APK Signature Scheme v2 verifies with the Android debug certificate.
- `PixelDone-3.2.0-release.apk` is 13,126,291 bytes with SHA-256 `184C51CEF53DDE106071D1128080A8B3DCCE72E0FFD05AB238B38F64759E07FF`; APK Signature Scheme v2 verifies with the long-lived PixelDone 4096-bit release certificate.
- On 2026-07-13, the operator applied the private Storage policies as `supabase_storage_admin`, then completed the 3.2 public-schema cutover. The consolidated gate returned `true` for schema version, bucket limits/MIME types, RPC overloads, five-table Realtime publication, all four owner-scoped Storage policies, and the active daily trash cron.
- Pixel 10 installation and two-device 3.2 Storage, password/global-logout, Realtime, and notification regression verification remain pending. The related parity rows intentionally remain `in_progress`.
