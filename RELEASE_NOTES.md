# PixelDone for Windows 3.2.6

PixelDone for Windows 3.2.6 builds on the published 3.2.5 release and the subsequent checklist alignment fix. It focuses on a cleaner account flow and synchronization that recovers from normal races and transient outages without alarming the user.

## Account and Options

- Moves password changes into a focused rectangular dialog that matches sign-in and registration.
- Adds localized client validation, independent password visibility controls, focus containment, Enter/Escape/backdrop handling, pending-request isolation, and focus restoration.
- Signs out the current device after a successful password change and shows a short non-red success notice.
- Moves the manual sync action from Account to the Sync row and disables it while synchronization is active.
- Removes todo counts from ordinary checklist navigation while retaining the Trash item count.

## Synchronization reliability

- Silently reloads and retries one UI mutation after `STALE_REVISION`; repeated races and remote deletion use non-red guidance.
- Rejects old mutation results and reloads on revision gaps so delayed deltas cannot regress the WebView snapshot.
- Validates revisions before authentication cloud side effects and converges successful sign-in, sign-up, sign-out, and password changes onto the latest local snapshot.
- Retries network sync failures after 1, 2, 4, 8, 16, and 30 seconds, capped at 30 seconds until connectivity returns. New local changes and Supabase Realtime invalidations reset the backoff.
- Classifies sync problems as network retry, expired authentication, server update required, local storage error, invalid remote data, or unknown error and keeps them in the Sync row.
- Keeps the top red alert for initialization, local database, unrecoverable Windows operations, and image reading failures instead of normal cloud recovery states.

## Release status

- Version metadata, tag, updater manifest, and x64 NSIS artifact target 3.2.6.
- The formal NSIS package carries the required Tauri updater integrity signature. Authenticode remains intentionally disabled, so Windows can still identify the installer as an unknown publisher.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
