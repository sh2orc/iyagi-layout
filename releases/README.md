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
