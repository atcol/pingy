[![Build legacy Nix package on Ubuntu](https://github.com/atcol/pingy/actions/workflows/build_nix.yml/badge.svg)](https://github.com/atcol/pingy/actions/workflows/build_nix.yml)

# pingy
A fast &amp; simple website monitor running in [Shuttle](https://shuttle.rs).

## Development

### Setup

Make sure you've set up your Shuttle project as-per [Shuttle's quick-start guide](https://docs.shuttle.rs/getting-started/quick-start).

This project uses Nix. To begin development you can:

```bash
$ nix develop
```

## Deploying

```bash
$ cargo shuttle deploy
```
