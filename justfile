GDB_CMD := justfile_directory() / "firmware/gdb.gdb"

all: check clippy check-fmt build

check-fmt:
  cd firmware && cargo fmt --all -- --check

fmt:
  cd firmware && cargo fmt

check:
  cd firmware && cargo check

build:
  cd firmware && cargo build

clippy:
  cd firmware && cargo clippy

start-grmon:
  grmon -digilent -gdb

[no-cd]
run binary init_args="":
  grmon -digilent -u -batch -e "load {{binary}}; run"

[no-cd]
debug-gdb-multiarch binary init_args="":
  gdb-multiarch -q -x {{GDB_CMD}} {{binary}} -tui

[no-cd]
debug-gdb-gaisler binary init_args="":
  riscv-gaisler-elf-gdb -q -x {{GDB_CMD}} {{binary}} -tui

[working-directory:"gateware"]
download-bitstream:
  curl -L -o noelv.bit "https://www.dropbox.com/scl/fi/ye6i26jvvgo15wff0y42g/noelv.bit?rlkey=hs7rb1zspmm6giq3941ptxv5g&st=4y87q674&dl=0"

[working-directory:"gateware"]
download-mcs:
  curl -L -o noelv.mcs "https://www.dropbox.com/scl/fi/sxztmzxqqueqx5tqzlyev/noelv.mcs?rlkey=fa2xl5md479dtnj3aft0kgeoy&st=qsxmrjd0&dl=0"

[working-directory:"gateware"]
flash-nvm:
  openFPGALoader -b arty_a7_100t --fpga-part xc7a100tcsg324 -f --verify noelv.mcs

flash-any-bitstream binary:
  openFPGALoader -b arty_a7_100t --fpga-part xc7a100tcsg324 --verify noelv.bit

[working-directory:"gateware"]
flash-bitstream:
  openFPGALoader -b arty_a7_100t --fpga-part xc7a100tcsg324 --verify noelv.bit
