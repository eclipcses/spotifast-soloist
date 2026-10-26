# Soloist playback in this personal Windows build

This build keeps Spotifast's interface and adds an optional Soloist playback
process. It controls Soloist through Spotifast's existing Spotify Connect
device controls. It does not bundle Soloist, change its DRM, or require a
terminal for everyday use.

## Setup

Close the ordinary Spotifast copy while testing this one, so two clients do
not compete for playback or distort a memory comparison.

1. Download and extract Spotify's **x86_64 Windows** Soloist archive from
   [Spotify's downloads page](https://developer.spotify.com/documentation/soloist/reference/downloads-and-updates).
2. Generate your own key at [Spotify's Soloist dashboard](https://developer.spotify.com/dashboard/soloist).
3. Sign into this Spotifast build normally for library and search access.
4. Open **Settings → Playback → Soloist playback**. Use **Browse...** to choose
   `soloist.exe`, then enter your Soloist key and a device name.
   Choose **Save and start**.
5. For initial pairing, open an official Spotify app on the same network,
   using the same account, and select your Soloist device in its device picker.
6. Select that device in Spotifast's device picker. Browse and control music
   from Spotifast. The official desktop Spotify app can then be closed.

Your Soloist setup is saved in Windows Credential Manager, separately for
this Spotifast profile. The next start of this build starts Soloist quietly.
Closing to the tray keeps playback running; exiting Spotifast stops its child
Soloist process. **Stop and forget setup**, or signing out, stops it and
removes its saved key and pairing. It does not change separately installed
Soloist processes. Setup errors appear in the app.

## Limits

- This is a personal fork, not an upstream Spotifast release. Builds from
  0.14.3 onward check this fork's signed releases for updates. Installing an
  original-project installer manually will replace the integration.
- Soloist plays the audio; Spotifast's equalizer, audio visualizers,
  local-file playback, audio-output selection and librespot audio settings
  do not affect it. Choose the Windows default audio output instead.
  Soloist also does not inherit Spotifast's proxy configuration.
- Spotifast's Web API still supplies library, search and playback controls.
  From upstream 0.14.2, this requires your own Spotify app's Client ID, which
  the sign-in screen helps you create. It is separate from the Soloist key.
  Its existing account, permission, Premium and rate-limit requirements apply.
  Session-only library features may be unavailable because the librespot
  playback session is stopped while Soloist is selected.
  Spotifast treats Soloist as a remote Connect device even on this PC, so
  controls that require Spotifast's own local player, such as inserting at
  the front of its queue, remain unavailable.
- Soloist's builds expire after 90 days. Replace the executable with a newer
  official build and choose **Start saved setup**. An expiry exit is reported
  in Spotifast. A different executable path needs **Save and start** again.
- Soloist is a separate process; memory comparisons must include both it and
  Spotifast. No RAM reduction has been established by this change.
- The key is not saved to Spotifast's JSON files or logs. Spotify requires
  it as a process argument when starting Soloist. Only run this on a Windows
  account you trust.

Build with `cargo +stable build --release --locked --no-default-features`.
The no-default-features build omits MilkDrop, which cannot read Soloist's
audio anyway. The normal Spotifast/librespot path remains available when
no Soloist setup has been saved.
