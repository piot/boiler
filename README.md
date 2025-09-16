# 🚂 Boiler

Build your game without leaving your command line.

This tiny CLI grabs your prebuilt binaries from GitHub releases, downloads your game
content and prepares depots. It’s perfect for when you’re too busy making your
game to remember what the `.vdf` files even does.

## Usage

```sh
boiler your_app.boiler.yini
```

Optionally set a Steam branch to go live on upload:

```sh
boiler your_app.boiler.yini --live-branch beta
```

### CLI arguments

Positional:

- **INI**: Path to your `.boiler.yini` configuration. Required.

- **--steam-redist <PATH>**: Path to Steam SDK redistributables. Default:
  `steam_redist`.
  - Expected subdirectories: `osx/`, `win64/`, `linux64/` (these are copied into
    each platform’s `binaries/` dir).

- **--build-dir <PATH>**: Output staging directory. Default: `build`.

- **--temp-dir <PATH>**: Temporary working directory. Default: `temp`.

- **--live-branch <STEAM_BRANCH>**: Optional Steam branch
  to set live during upload. Safety check refuses `default` and `public`.

- **--keep-build-dir**: Do not delete `--build-dir` at startup. Still cleans `--temp-dir`.

- **--targets <LIST>**: Comma-separated list of parts to process.
  Accepted values: `content`, `mac` (aliases: `macos`, `osx`), `linux`, `windows`.
  Default (when not provided): all of them.
  - Examples:
    - `--targets content` (only content)
    - `--targets mac,windows` (macOS and Windows binaries)

### Important behavior

- ⚠️ **Destructive clean (default):** At startup it deletes the entire `--build-dir`
  and `--temp-dir` if they exist. Do not point these to directories with
  anything you want to keep. Use `--keep-build-dir` to preserve `--build-dir`.

- Generated files include:
  - `app_build_<APP_ID>.vdf`

  - `depot_content.vdf`, `depot_macos.vdf`, `depot_linux.vdf`,
    `depot_windows.vdf`

  - Build info files (see below)

After the build, you can upload with SteamCMD (verify `build/` first):

```sh
steamcmd +login <your_steam_name> +run_app_build /absolute/path/to/build/app_build_<APP_ID>.vdf +quit
```

note: `your_steam_name` is sometimes your email.

Install SteamCMD with Homebrew or see the [SteamCMD Documentation](https://developer.valvesoftware.com/wiki/SteamCMD).

## Example ini file

```ini
steam_app_id 1234560

binaries {
    repo "game-engine/engine" # github repo
    name executable_name
    version "0.1.2"

    macos {
        depot 1234562
    }

    windows {
        depot 1234563
    }

    linux {
        depot 1234564
    }
}

content {
    depot 1234561

    repo yourgame/contents # github repo

    # the directories and files that should be 
    # copied from content
    copy [
        "assets" "data/assets",
        "scripts" "data/scripts",
        "packages" "data/packages",
        "your_game.ini" "data/",
    ]
}
```

### Notes

- `content.copy` lists what to copy from the content repo into
  `build/data/`.
- Binaries are pulled from the GitHub release assets for the specified
  repo/name/version and unpacked to `build/binaries/<platform>/`. The Steam
  redistributables from `--steam-redist` are copied into each platform
  directory.
- If you specify `--live-branch`, the branch is included in
  `app_build_<APP_ID>.vdf` so Steam sets that branch live during upload.


## Build info files

It writes small, human-readable text files describing what was built.

- **Content build info**: `build/data/buildinfo_content.txt`

  - Fields: `repo`, `commit`, `committed_at_utc`, `built_at_utc`
  - Example:
    ```
    repo: org/content-repo
    commit: 3f2e4a8
    committed_at_utc: 2025-09-13T10:11:12Z
    built_at_utc: 2025-09-13T12:34:56Z
    ```

- **Binaries build info**: `build/binaries/<platform>/buildinfo_binaries.txt`
  for each of `macos`, `windows`, `linux`
  - Fields: `repo`, `version`, `built_at_utc`
  - Example:
    ```
    repo: org/engine-repo
    version: 0.1.2
    built_at_utc: 2025-09-13T12:34:56Z
    ```
---

_Copyright 2025 Peter Bjorklund. All rights reserved._
