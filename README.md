# `lr`

`lr` is a small daemon that sets the currently playing track on Last.fm to the Revolt user status.

## Installation

Grab a (linux only) binary from releases or compile it yourself (just `cargo build --release`).

## Configuration

`lr` uses env variables for configuration. Here's a table with all settings:

| Variable name | Required | Default value       | Description                                                                    |
|---------------|----------|---------------------|--------------------------------------------------------------------------------|
| `LR_TOKEN`    | +        |                     | Revolt user session token                                                      |
| `LR_API_KEY`  | +        |                     | Last.fm API key. Create one [here](https://www.last.fm/api/account/create)     |
| `LR_USER`     | +        |                     | Your Last.fm username                                                          |
| `LR_DELAY`    | -        | 5                   | Delay between polling in seconds. Minimum of 5 is recommended                  |
| `LR_TEMPLATE` | -        | 🎵 %ARTIST% – %NAME% | Status template when listening to music                                        |
| `LR_IDLE`     | -        | *none*              | Status when not listening to music                                             |

Also note that you can define variables in `.env` file.
