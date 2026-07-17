# PixelDone for Windows Release Specification

## Authority and version contract

- GitHub repository `Siyuan-Xue/PixelDone-windows` is the source repository and primary release authority.
- Gitee repository `milesxue/pixel-done-windows` is a Git mirror and secondary release channel.
- A formal release uses one immutable strict `vX.Y.Z` tag reachable from `main`.
- The tag version must equal `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Existing tags, release notes, and release assets must never be overwritten or clobbered.

## Build once, publish twice

- GitHub Actions is the only formal build authority. Gitee must not run an independent package build.
- CI runs the complete Bun, Svelte, parity, Rust formatting, Clippy, and Rust test gates before packaging.
- CI builds and updater-signs one x64 NSIS installer, then generates both provider-specific updater manifests from that same installer and signature.
- The identical staged files are published first to GitHub Release and then to Gitee Release.
- A retry may add a missing asset only after verifying all existing metadata and assets. Any conflict fails closed.

## Formal release assets

Every GitHub and Gitee Release must contain exactly the same bytes for:

- `PixelDone_X.Y.Z_x64-setup.exe`
- `PixelDone_X.Y.Z_x64-setup.exe.sig`
- `PixelDone_X.Y.Z_x64-setup.exe.sha256`
- `latest.json`
- `latest-gitee.json`

`latest.json` points to the GitHub installer. `latest-gitee.json` points to the Gitee installer. Both manifests must declare the same version and Tauri updater signature.

## Secrets

- `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` sign updater artifacts.
- `GITEE_ACCESS_TOKEN` creates the Gitee Release and uploads its attachments. It requires repository and release write access and must remain a GitHub Actions secret.
- Supabase build configuration uses the existing repository secrets and must not be printed.

## Gitee mirror recovery

- If the Git mirror tag is delayed, the workflow waits for the exact Gitee tag and commit before creating a Release.
- If GitHub publishing succeeds but Gitee publishing fails, manually dispatch `Release Windows x64` with the immutable tag and `mirror_existing=true`.
- Recovery downloads the existing GitHub Release, verifies its checksum and metadata, and mirrors those files without rebuilding.

## Workflow handoff

- Confirm that the release workflow was triggered for the intended immutable tag and entered the expected build or recovery path.
- When the automated workflow has been exercised repeatedly and is considered stable, a Codex task may end while the workflow is still running if no manual work remains other than checking its final conclusion.
- The handoff must include the workflow URL and its current status. Do not report the workflow or Release as successful before GitHub records a successful conclusion.
- A known failure, conflicting asset, or partial Release that still requires recovery is remaining manual work and must not be treated as a monitoring-only handoff.

## Application updater fallback

- The installed application checks the signed GitHub updater endpoint first.
- When GitHub discovery fails, it queries the public Gitee Releases API and selects only a strict stable version newer than the installed version.
- When a GitHub update is discovered but its download or installation fails, fallback is restricted to the exact same version on Gitee.
- Every provider uses the embedded Tauri public key to verify the updater signature. A Gitee Release without a valid `latest-gitee.json`, matching version, asset, and signature is rejected.
