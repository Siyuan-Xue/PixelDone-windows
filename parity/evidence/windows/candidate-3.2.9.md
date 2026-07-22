# Windows 3.2.9 formal release evidence

PixelDone Windows 3.2.9 is the formal successor to the immutable 3.2.8 release. It fixes the task deletion confirmation dialog being rendered beneath the task editor.

- The destructive-confirmation backdrop has an explicit stacking level above the task-editor backdrop.
- The editor remains open behind the confirmation so cancellation can restore focus to the Delete button without losing draft state.
- The todo WebView2 regression scenario verifies both computed stacking order and pointer hit-testing at the confirmation button before exercising cancel and confirm flows.
- No persistence, synchronization, reminder, image, authentication, or installer behavior changes in this patch release.

## Release verification

- Local verification covers Svelte diagnostics, Bun tests, the production build, the parity gate, Rust formatting, Clippy with warnings denied, Rust tests, and the complete binary WebView2 suite.
- The parity gate continues to preserve all authorized cross-device cloud scenarios as incomplete; they are not represented as verified.
- Exact public installer size, SHA-256, updater signature, manifest URLs, and provider-side asset hashes are verified by CI and the publishing scripts rather than predeclared in source evidence.
