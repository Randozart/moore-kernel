#!/bin/bash
# assemble.sh - Assemble Moore Kernel from Brief to ARM ELF
# Usage: ./assemble.sh <kernel.bv> [--out <dir>]
#
# Prerequisites:
#   - aarch64-none-elf toolchain (cargo install cross)
#   - ARM target: rustup target add aarch64-none-elf
#   - Linked to: target/aarch64-none-elf/lib/moore-kernel.rlib

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KERNEL_DIR="$PROJECT_ROOT/kernel/moore"
GENERATED_DIR="$KERNEL_DIR/generated"
LINKER_SCRIPT="$KERNEL_DIR/linker/kernel.ld"

# Parse arguments
BV_FILE=""
OUTPUT_DIR="./build"

while [[ $# -gt 0 ]]; do
    case $1 in
        --out)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        *.bv)
            BV_FILE="$1"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

if [[ -z "$BV_FILE" ]]; then
    echo "Usage: assemble.sh <kernel.bv> [--out <dir>]"
    exit 1
fi

echo "=== Moore Kernel Assembler ==="
echo "Source: $BV_FILE"
echo "Output: $OUTPUT_DIR"

# Step 1: Compile Brief to ARM Rust
echo "Step 1: Compiling Brief to ARM Rust..."
mkdir -p "$GENERATED_DIR"
"$PROJECT_ROOT/target/release/counsel" arm "$BV_FILE" --out "$GENERATED_DIR"

# Step 2: Copy generated code to moore crate
echo "Step 2: Copying to moore kernel..."
cp "$GENERATED_DIR/main.rs" "$KERNEL_DIR/src/generated.rs"

# Step 3: Build with cargo
echo "Step 3: Building ARM ELF..."
cd "$PROJECT_ROOT"

# Check for ARM target
if ! rustup target list | grep -q "aarch64-none-elf (installed)"; then
    echo "Installing aarch64-none-elf target..."
    rustup target add aarch64-none-elf
fi

# For now, we can't build moore directly because it needs embedded targets
# This would be run on the actual target or with proper cross-compilation setup
echo "Note: Full ARM build requires:"
echo "  - aarch64-none-elf-gcc cross-compiler"
echo "  - Linked against kernel/moore/libmoore.a"
echo ""
echo "Generated files:"
ls -la "$GENERATED_DIR/"
echo ""
echo "To build manually:"
echo "  aarch64-none-elf-gcc -T $LINKER_SCRIPT -nostdlib \\"
echo "    -o build/moore.elf \\"
echo "    generated/libmoore.a"

echo "=== Assembly complete ==="