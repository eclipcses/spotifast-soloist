# Maintaining Spotifast Soloist

This fork keeps upstream's native interface and adds a Windows Soloist
process. It supports Windows x86_64 releases without MilkDrop. Upstream's
Linux, Nix, Flatpak and macOS packaging files are retained for reference,
but this fork does not publish or validate those packages.

## Getting an upstream update

An update in the original repository does not replace your installed fork.
Merge it here first, retaining `src/soloist.rs`, its app/backend/settings
integration, the fork updater repository, and the fork signing key.

1. Fetch `https://github.com/crmne/spotifast.git` as `upstream`.
2. Merge `upstream/main` into this fork's `main`, resolving conflicts while
   retaining Soloist. A merge commit for this synchronization is permitted;
   do not rewrite the published main branch.
3. Choose a stable numeric fork version higher than the last fork release.
   Update Cargo.toml, this package's Cargo.lock version, the metainfo release
   entry, and `packaging/release-notes/vVERSION.md`. Fork versions are
   independent of upstream; record the upstream version in the release notes.
4. Push main and wait for **Soloist Windows / Windows checks** to pass.
5. Tag that tested commit `vVERSION` and push the tag. The same workflow
   validates the tagged commit again, builds Windows binaries from scratch,
   packages the portable ZIP, MSI and setup EXE, signs and verifies the
   checksums, and publishes the release only after all of those succeed.

Future changes may be handled in this chat; no local compilation is needed
to download the resulting GitHub release. CI still takes time to build.
Upstream updates are not merged or published automatically on a schedule.

## Release signing and workflow setup

`assets/soloist-update-public-key.hex` is the public Ed25519 publisher key.
Its private PEM key is the repository Actions secret
`SPOTIFAST_UPDATE_SIGNING_KEY`. Never commit, print, replace, or rotate the
private key during an ordinary upstream merge. Do not substitute upstream's
publisher key: this fork cannot sign releases with upstream's private key.
`packaging/soloist-release.cjs` produces the raw 64-byte signature required
by fastframe-update and verifies it against the committed public key.

Only `.github/workflows/soloist-windows.yml` should be enabled on this fork.
Disable inherited upstream workflows in the repository's Actions settings;
they target upstream's platforms, publishing infrastructure and policies.
After an upstream merge, check that this is still the only enabled workflow.

No Soloist executable, Spotify token, API key, pairing directory, or local
Spotifast profile belongs in Git or in a release archive. Each listener
downloads Soloist from Spotify and supplies their own key through the UI.
Updating the app preserves its profile and saved pairing. Spotify's Soloist
build expiry is independent of Spotifast app updates.

## First installation

The earlier locally built app still checks upstream for updates. Replace it
once with this fork's first signed release, then future app update checks
stay on this fork. Keep the installed profile location to retain your setup.

Authenticated playback and memory use need a real user account and must not
be claimed from compilation, synthetic tests or a CI pass alone.
