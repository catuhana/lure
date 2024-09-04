# lure

[![CI Status](https://img.shields.io/github/actions/workflow/status/catuhana/lure/ci.yaml?style=flat-square&label=CI)](https://github.com/catuhana/lure/actions/workflows/ci.yaml)
[![CD Status](https://img.shields.io/github/actions/workflow/status/catuhana/lure/cd.yaml?style=flat-square&label=CD)](https://github.com/catuhana/lure/actions/workflows/cd.yaml)
[![Latest Release](https://img.shields.io/github/v/release/catuhana/lure?style=flat-square)](https://github.com/catuhana/lure/releases/latest)

Lure is an improved fork of [lr](https://codeberg.org/arslee07/lr), a small process that sets the currently playing track on Last.fm, ListenBrainz (and other future platforms, PRs welcome!) as Revolt user status.

> [!WARNING]
> Version 1 contains big changes for configurating lure. If you were on previous versions, check [configuration](#configuration) section.

## Install

> [!IMPORTANT]
> lure is tested on Linux and Windows, and is expected to work on macOS too.

```sh
cargo install --git https://github.com/catuhana/lure
```

## Run

> [!IMPORTANT]
> Reading the [configuration](#configuration) first is highly recommended.

To run lure, run:

```sh
lure start
```

> [!TIP]
> By default, lure logs useful information to the console. If you'd want to see other log levels, use the `RUST_LOG` environment variable. Check [`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) documentation from [`tracing-subscriber`](https://docs.rs/tracing-subscriber) for more information.
>
> ```sh
> export RUST_LOG="lure=trace" # log trace level logs only from lure
> export RUST_LOG="trace" # log trace level logs from every library used that supports it
> ```

## Configuration

Lure uses a YAML configuration file and environment variables for configuration. Check [the sample configuration file](resources/config.sample.yaml) as a reference, as it contains important information for every option (including environment variables).

To generate an example configuration file, run:

```sh
lure config generate # prints to the stdout
lure config generate >config.yaml # creates a file
```

### Services (Features)

Lure currently has two service features and they're enabled by default: LastFM and ListenBrainz. PRs for adding new platforms is very welcome.

> [!TIP]
> If you'd like to only enable the service you're using, you can pass `--no-default-features` and `--features services-<platform>` to the [install command above](#install), `<platform>` being the lowercase platform string. To see all exact feature names, [see Cargo.toml](Cargo.toml)
