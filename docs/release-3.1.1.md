# PixelDone for Windows 3.1.1

## Delivered scope

- Windows Scheduled Toast queue replaces app-lifetime polling as the primary reminder mechanism.
- Every priority, including XHIGH, defaults to a standard bottom-right Windows notification. A local-only Settings switch optionally enables the Windows alarm scenario and looping audio for XHIGH.
- STOP, SNOOZE 10 MIN, and open actions use the private `pixeldone-reminder://` protocol with local task validation.
- Reminder schedule, snooze, and delivery metadata are persisted in SQLite and reconciled for a rolling 12-month horizon.
- Installed builds register a stable process/shortcut AUMID and Stub Toast Activator CLSID. Notification settings or registration failures are surfaced in the UI without an intrusive-window fallback.
- Supabase Realtime invalidates local state for checklists, todos, settings, and tombstones, then reuses the transactional pull RPC after debounce. No fixed-interval synchronization polling remains.
- Login/session recovery, local mutations, Realtime join/rejoin, and successful manual or event-driven sync immediately rebuild the local Windows notification queue from the latest snapshot.
- Startup defaults to enabled once, can be disabled in Settings, and is not silently re-enabled after external/user disablement.
- Update checks run automatically without automatic download; Settings includes manual check, install, and progress reporting.
- Storage & privacy reports executable, SQLite, attachment, cache, log, WebView2, legacy database, and credential locations.
- The executable is `PixelDone.exe`; the NSIS installer is `PixelDone_3.1.1_x64-setup.exe` and uses current-user product identity.
- Windows raster/ICO/ICNS assets are generated from the Android launcher icon geometry and colors.
- Formal CI requires both Authenticode certificate material and the Tauri updater signing key.

## Release gates

- [x] `bun run check`
- [x] `bun test`
- [x] `cargo fmt --check`
- [x] `cargo check --all-targets`
- [x] `cargo test`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `bun run build`
- [x] `bun run parity:check` (100.00%)
- [x] Local unsigned validation bundle emits `PixelDone.exe` and `PixelDone_3.1.1_x64-setup.exe`.
- [ ] Build x64 NSIS installer with production cloud configuration.
- [ ] Verify Authenticode publisher, timestamp, updater signature, SHA-256, clean install, in-place reinstall, upgrade, and uninstall.
- [ ] Validate standard and opt-in enhanced XHIGH notifications while PixelDone is closed, including notification-center retention, STOP and SNOOZE protocol activation, and the Windows-disabled state.
- [ ] Validate two live devices: Realtime propagation in seconds, reconnect catch-up after a forced disconnect, and immediate schedule replacement/removal on the receiving Windows device.

## Accepted release risks

- The configured Supabase endpoint remains HTTP, so Realtime uses `ws://`. Credentials, access tokens, and synchronized data do not have transport confidentiality, integrity, or authenticated-server protection.
- Windows may discard a scheduled notification when its delivery time was missed by more than approximately five minutes. The 15-minute reconciliation repairs future recurring schedules but intentionally does not replay stale one-time reminders.

The unchecked artifact and OS integration gates must be completed before publishing `v3.1.1`. The release workflow intentionally blocks a formal build when Authenticode secrets are unavailable.

The 2026-07-12 local unsigned 3.1.1 validation installer is 7,003,666 bytes with SHA-256 `604FC59C8DB3994140149C6AA88DCAC42D4B37D926DEF2A8E4E2B24FF4658AC1`. It is non-publishable and intentionally reported by Windows as unsigned.
