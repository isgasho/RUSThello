.PHONY: emit-asm emit-llvm-ir run
emit-asm:
	cargo rustc --release --bin arbeval -- --emit=asm -C target-feature=+popcnt
emit-llvm-ir:
	cargo rustc --release --bin arbeval -- --emit=llvm-ir -C target-feature=+popcnt
run:
	cargo run --release --bin arbeval -- -C target-feature=+popcnt
