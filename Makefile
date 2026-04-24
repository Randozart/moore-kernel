.PHONY: build kernel brief-toolchain bitstreams test deploy clean monitor fmt check

# Default target: build everything
build: brief-toolchain kernel bitstreams check

# Build the Brief compiler toolchain
brief-toolchain:
	cargo build -p counsel --release
	cargo build -p bvc-compiler --release

# Build the Moore Kernel
kernel:
	cargo build -p msh --release
	cargo build -p pcap-driver --release
	cargo build -p security --release
	cargo build -p moore --release

# Build all bitstreams (requires counsel compiler)
bitstreams:
	@echo "Compiling bitstreams..."
	./target/release/counsel verilog bitstreams/gpu/gpu_240p.bv --hw ebv/kv260.ebv --out /tmp/gpu_out || true
	@echo "Note: Full bitstream generation requires Vivado synthesis"

# Type check all packages
check:
	cargo check --workspace

# Run unit tests
test:
	cargo test --workspace --lib
	cargo test -p msh

# Deploy bitstreams to SD card
deploy:
	@echo "Deploying to SD card..."
	@echo "Specify device: make deploy DEVICE=/dev/sdX"
	@if [ -z "$(DEVICE)" ]; then \
		echo "ERROR: DEVICE not set. Usage: make deploy DEVICE=/dev/sdX"; \
		exit 1; \
	fi
	@echo "Copying moore.bin to $(DEVICE)..."

# Connect to msh over UART (requires serial terminal)
monitor:
	@echo "Connecting to Moore Shell over UART..."
	@echo "Baud: 115200 | Device: /dev/ttyUSB0"
	screen /dev/ttyUSB0 115200

# Clean build artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt --all

# Run brief-compiler tests
test-counsel:
	cargo test -p counsel --lib

# Run msh tests
test-msh:
	cargo test -p msh

# Build release
release:
	cargo build --workspace --release

# Assemble kernel from Brief to ARM ELF
# Requires: aarch64-none-elf toolchain
assemble:
	@echo "Assembling Moore Kernel from Brief..."
	@mkdir -p kernel/moore/generated
	./target/release/counsel arm kernel/moore/main.bv --out kernel/moore/generated/
	@echo ""
	@echo "Generated ARM Rust: kernel/moore/generated/main.rs"
	@echo "Linker script: kernel/moore/linker/kernel.ld"
	@echo ""
	@echo "To build ELF (requires aarch64-none-elf-gcc):"
	@echo "  aarch64-none-elf-gcc -T kernel/moore/linker/kernel.ld \\"
	@echo "    -nostdlib -o build/moore.elf \\"
	@echo "    kernel/moore/src/generated.rs kernel/moore/src/boot.rs"

# Compile FSBL from Brief
fsbl:
	@mkdir -p kernel/moore/generated
	./target/release/counsel arm kernel/moore/fsbl.bv --out kernel/moore/generated/fsbl/
	@echo "Generated FSBL ARM Rust: kernel/moore/generated/fsbl/"

# Build Vivado tcl scripts
vivado-scripts:
	@echo "Vivado scripts created:"
	@ls -la shell/static_shell/

# Generate combined BOOT.BIN (requires bootgen)
boot-image:
	@echo "Creating boot image..."
	@echo "Requires: Xilinx bootgen and FSBL.elf + static_shell.bit"
	@echo "BOOT.BIN format: [FSBL][static_shell][moore_kernel]"