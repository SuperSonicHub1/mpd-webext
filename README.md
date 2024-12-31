# mpd-webext

Small web server that glues [an MPD daemon](https://mpd.readthedocs.io/en/latest/) and [yt-dlp](https://github.com/yt-dlp/yt-dlp) together so one can easily download and listen to music from the internet.

## Setup
- Install yt-dlp and FFmpeg.
- Get a valid Rust toolchain running on your machine.
- Change constants in `src/config.rs` to fit your config.
- Run `cargo install --path .` to install the crate to your `PATH`.
- Run `mpd-webext`. Consider using something like systemd to run the binary after MPD launches on startup.
