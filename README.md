# Aepc

[English](README.md) | [简体中文](/docs/README.zh-CN.md)

Aepc (a.k.a. Arknights: Endfield Pipeline Calculator) — Endfield's Automated Industry Complex inside your terminal.

[![built with garnix](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fgarnix.io%2Fapi%2Fbadges%2FStarryReverie%2Faepc%3Fbranch%3Dmaster)](https://garnix.io/repo/StarryReverie/aepc)

## Overview

Aepc is a planning utility application for the AIC system in the game [Arknights: Endfield](https://endfield.hypergryph.com/), written in Rust.

![Plan Tree](/docs/assets/plan-tree.png)

Generates tree-style optimal production plans for any item with essential metrics annotated. Arbitrarily complex pipelines are well supported, since Aepc handles:

- Multi-product recipes
- Multi-source production for items
- Cyclic pipelines
- Inter-recipe byproduct consumption

## Usage

Aepc is a TUI application. You can simply run `aepc` to launch.

1. Enter a target production flow in the `Expected Flow` input.
2. Search and select a target item from the `Expected Item List`.
3. The optimal production tree will be displayed in the `Plan Tree` panel.

## Build

### Cargo

Aepc uses the 2024 edition, which requires Rust 1.85 or later. The toolchain is pinned to 1.93.1 via `rust-toolchain.toml`, but any newer stable toolchain should work as well.

Run the following command to build the application:

```sh
cargo build --release
```

To install the binary to `~/.cargo/bin`:

```sh
cargo install --path .
```

### Nix

A Nix flake is provided. Replace `<system>` in the commands below with your target platform: `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, or `aarch64-darwin`.

Build from the current directory:

```sh
# Using Nix flake and Nix command CLI
nix --extra-experimental-features "nix-command flakes" build .#packages.<system>.aepc
# Using traditional and stable Nix
nix-build . -A packages.<system>.aepc
```

Add the application exported by this flake to your NixOS configuration:

```nix
# flake.nix
{
  inputs.aepc.url = "github:StarryReverie/aepc";
  # ...
}

# configuration.nix
{ pkgs, inputs, ... }:
{
  environment.systemPackages = [
    inputs.aepc.packages.${pkgs.stdenv.hostPlatform.system}.aepc
  ];

  # Optional, merge the following lines with your existing configurations
  nix.settings.substituters = [ "https://cache.garnix.io" ];
  nix.settings.trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];
}
```

To avoid building locally, add [garnix.io](https://garnix.io) as a substituter:

```nix
# configuration.nix
{ pkgs, inputs, ... }:
{
  nix.settings.substituters = [ "https://cache.garnix.io" ];
  nix.settings.trusted-public-keys = [ "cache.garnix.io:CTAFy1dD+zsR3d+Cc32+Rz2m1/rg3oC+jhSFBzV3Y6c=" ];

  environment.systemPackages = [
    inputs.aepc.packages.${pkgs.stdenv.hostPlatform.system}.aepc
  ];
}
```

## License

The source code of this project is licensed under [GPL-3.0-or-later](/LICENSE), except the portion that embeds the game data.

Arknights: Endfield is a trademark of [Hypergryph](https://www.hypergryph.com/). All game-related names, images, and data are the property of Hypergryph. This project is an unofficial fan-made tool and is not affiliated with or endorsed by Hypergryph.

Copyright (C) 2026 Justin Chen
