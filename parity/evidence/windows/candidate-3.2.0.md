# Windows 3.2.0 candidate verification evidence

PixelDone Windows 3.2.0 is a local candidate only. It has not been pushed, tagged, or published.

- SQLite migration `0007_attachment_sync.sql` separates durable attachment transfer metadata from todo snapshot replacement and backfills existing local image markers.
- Settings verifies the current password, updates it through Supabase Auth, requests global logout, clears Windows Credential Manager, and requires a fresh sign-in.
- JPEG, PNG, and WebP originals are limited to 10 MiB, validated by signature/decode/hash, uploaded to private Supabase Storage, and downloaded only when previewed.
- Realtime subscribes to five tables including `todo_attachments`; there is no fixed-interval synchronization poll.
- Svelte check passed with 0 errors and 0 warnings, all 17 Bun tests passed, and the production Svelte build completed without the old missing `.svelte-kit/tsconfig.json` warning.
- `cargo fmt --check` and clippy with warnings denied passed. Rust executed 22 library tests, 1 migration integration test, and 7 ordinary Windows productization tests; 2 formal installed-artifact tests remain intentionally ignored at candidate time.
- The no-sign Tauri/NSIS build completed without passing a redundant `--config` argument. `PixelDone_3.2.0_x64-setup.exe` is 5,383,705 bytes with SHA-256 `7B85BE1C55865E527D99E5D3340EBD5539E81194673BA51E90B14205FC795A0B`; Windows reports `NotSigned`, as required by the established unsigned-publisher policy.
- On 2026-07-13, the operator applied the private Storage policies as `supabase_storage_admin`, then completed the 3.2 public-schema cutover. The consolidated gate returned `true` for schema version, bucket limits/MIME types, RPC overloads, five-table Realtime publication, all four owner-scoped Storage policies, and the active daily trash cron.
- The parity report remains intentionally held below 100% until installed/two-device Storage, password/global-logout, Realtime, and notification regression verification completes.
