GDB_CMD := justfile_directory() / "firmware/gdb.gdb"

[no-cd]
run binary init_args="":
  gdb-multiarch -q -x {{GDB_CMD}} {{binary}} -tui
