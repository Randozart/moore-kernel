.PHONY: build kernel brief-toolchain bitstreams test deploy clean monitor

# Default target: build everything
build: brief-toolchain kernel bitstreams

# Build the Brief compiler + verifier toolchain
brief-toolchain:
	cd brief/compiler && cargo build --release
	cd brief/verifier && cargo build --release

# Build the Moore Kernel (bare-metal)
kernel:
	cd kernel/moore && cargo build --release
	cd kernel/msh && cargo build --release

# Build all bitstreams (requires Vivado)
bitstreams:
	@echo "Bitstream build requires Vivado. Run: make bitstreams TARGET=kv260"
	@echo "Individual bitstreams:"
	@ls bitstreams/gpu/*.bv 2>/dev/null || echo "No .bv sources found"

# Deploy bitstreams to SD card
deploy:
	@echo "Deploying to SD card..."
	@echo "Specify device: make deploy DEVICE=/dev/sdX"
	@if [ -z "$(DEVICE)" ]; then \
		echo "ERROR: DEVICE not set. Usage: make deploy DEVICE=/dev/sdX"; \
		exit 1; \
	fi
	@echo "Copying bitstreams to $(DEVICE)..."
	@mkdir -p /tmp/moore_mnt && sudo mount $(DEVICE) /tmp/moore_mnt
	@sudo cp bitstreams/gpu/*.bvc /tmp/moore_mnt/ 2>/dev/null || true
	@sudo cp bitstreams/blanks/*.bvc /tmp/moore_mnt/ 2>/dev/null || true
	@sudo umount /tmp/moore_mnt

# Run unit tests
test:
	cargo test --lib
	cargo test --workspace

# Connect to msh over UART (requires serial terminal)
monitor:
	@echo "Connecting to Moore Shell over UART..."
	@echo "Baud: 115200 | Device: /dev/ttyUSB0"
	screen /dev/ttyUSB0 115200

# Clean build artifacts
clean:
	cargo clean
	rm -f bitstreams/gpu/*.bit bitstreams/gpu/*.bvc bitstreams/blanks/*.bit bitstreams/blanks/*.bvc

# Format code
fmt:
	cargo fmt --all
