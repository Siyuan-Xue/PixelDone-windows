# Windows 3.2.2 candidate verification evidence

PixelDone Windows 3.2.2 is an emergency startup hotfix for the 3.2.1 release. It preserves user data and the 3.2.1 UI behavior.

- The installed 3.2.1 release was reproduced exiting with Rust code 101 in approximately 600 ms.
- Captured startup diagnostics identified SQLx migration checksum drift: migration 7 had already been applied with its 3.2.0 LF checksum but was embedded with CRLF by the 3.2.1 GitHub Windows build.
- `.gitattributes` now gives migration 7 a specific `eol=lf` rule after the general Windows migration rule.
- Bun and Rust release-invariant tests assert the exact deployed SHA-384 checksum `9606CFB487A71F9661010578B3E0D527A5A44B0B9D554650F1E6D524D086594560669F8548893FC6E85DECC900BE1CC6`.
- The GitHub release parity job now runs the Bun test suite before the signed Windows build can start.
- A disposable copy of the affected installed database upgraded from migration 7 to migration 8 successfully; SQLite integrity remained `ok` and the existing 420 px sidebar setting was preserved.
- Bun passed 19 tests with 66 expectations; Svelte diagnostics reported 0 errors and 0 warnings; all 23 Rust unit tests, migration tests, 7 Windows productization tests, Clippy, production build, and all 10 native Windows E2E spec files passed.
- The local formal x64 NSIS installer is 5,395,723 bytes with SHA-256 `376524DAED29992ACEFF0616C5F0558E43A0F2E05722F3DDCC19B2CAA54D6F1A` and a 420-byte Tauri updater signature verified against the embedded public key.
- The 3.2.2 installer returned exit code 0 while overwriting the affected installed 3.2.1 application. The installed file reports version 3.2.2.
- The installed 3.2.2 application opened against the original user database and remained responsive beyond the 10-second observation window, replacing the reproducible 3.2.1 exit after approximately 600 ms.
- The actual installed database then reported successful migration 8, SQLite integrity `ok`, and the preserved 420 px sidebar setting. A pre-upgrade backup was retained in the application data `backups` directory.
- GitHub release and cloud-built artifact verification results are recorded after publication.
- The six pre-existing cross-device cloud rows remain unverified and are not represented as passed.
