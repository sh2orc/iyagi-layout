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
xcrun notarytool submit "target/release/bundle/dmg/IYAGI.Layout_0.1.6_aarch64.dmg" \
  --keychain-profile iyagi-notary --wait
xcrun stapler staple "target/release/bundle/dmg/IYAGI.Layout_0.1.6_aarch64.dmg"
```

Staple the `.app` before packaging if you build the DMG yourself. Never enable App Sandbox:
the Accessibility API does not work inside it.

## Published releases

| Version | File | SHA-256 | Notarization submissions |
|---|---|---|---|
| 0.1.0 | `IYAGI.Layout_0.1.0_aarch64.dmg` | `a2998196ec2e2caefcadeca17e27b8493a05183d472a2bebdc6301925eaed341` | app `d91a2fd4-3936-4859-acae-5c2184d93628`, DMG `b7c81ba1-5768-4f11-8fab-38bc732ddbf6` (both Accepted) |
| 0.1.1 | GitHub release `v0.1.1`: `IYAGI.Layout_0.1.1_aarch64.dmg` (not committed) | `2fe34193a8525e20b82231af913726b19c030a6e16cdbd5c691890d9126deade` | app `73c63824-f4c9-466a-8ae9-eec0459492e6`, DMG `306f63c6-047d-4dcc-9608-3934ad764c7d` (both Accepted) |
| 0.1.2 | GitHub release `v0.1.2`: `IYAGI.Layout_0.1.2_aarch64.dmg` (not committed) | `74cb388b78e785cd1525f3291a319271c69eb5bdd5c5a68053e188777dc2a552` | app `2a675aad-a818-463e-b7f8-51fae869e00b`, DMG `1b54634f-4164-4ae2-a87d-68252ae12107` (both Accepted) |
| 0.1.3 | GitHub release `v0.1.3`: `IYAGI.Layout_0.1.3_aarch64.dmg` (not committed) | `e0f304822895c625bfd450bd6fe38e06aaa60598bbf75894369f980554f21a9f` | app `97f907e5-638a-43cc-819c-d2219f246365`, DMG `a58b78b0-4073-4225-bed0-33e0dae4bdbf` (both Accepted) |
| 0.1.4 | GitHub release `v0.1.4`: `IYAGI.Layout_0.1.4_aarch64.dmg` (not committed) | `4584c5f921b201398354a05ea0704c07af6466ce496d39bc77a9d522ade7693e` | app `3c6c811e-49a8-483f-b524-0e2b439a08b8`, DMG `306523f9-9a25-4cec-a645-87c177eb20c5` (both Accepted) |
| 0.1.5 | GitHub release `v0.1.5`: `IYAGI.Layout_0.1.5_aarch64.dmg` (not committed) | `ab984793d234fdb95d62d52d3303d1d440e59e0a91d6c29ad1809ce2badab55a` | app `480f1926-b008-40da-ac25-3360b46727c5`, DMG `8a8c5015-83ab-442c-9514-78e61a073cc3` (both Accepted) |
| 0.1.6 | GitHub release `v0.1.6`: `IYAGI.Layout_0.1.6_aarch64.dmg` (not committed) | `aa918f84e5d45ba7a9de8a3ee716773006aebbd61e4f2e6d2071e9641c504d01` | app `2e301512-4a43-400d-a8d0-6d736b3650c1`, DMG `f4884a0d-850c-42e2-8290-139bfce41bb9` (both Accepted) |


Procedure used for 0.1.1 through 0.1.6: `npm run tauri -- build --bundles app` (the app alone, so the Finder-based
DMG step is skipped) → zip the `.app` with `ditto` and submit →
`stapler staple` the app → `hdiutil create` a DMG (app + `/Applications` link) from the stapled app →
`codesign` the DMG → submit and `stapler staple` the DMG. `spctl` then reports
`source=Notarized Developer ID` for both. If a previous build left disk images mounted under
`/Volumes/dmg.*`, detach them first or the Tauri DMG step fails.
