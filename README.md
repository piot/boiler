# 🚂 Boiler

Build your game without leaving your command line.

This tiny CLI takes your prebuilt binaries from GitHub releases, downloads game content and prepares depots.
It’s perfect for when you’re too busy making your game to remember what the .vdf files even does.

## ini file

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

    # what directories to copy to the build directory
    directories ["assets", "scripts", "packages"] 
}
```

## Usage

```sh
boiler --ini your_app.boiler.yini
```
