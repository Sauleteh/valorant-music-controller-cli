# Valorant Music Controller (CLI)
Automatically pause/play and control the volume of your music depending on the state of the game you are in on Valorant. It should work on all music platforms, including YouTube, Spotify (browser and app), etc.
It currently supports three states on Valorant to determine the volume:
1. Not in game (No active game, choosing agent, map is loading)
2. In game - Preparing (Buy phase)
3. In game - Playing (Alive, playing the round)

This app uses the log file of the game to detect state changes in the game, so there aren't any restriction to use this program.

## Instructions
To get the app, just download the .exe <a href="https://github.com/Sauleteh/valorant-music-controller-cli/releases/latest">here</a> or build the source code with `cargo update` and `cargo build --release` (you will need rustup to compile).

To use it, just open the executable and choose the process that will be used to control the volume. It's recommended to close the app by pressing CTRL+C to reset the process volume to the value that it had before the app started.
When the volume of the process is set 0, it will pause the media automatically.

## Graphical version
I also made this app with GUI, you can check it <a href="https://github.com/Sauleteh/valorant-music-controller-gui">here</a>, it's an upgraded version.
