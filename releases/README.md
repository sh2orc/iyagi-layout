# Release binaries

Installers for direct download from the README. Naming: `IYAGI.Layout_<version>_<arch>.dmg`.
Each entry's SHA-256 is recorded in the README download section and in its GitHub release notes.
From 0.1.1 on, the DMG is attached to the [GitHub release](https://github.com/sh2orc/iyagi-layout/releases)
instead of being committed here; 0.1.0 stays in this folder so its existing links keep working.

## Local macOS signing and notarization

`npm run tauri -- build` prefers the sole valid Developer ID Application certificate in the
login keychain and enables hardened runtime (`scripts/tauri.mjs`, shared with iyagi-term).
Then:

```sh
xcrun notarytool submit "target/release/bundle/dmg/IYAGI Layout_0.1.1_aarch64.dmg" \
  --keychain-profile iyagi-notary --wait
xcrun stapler staple "target/release/bundle/dmg/IYAGI Layout_0.1.1_aarch64.dmg"
```

Staple the `.app` before packaging if you build the DMG yourself. Never enable App Sandbox:
the Accessibility API does not work inside it.

## Published releases

| Version | File | SHA-256 | Notarization submissions |
|---|---|---|---|
| 0.1.0 | `IYAGI.Layout_0.1.0_aarch64.dmg` | `a2998196ec2e2caefcadeca17e27b8493a05183d472a2bebdc6301925eaed341` | app `d91a2fd4-3936-4859-acae-5c2184d93628`, DMG `b7c81ba1-5768-4f11-8fab-38bc732ddbf6` (both Accepted) |
| 0.1.1 | GitHub release `v0.1.1`: `IYAGI.Layout_0.1.1_aarch64.dmg` (not committed) | `2fe34193a8525e20b82231af913726b19c030a6e16cdbd5c691890d9126deade` | app `73c63824-f4c9-466a-8ae9-eec0459492e6`, DMG `306f63c6-047d-4dcc-9608-3934ad764c7d` (both Accepted) |

Procedure used for 0.1.1: `npm run tauri -- build --bundles app` (the app alone, so the Finder-based
DMG step is skipped) → zip the `.app` with `ditto` and submit →
`stapler staple` the app → `hdiutil create` a DMG (app + `/Applications` link) from the stapled app →
`codesign` the DMG → submit and `stapler staple` the DMG. `spctl` then reports
`source=Notarized Developer ID` for both. If a previous build left disk images mounted under
`/Volumes/dmg.*`, detach them first or the Tauri DMG step fails.
