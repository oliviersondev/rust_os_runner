build-kernel:
	cargo build --package kernel --target x86_64-unknown-none

run-qemu:
	cargo run

run-debug-qemu:
	cargo r -- -s -S

run-gdb:
	gdb $(KERNEL_PATH)
# todo essayer de lancer run-debug et run gdb en une fois
	#(gdb)target remote :1234
	#(gdb) break main.rs:36
	#(gdb) continue