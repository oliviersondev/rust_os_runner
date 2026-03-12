build-kernel:
	cargo build --package kernel --target x86_64-unknown-none

run-qemu:
	cargo run

run-debug_qemu:
	cargo r -- -s -S
