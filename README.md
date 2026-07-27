Rust on the Noel-V
======

Experiments of running Rust code on the [Noel-V](https://www.gaisler.com/products/noel-v) processor
on an [Arty-A7 FPGA board](https://digilent.com/shop/arty-a7-100t-artix-7-fpga-development-board/).

This repository currently contains the following components:

- [`gateware`](./gateware/) contains instructions on how to download and flash a pre-built Noel-V
  design to the Arty-A7 board.
- [`firmware`](./firmware/) contains Rust code for the Noel-V processor. It contains the following
  components:
    - [`grlib` library](./firmware/grlib/) which contains generic drivers for GRLIB components.
    - [`noelv` library](./firmware/noelv/) which contains Noel-V specific drivers and components.
    - [`arty-a7` application](./firmware/arty-a7/) which contains various applications you can flash
    - [`arty-a7-embassy` application](./firmware/arty-a7-embassy/) which contains example apps
      using the [embassy](https://github.com/embassy-rs/embassy) framework.

## Prerequisites

To build and run Rust software on the Noel-V, you need the following tools.

- [Rust installation](https://rust-lang.org/tools/install/) for building the firmware
- [`just` command runner installation](https://github.com/casey/just)
- [grmon installation](https://www.gaisler.com/products/grmon4) for flashing the firmware
- [Optional: NCC installation](https://www.gaisler.com/products/noel-bare-metal-cross-compiler) or
  `gdb-multiarch` if you want to do GDB debugging.

## Setting up the Arty-A7

You need to prepare the Noel-V core first by flashing the corresponding FPGA design to the
board. The [`gateware` README](./gateware/README.md) gives instructions on how to do this.

## Flashing the firmware

After you have prepared the board, you can simply use `cargo run --release --bin arty_a7` inside
the firmware folder to flash an example application to the board. The project `justfile` and
`.cargo/config.toml` configure a runner which calls `grmon` and uses it to flash and run the Rust
application.
