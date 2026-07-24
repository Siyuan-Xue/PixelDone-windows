# Android 3.3.0 release-candidate evidence

PixelDone Android 3.3.0 adds Markdown export to the configurable Dock and an Android launcher widget.

- The export dialog offers simple and detailed copies, includes completed and unfinished tasks in the current sort order, and copies a Markdown heading plus task checkboxes.
- Detailed export adds localized priority, due-date, and repeat metadata.
- Each widget instance stores one normal checklist selection, renders unfinished tasks responsively, opens that checklist in the app, and completes a task from its checkbox.
- Unit tests cover Markdown escaping, task inclusion and order, Dock action rules, widget checklist filtering, responsive row limits, and unfinished-task ordering.
- The formal Android release workflow runs unit tests, lint, an API 36 connected test, the signed release build, APK/signing verification, and GitHub/Gitee publication.
