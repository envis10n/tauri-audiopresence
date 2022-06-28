# tauri-audiopresence

A Tauri-based app utilizing [AudioPresence](https://github.com/envis10n/audiopresence) for setting Discord Rich Presence based on currently playing media.

## Installation

Currently no builds will be published, but it should be usable on both Linux and Windows.

- Clone the repo.
- In the root of the repo, install dependencies with `yarn install`.
  - *NOTE: Linux dependencies required for Tauri can be found [here](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux).*
- From the root, use `yarn tauri dev` to start the application and watch for changes. This will recompile after code changes and restart.
  - *For a regular build, use `yarn tauri build`. This will build an optimized release build which can be found in the `target` directory.*

### Usage

The application will sit in the system tray, with only an option to quit.

When a change occurs with the currently playing media, it will update your Discord activity status with the new metadata.

That's it. Really.