# Windows 3.1.1 verification evidence

- The local-only `enhanced_xhigh_alarm_enabled` setting is persisted by SQLite migration `0005` and defaults to disabled.
- Scheduled-toast XML tests verify that default XHIGH notifications contain neither the alarm scenario nor looping audio, while explicit opt-in restores both and retains protocol activation.
- Installed-build notification identity uses stable AUMID `com.milesxue.pixeldone.windows`, a start-menu shortcut AUMID, and a Stub Toast Activator CLSID. Direct development binaries report that installation is required instead of rewriting the shortcut.
- Realtime tests cover the WebSocket URL, four-table join payload and owner filter, change-event classification, heartbeat, access-token refresh reconnect, and bounded exponential retry.
- Realtime events only invalidate state; the existing serialized transactional pull remains the sole remote merge path. Windows has no fixed-interval synchronization polling loop.
- Local mutations, Realtime join/rejoin, and successful manual or event-driven synchronization wake an independently serialized reminder reconciliation, which rebuilds scheduled notifications from the complete snapshot.
- Live two-device propagation and installed Windows notification delivery remain explicit pre-release acceptance gates in `docs/release-3.1.1.md`.
