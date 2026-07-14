# Windows 3.2.3 candidate verification evidence

PixelDone Windows 3.2.3 is the formal startup hotfix for the broken 3.2.1 release. It preserves user data and all 3.2.1 UI behavior.

- The installed 3.2.1 release was reproduced exiting with Rust code 101 in approximately 600 ms because SQLx rejected migration 7 checksum drift.
- Migration 7 is fixed to LF with the exact 3.2.0 SHA-384 checksum `9606CFB487A71F9661010578B3E0D527A5A44B0B9D554650F1E6D524D086594560669F8548893FC6E85DECC900BE1CC6`.
- Bun and Rust regression tests assert both the Git attribute override and the deployed checksum.
- A disposable copy of the affected installed database upgraded from migration 7 to migration 8 with SQLite integrity `ok` and the existing 420 px sidebar setting preserved.
- A locally signed hotfix installer overwrote 3.2.1 with exit code 0; the installed app remained responsive beyond 10 seconds against the original database, which then reported migration 8 and integrity `ok`.
- The v3.2.2 tag was intentionally left immutable after its strengthened cloud gate exposed a fresh-checkout test prerequisite: generated SvelteKit modules did not exist before `bun test`.
- The package test command now runs `svelte-kit sync` before Bun, and the GitHub parity gate invokes that reproducible package test command.
- A fresh v3.2.2 Git checkout passed all 19 Bun tests after the same explicit SvelteKit sync, confirming the repaired cloud-gate order independently of the working tree.
- For 3.2.3, Bun passed 19 tests with 68 expectations; all 23 Rust unit tests, migration tests, 7 Windows productization tests, Clippy, production builds, and all 10 native Windows E2E spec files passed.
- The local formal 3.2.3 x64 NSIS installer is 5,394,334 bytes with SHA-256 `2E483900DD3E530561E6B020BB6C10CA78EC24B82BCFA65DC90A40BBA4EF72CB` and a 420-byte Tauri updater signature verified against the embedded public key.
- The 3.2.3 installer overwrote the working 3.2.2 candidate with exit code 0. The installed file reports version 3.2.3, and the installed application remained responsive beyond the 10-second observation window against the original user database.
- The six pre-existing cross-device cloud rows remain unverified and are not represented as passed.

## Published release verification

- GitHub Actions run `29297164894` completed successfully for source commit `d96f55d5bd332e78f809aa21ff754d416cd628e6` and immutable tag `v3.2.3`.
- The public, non-draft, non-prerelease `PixelDone v3.2.3` release contains the four expected assets: x64 NSIS installer, detached updater signature, SHA-256 file, and `latest.json`.
- The cloud-built installer is 5,396,845 bytes with SHA-256 `92B63D9934705C12932C1C370906A34BD3C12C5ED3A447E52567FA73153C4A7E`; the uploaded SHA file and GitHub asset digest match exactly.
- The public `latest.json` reports version 3.2.3, the v3.2.3 installer URL, and a 420-character updater signature.
- The downloaded cloud installer and signature passed the embedded-public-key integrity test. That cloud installer then overwrote the local candidate with exit code 0, and the installed 3.2.3 application remained responsive beyond 10 seconds against the original user database.
