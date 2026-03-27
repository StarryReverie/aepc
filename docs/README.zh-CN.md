# Aepc

Aepc（全称 Arknights: Endfield Pipeline Calculator）—— 终端中的明日方舟：终末地集成工业系统。

[English](README.md) | [简体中文](/docs/README.zh-CN.md)

## 概览

Aepc 是一款用于游戏[明日方舟：终末地](https://endfield.hypergryph.com/)的集成工业系统的生产规划工具，使用 Rust 编写。

![生产计划树](/docs/assets/plan-tree.png)

Aepc 可为目标物品生成标注关键指标的树状最优生产方案。任意复杂的流水线均可被正确规划，支持：

- 多产物配方
- 物品多来源生产
- 循环流水线
- 跨配方副产物消耗

## 使用方法

Aepc 是一个 TUI 应用程序，在终端中直接运行 `aepc` 即可启动。

1. 在 `Expected Flow` 输入框中输入目标生产流量。
2. 在 `Expected Item List` 中搜索并选择目标物品。
3. `Plan Tree` 面板将显示最优生产方案。

## 构建

### Cargo

Aepc 使用 Rust 2024 edition，需要 Rust 1.85 或更高版本。`rust-toolchain.toml` 中固定了 1.93.1 工具链，但任何更新的 stable 工具链也可正常使用。

运行以下命令构建应用程序：

```sh
cargo build --release
```

将二进制文件安装到 `~/.cargo/bin`：

```sh
cargo install --path .
```

### Nix

本项目提供了 Nix flake 来支持可复现构建。将下方命令中的 `<system>` 替换为你的目标平台：`x86_64-linux`、`aarch64-linux`、`x86_64-darwin` 或 `aarch64-darwin`。

从当前目录构建：

```sh
# 使用实验性的 Nix flake 和 Nix command CLI
nix --extra-experimental-features "nix-command flakes" build .#packages.<system>.aepc
# 使用传统特性的稳定 Nix
nix-build . -A packages.<system>.aepc
```

将本 Flake 提供的应用程序添加到 NixOS 配置中：

```nix
# flake.nix
{
  inputs.aepc.url = "github:starryreverie/aepc";
  # ...
}

# configuration.nix
{ pkgs, inputs, ... }:
{
  environment.systemPackages = [
    inputs.aepc.packages.${pkgs.stdenv.hostPlatform.system}.aepc
  ];
}
```

## 许可证

本项目的源代码基于 [GPL-3.0-or-later](/LICENSE) 许可证授权，嵌入游戏数据的部分除外。

明日方舟：终末地是[鹰角网络](https://www.hypergryph.com/)的商标。所有与游戏相关的名称、图像和数据均归鹰角网络所有。本项目为非官方同人工具，与鹰角网络无任何关联或受其认可。

Copyright (C) 2026 Justin Chen
