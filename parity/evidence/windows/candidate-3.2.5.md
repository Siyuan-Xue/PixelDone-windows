# Windows 3.2.5 formal release evidence

PixelDone Windows 3.2.5 is the formal successor to the published, immutable 3.2.4 baseline.

- Windows typography now mirrors the Android semantic hierarchy with bundled Source and Noto variable fonts, real 400/500/600/700 weights, serif display roles, sans-serif UI roles, and clay section emphasis.
- User-authored checklist titles, todo titles, and conflict values are rendered by Unicode script run so changing the UI locale does not change the saved text's typeface.
- Native language labels carry their own language and direction metadata instead of inheriting the active UI font.
- The signed-out Account row opens a focused modal instead of expanding an inline form in Settings.
- Sign in and sign up use lightweight text tabs with a clay underline, local validation, password visibility, focus containment, Escape/backdrop dismissal, and trigger-focus restoration.
- Authentication errors stay inside the dialog. Closing a pending request cannot leak an obsolete error into the next dialog session.

## Release verification

- Bun passed 25 tests with 93 expectations; Svelte reported 0 errors and 0 warnings; the production frontend build completed.
- All 11 native Windows E2E spec files passed against the Tauri/WebView2 binary. The 18 scenarios cover authentication, bootstrap, checklists, cloud boundary, local images, minimum-window layout, reminders, multilingual Settings/RTL, todos, Trash, and update version reporting.
- Computed font assertions passed for all supported locale families. `auth-english.png`, `auth-chinese.png`, `auth-arabic.png`, and `settings-arabic.png` record the Source, Noto SC, Noto Arabic/Naskh, and RTL visual evidence.
- Rust passed 28 unit tests, 3 migration tests, 7 Windows productization tests, formatting, and Clippy with warnings denied. The formal updater-signature integrity test also passed against the generated 3.2.5 package.
- The parity gate authorizes the formal release at 85.71% while preserving all incomplete rows as unverified. Android remains on the existing 3.2.3 repository baseline and was not modified by this release.
- The local formal x64 NSIS package is 29,045,084 bytes with SHA-256 `CF2DD40CD159BB1AB6366C7BFF3FF21DC051CF64ED03FCCA7A2D6F498CEF0995`; its 420-byte Tauri updater signature has SHA-256 `CA79946543709B25F786242B336A150AEB4F7DD5B9D005DEA5FB828711A1841D`, and executable metadata reports product and file version 3.2.5. Public artifacts are checked separately after the immutable GitHub Release upload.
- Native installer overwrite, installed-app smoke testing, and six dual-device cloud scenarios were not reproduced locally. They remain explicitly authorized release exceptions and must not be inferred from this document.
