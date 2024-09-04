# lure

[![CI Status](https://img.shields.io/github/actions/workflow/status/catuhana/lure/ci.yaml?style=flat-square&label=CI)](https://github.com/catuhana/lure/actions/workflows/ci.yaml)
[![CD Status](https://img.shields.io/github/actions/workflow/status/catuhana/lure/cd.yaml?style=flat-square&label=CD)](https://github.com/catuhana/lure/actions/workflows/cd.yaml)
[![Latest Release](https://img.shields.io/github/v/release/catuhana/lure?style=flat-square)](https://github.com/catuhana/lure/releases/latest)

Lure is an improved fork of [lr](https://codeberg.org/arslee07/lr), a small process that sets the currently playing track on Last.fm, ListenBrainz (and other future platforms, PRs welcome!) as Revolt user status.

> [!WARNING]
> Version 1 contains big configuration changes. If you were on previous versions, check [configuration](#configuration) section.

## Installation

> [!IMPORTANT]
> Lure is tested on Linux and Windows, and is expected to work on macOS too.

```sh
cargo install --git https://github.com/catuhana/lure
```

Or if you'd want to use an container image (Docker, Podman, etc.), you can pull the image from GitHub Container Registry.

```sh
docker pull ghcr.io/catuhana/lure:latest
```

> [!TIP]
> Container images support AMD64, ARM64 and ARM64v7 architectures.

## Running

> [!IMPORTANT]
> Reading the [configuration](#configuration) first is highly recommended.

To run lure, run:

```sh
lure start
```

Or using a container management tool:

```sh
docker/podman run -v $(pwd)/config.yaml:/app/config.yaml:ro lure:latest
```

> [!TIP]
> By default, lure logs useful information to the console. If you'd want to see other log levels, use the `RUST_LOG` environment variable. Check [`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) documentation from [`tracing-subscriber`](https://docs.rs/tracing-subscriber) for more information.
>
> ```sh
> export RUST_LOG="lure=trace" # log trace level logs only from lure
> export RUST_LOG="trace" # log trace level logs from every library used that supports it
> # for container management tools, use `-e RUST_LOG=` option
> ```

## Configuration

Lure uses a YAML configuration file and environment variables for configuration. Check [the sample configuration file](resources/config.sample.yaml) as a reference, as it contains important information for every option (including environment variables).

To generate an example configuration file, run:

```sh
lure config generate # prints to the stdout
lure config generate >config.yaml # creates a file
```

### Container Management Tools

If you're using any container management tools, you can either mount the host configuration file to the container or use environment variables. The volume for the app and its configuration file is `/app`. Refer to the [run section](#run) for example.

### Services (Features)

Lure currently has two service features and, they're enabled by default: LastFM and ListenBrainz. PRs for adding new platforms is very welcome.

> [!TIP]
> If you'd like to only enable the service you're using, you can pass `--no-default-features` and `--features services-<platform>` to the [install command above](#install), `<platform>` being the lowercase platform string. To see all exact feature names, [see Cargo.toml](Cargo.toml)
