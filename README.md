# lure

lure is an improved fork of [lr](https://codeberg.org/arslee07/lr), a small _daemon_ that sets the currently playing track on Last.fm, ListenBrainz (and other platforms) as Revolt user status.

[![CI Status](https://img.shields.io/github/actions/workflow/status/tuhanayim/lure/ci.yaml?style=flat-square&label=CI)](https://github.com/tuhanayim/lure/actions/workflows/ci.yaml)
[![CD Status](https://img.shields.io/github/actions/workflow/status/tuhanayim/lure/cd.yaml?style=flat-square&label=CD)](https://github.com/tuhanayim/lure/actions/workflows/cd.yaml)
[![Latest Release](https://img.shields.io/github/v/release/tuhanayim/lure?style=flat-square)](https://github.com/tuhanayim/lure/releases/latest)

## Installation

lure is tested on Linux and Windows, and is expected to work on macOS too. To install, simply run:

```sh
cargo install --git https://github.com/tuhanayim/lure
```

lure currently has two platform features enabled by default. Unused platforms can be disabled by using `--no-default-features` and `--features`.

For example, to build lure only with Last.fm platform feature, simply append `--no-default-features --features lastfm` to the build command above:

```sh
cargo install --git https://github.com/tuhanayim/lure --no-default-features --features lastfm
```

All available and current default platform features can be checked from [Cargo.toml](Cargo.toml) file.

## Configuration

lure uses a config file and environment variables for configuration. To get help about how to use CLI, simply run:

```sh
lure help
```

To generate a config file for lure, simply run:

```sh
lure config generate
```

> [!NOTE]
> Config fields can be different depending on which platform features are used to build lure. For example, disabling `lastfm` feature will not generate and use Last.fm specific configuration options.

If you'd like to configure options using environment variables, here's the table of current environment variables:

| Variable Name                      | Description                                       | Default Value                | Is Required | Platform Feature |
| ---------------------------------- | ------------------------------------------------- | ---------------------------- | ----------- | ---------------- |
| `LURE_REVOLT_SESSION_TOKEN`        | Revolt session token                              | None                         | Yes         | \*               |
| `LURE_STATUS_TEMPLATE`             | Status template to show when listening            | 🎵 %ARTIST% %NAME%           | No          | \*               |
| `LURE_STATUS_IDLE`                 | Status to show when not listening anything        | None                         | No          | \*               |
| `LURE_LASTFM_USER`                 | Last.fm username to check listens                 | None                         | Yes         | `lastfm`         |
| `LURE_LASTFM_API_KEY`              | Last.fm API key to fetch listens                  | None                         | Yes         | `lastfm`         |
| `LURE_LASTFM_CHECK_INTERVAL`       | Checking listening status interval in seconds     | 12                           | No          | `lastfm`         |
| `LURE_LISTENBRAINZ_USER`           | ListenBrainz username to check listens            | None                         | Yes         | `listenbrainz`   |
| `LURE_LISTENBRAINZ_API_URL`        | Custom ListenBrainz API URL to check listens from | https://api.listenbrainz.org | No          | `listenbrainz`   |
| `LURE_LISTENBRAINZ_CHECK_INTERVAL` | Checking listening status interval in seconds     | 12                           | No          | `listenbrainz`   |
