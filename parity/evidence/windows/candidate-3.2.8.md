# Windows 3.2.8 formal release evidence

PixelDone Windows 3.2.8 is the formal successor to the immutable 3.2.7 release. It incorporates the approved Trash management, destructive confirmation, notification identity, localization, and test-reliability changes on `main`.

- A shared localized alert dialog protects recoverable and permanent destructive actions with explicit semantics, stable targets, focus containment, cancellation, and duplicate-submit prevention.
- Trash supports title, priority, and source-checklist filtering while permanent deletion remains independent from the visible filtered subset.
- Trash action buttons are accessible borderless icons and source labels no longer expose formatting placeholders.
- Direct development builds reuse an installed PixelDone executable for the Windows notification identity instead of producing a false warning.
- The release workflow builds and signs one NSIS artifact, verifies its signature and SHA-256, then publishes byte-identical assets and notes to immutable GitHub and Gitee Releases.

## Release verification

- Local verification passed Svelte diagnostics with zero errors and warnings, 38 Bun tests with 329 assertions, the production build, and the parity gate at 85.71%.
- Rust formatting, Clippy with warnings denied, unit, migration, and Windows productization tests are repeated locally and by the tag-triggered quality gate.
- The complete binary WebView2 suite passed all 25 scenarios across 11 specs, including all destructive confirmation paths, filtered and unfiltered Trash deletion, quick delete, focus restoration, narrow and RTL layouts, and the Rust update version.
- The parity gate continues to preserve all authorized cross-device cloud scenarios as incomplete; they are not represented as verified.
- Exact public installer size, SHA-256, updater signature, manifest URLs, and provider-side asset hashes are verified by CI and the publishing scripts rather than predeclared in source evidence.
