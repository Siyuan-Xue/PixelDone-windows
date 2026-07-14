# PixelDone for Windows 3.2.5

PixelDone for Windows 3.2.5 starts from the published 3.2.4 baseline and focuses on deterministic multilingual typography and a cleaner account flow.

## Typography

- Aligns Windows text roles with Android: serif for page, checklist, dialog, and section titles; sans-serif for tasks, labels, buttons, inputs, status, and supporting text.
- Promotes top-level section titles to 16/22 semibold serif text with restrained clay emphasis.
- Uses bundled Source families for English, French, Russian, and Spanish; Noto SC for Simplified Chinese; and Noto Arabic/Naskh Arabic for Arabic.
- Renders user-authored checklist names, todo titles, and conflict values by Unicode script run so their typeface does not change with the UI locale.
- Gives every native language selector label its own language and direction metadata.

## Account dialog

- Removes the always-visible sign-in/register form from Settings.
- Opens a focused, rectangular account dialog from the Account row's login action.
- Replaces the filled segmented switch with lightweight text tabs and a 2 px clay underline.
- Adds password visibility, localized client validation, inline authentication errors, focus containment, Escape/backdrop dismissal, and trigger-focus restoration.
- Keeps pending authentication results safe when the dialog is closed and prevents obsolete errors from leaking into a later session.

## Release status

- Version metadata, tag, updater manifest, and the x64 NSIS artifact target 3.2.5.
- The formal NSIS package carries the required Tauri updater integrity signature. Authenticode remains intentionally disabled, so Windows can still identify the installer as an unknown publisher.
- Existing cleartext HTTP/WS deployment risk remains unchanged. Six cross-device cloud scenarios remain explicitly authorized release exceptions and are not represented as verified.
