Noel-V Arty-A7 Gateware
=========

This folder contains instructions on how to download a pre-built Noel-V FPGA design to the
Arty-A7 board.

The pre-built designs can be found [here](https://www.dropbox.com/scl/fo/y3q5f416iv64b0jysv9v2/AOSrq_-3WJCGfaZ07Yuz-ic?rlkey=98nm1ufbpqzlxmraftxoyctjr&st=2k6700kp&dl=0). Some scripts are provided to automatically
download and flash the appropriate files.

The current design is based on version `grlib-gpl-2025-2-b4298` of the GRLIB. The configuration
used is provided in `config.vhd` and `config.h`.

## Prerequisites

1. Install the [openFPGALoader](https://trabucayre.github.io/openFPGALoader/guide/install.html).
2. Install the [curl tool](https://curl.se/download.html)
3. Install the [grmon4 tool](https://www.gaisler.com/products/grmon4)
4. (Recommended) Install [just](https://github.com/casey/just).

## Downloading and flashing the Noel-V design to the QSPI memory

Make sure jumper JP1 is set so the Arty-A7 will program the FPGA from the
QSPI flash on boot. The easiest way to flash the board with the pre-built design is to use the
provided `justfile` commands:

1. Use `just download-mcs` to download the pre-built MCS file for the Noel-V design.
2. Use `just flash-nvm` to flash the downloaded MCS file to the QSPI using the `openFPGALoader`

Please note it will take about 10 seconds for the design to be configured after power-up.

## Downloading and Flashing the Noel-V design via JTAG

If you want to only flash the bitstream to the board via JTAG without writing it to non-volatile
memory, you can use the following commands:

1. Use `just download-bitstream` to download the pre-built design bitstream for the Noel-V design.
2. Use `just flash-bitstream` to flash the downloaded file to the QSPI using the `openFPGALoader`

Please note that this will not write the bitstream to non-volatile memory, so the design will be
lost on power cycle.

## Verifying the design works properly

You can use the command `grmon -digilent` to connect to the Noel-V design. You also need to close
all AMD tooling like the Vivado Hardware Manager or other tools like `hw_server` before you do
this.

You should see output like this:

```sh
❯ grmon -digilent


  GRMON debug monitor v4.1.2 64-bit eval version

  Copyright (C) 2026 Frontgrade Gaisler - All rights reserved.
  For latest updates, go to https://www.gaisler.com/
  Comments or bug-reports to support@gaisler.com

  This eval version will expire on 25/12/2026

WARNING! Share directory not found
JTAG chain (1): xc7a100t
  Device ID:           0x330
  GRLIB build version: 4298
  Detected frequency:  40.0 MHz

  Component                            Vendor
  NOEL-V RISC-V Processor              Frontgrade Gaisler
  AHB Debug UART                       Frontgrade Gaisler
  JTAG Debug Link                      Frontgrade Gaisler
  AHB/APB Bridge                       Frontgrade Gaisler
  Xilinx MIG Controller                Frontgrade Gaisler
  Generic AHB ROM                      Frontgrade Gaisler
  AHB/APB Bridge                       Frontgrade Gaisler
  AHB/APB Bridge                       Frontgrade Gaisler
  RISC-V ACLINT                        Frontgrade Gaisler
  RISC-V PLIC                          Frontgrade Gaisler
  RISC-V Debug Module                  Frontgrade Gaisler
  Generic UART                         Frontgrade Gaisler
  Modular Timer Unit                   Frontgrade Gaisler
  Version and Revision Register        Frontgrade Gaisler
  AHB Status Register                  Frontgrade Gaisler
  General Purpose I/O port             Frontgrade Gaisler

  Use command 'info sys' to print a detailed report of attached cores
```

## MCS file generation

The MCS file is QSPI chip specific and was generated for the `s25fl128sxxxxxx0-spi-x1_x2_x4` QSPI
flash device. The Vivado commands to do this are provided here for reference.

```tcl
create_hw_cfgmem -hw_device [get_hw_devices xc7a100t_0] -mem_dev [lindex [get_cfgmem_parts {s25fl128sxxxxxx0-spi-x1_x2_x4}] 0]
write_cfgmem  -format mcs -size 16 -interface SPIx1 -loadbit {up 0x00000000 "noelvmp.bit" } -force -file "noelv.mcs"
```
