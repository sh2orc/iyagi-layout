# Release binaries

Installers for direct download from the README. Naming: `IYAGI.Layout_<version>_<arch>.dmg`.
Each entry's SHA-256 is recorded in the README download section.

## Local macOS signing and notarization

`npm run tauri -- build` prefers the sole valid Developer ID Application certificate in the
login keychain and enables hardened runtime (`scripts/tauri.mjs`, shared with iyagi-term).
Then:

```sh
xcrun notarytool submit "target/release/bundle/dmg/IYAGI Layout_0.1.0_aarch64.dmg" \
  --keychain-profile iyagi-notary --wait
xcrun stapler staple "target/release/bundle/dmg/IYAGI Layout_0.1.0_aarch64.dmg"
```

Staple the `.app` before packaging if you build the DMG yourself. Never enable App Sandbox:
the Accessibility API does not work inside it.

## Published releases

| Version | File | SHA-256 | Notarization submissions |
|---|---|---|---|
| 0.1.0 | `IYAGI.Layout_0.1.0_aarch64.dmg` | `a2998196ec2e2caefcadeca17e27b8493a05183d472a2bebdc6301925eaed341` | app `d91a2fd4-3936-4859-acae-5c2184d93628`, DMG `b7c81ba1-5768-4f11-8fab-38bc732ddbf6` (both Accepted) |

Procedure used for 0.1.0: `npm run tauri build` → zip the `.app` with `ditto` and submit →
`stapler staple` the app → `hdiutil create` a DMG (app + `/Applications` link) from the stapled app →
`codesign` the DMG → submit and `stapler staple` the DMG. `spctl` then reports
`source=Notarized Developer ID` for both. If a previous build left disk images mounted under
`/Volumes/dmg.*`, detach them first or the Tauri DMG step fails.
