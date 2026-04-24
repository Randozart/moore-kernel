# Moore Kernel

A bare-metal operating system that treats FPGA bitstreams as first-class physical processes. Programs are written in Brief, transpiled to SystemVerilog, synthesized into bitstreams, and mounted onto live FPGA fabric.

**Status:** MVP development — not production-ready

## Quick Start

```bash
# Build everything
make build

# Run tests
make test

# Deploy to SD card (requires KV260 connected)
make deploy DEVICE=/dev/sdX

# Connect to Moore Shell over UART
make monitor
```

## Architecture

```
User writes Brief (.bv)
        │
        ▼
Brief Compiler + Verifier
        │  Formal contracts, LTL, SMT proofs
        ▼
SystemVerilog
        │
        ▼
Vivado Synthesis + Place-and-Route
        │
        ▼
Signed Bitstream Package (.bvc + .bv.sig + .bv.enc)
        │
        ▼
SD Card Storage
        │
        ▼
Moore Kernel (bare-metal on KV260 ARM Cortex-A53)
        │
        ▼
PCAP/ICAP ──► Mounted to FPGA Reconfigurable Partition
        │
        ▼
Rendered GPU (bitstream) ──► HDMI Display
```

## The Language Stack

| Layer | Extension | Role |
|---|---|---|
| Brief | `.bv` | Pure logic, contracts |
| Embedded Brief | `.ebv` | Chip drivers, tethers |
| Brief Control | `.bvc` | Orchestration, mounting |
| Moore Kernel | — | Bare-metal OS |
| Moore Shell | `.msh` | Predicate-logic CLI |

## Target Hardware

- **Xilinx Kria KV260** (primary development target)
  - Zynq UltraScale+ MPSoC (XCZU3CG)
  - 256,000 logic cells
  - 1,248 DSP slices
  - 2 GB DDR4

## Security

Multi-tenant security is enforced from day one:
- Formal leakage contract verification before any bitstream can mount
- PUF-encrypted bitstreams at rest
- Active Ring Oscillator fences in moat zones
- SHELL/ROLE architecture prevents cross-partition data exfiltration

## Project Structure

```
brief/compiler/     # Brief language compiler
brief/verifier/      # Formal verification (LTL + SMT)
brief/stdlib/       # Standard library
brief-control/       # Brief Control compiler + orchestrator
kernel/moore/        # Moore Kernel (bare-metal)
kernel/msh/          # Moore Shell (predicate-logic CLI)
kernel/drivers/      # Board drivers (DDR4, SD, PCAP)
kernel/security/     # PUF, active fences, APAC
bitstreams/          # Bitstream sources and blanks
shell/               # Static Shell Vivado project
ebv/                 # Embedded Brief for target boards
docs/decisions/      # Architecture Decision Records
```

## Moore Shell (msh)

msh is not a traditional CLI. It is a **Propositional Interface** based on predicate logic. You do not issue commands; you propose states of the world.

```
MOORE SHELL v0.1

FABRIC:
- TILE(0) IS [CONNECTED] AND [VERIFIED].
- FABRIC_CAPACITY: 256,000 LUTs.

STORAGE (SD Card):
- Imp_Core.bvc       [VERIFIED]  [READY]
- Rendered_GPU.bvc   [VERIFIED]  [READY]

WHAT IS YOUR PROPOSITION?
> Imp_Core exists_on Tile_0.
WORK REQUIRED:
  1. Load Imp_Core.bvc from SD card.         [OK]
  2. Verify signature against PUF-KEK.        [OK]
  3. Check leakage contract.                 [VERIFIED]
  4. Mount to RP-0 via PCAP.                 [OK]
  5. Activate active fence around RP-0.      [OK]
RESULT: Imp_Core exists_on Tile_0. [TRUE]
```

## Build Requirements

- Rust (latest stable)
- Vivado 2024.x (for synthesis + PnR)
- ARM cross-compiler (aarch64-none-elf)
- SD card FAT32 formatted

## References

See `SPEC.md` for the full MVP specification.
