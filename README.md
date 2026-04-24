# Moore Kernel

A bare-metal operating system that treats FPGA bitstreams as first-class physical processes. Programs are written in Brief, transpiled to SystemVerilog, synthesized into bitstreams, and mounted onto live FPGA fabric.

**Status:** MVP development — not production-ready

## Quick Start

```bash
# Build everything
make build

# Run tests
make test

# Check code compiles
make check

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
counsel (Brief Compiler)
        │  Formal contracts, LTL, SMT proofs
        ▼
SystemVerilog
        │
        ▼
Vivado Synthesis + Place-and-Route
        │
        ▼
Signed Bitstream Package (.writ + .writ.sig)
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
| Rendered Brief | `.rbv` | Hardware UI (HTML/CSS → circuit) |

## Target Hardware

- **Xilinx Kria KV260** (primary development target)
  - Zynq UltraScale+ MPSoC (XCZU3CG)
  - 256,000 logic cells
  - 1,248 DSP slices
  - 2 GB DDR4

## Project Structure

```
brief-compiler/     # counsel - Brief language compiler (89 tests)
brief-control/bvc/  # bvc-compiler - Brief Control compiler (4 tests)
kernel/
  msh/              # Moore Shell - Propositional Interface (9 tests)
  moore/            # Moore Kernel - no_std bare-metal kernel
  drivers/          # PCAP driver
  security/         # PUF, active fences, APAC
ebv/                # Embedded Brief files (kv260.ebv, kernel.ebv)
bitstreams/gpu/     # GPU bitstream sources (gpu_240p.bv)
shell/static_shell/ # Static Shell documentation
tools/              # build.py, sign.py
```

## Moore Shell (msh)

msh is a **Propositional Interface** based on predicate logic. You do not issue commands; you propose states of the world.

```
MOORE SHELL v0.1
══════════════════════════════════════════════════════════

FABRIC:
- TOTAL CAPACITY: 256000 LUTs
- AVAILABLE:        256000 LUTs

STORAGE (SD Card):
- Imp_Core.writ  [1234567 bytes] [VERIFIED]
- Rendered_GPU.writ  [5678901 bytes] [VERIFIED]

MOUNTED:
- None.

PROPOSITIONAL CONTEXT READY.
WHAT IS YOUR PROPOSITION?
> Imp_Core exists_on Tile_0.
WORK REQUIRED:
  1. Load Imp_Core.writ from SD card.         [OK]
  2. Verify signature against PUF-KEK.        [OK]
  3. Check leakage contract.                   [VERIFIED]
  4. Mount to RP_0 via PCAP.                  [OK]
  5. Activate fence.                           [OK]
RESULT: Imp_Core exists_on Tile_0. [TRUE]
```

### Tether Discovery

Tethers expose hardware state via SQL-like queries. Use `?` for discovery:

```
> Tile_0 ?
TILE(0) is a [FABRIC_TILE].
Accepts predicates: { exists_on, absent, is_active, clear, probe }.
Capacity: 256000 LUTs, 256000 available.
```

## Security

Multi-tenant security is enforced from day one:
- Formal leakage contract verification before any bitstream can mount
- PUF-encrypted bitstreams at rest
- Active Ring Oscillator fences in moat zones
- SHELL/ROLE architecture prevents cross-partition data exfiltration

## Build Requirements

- Rust (latest stable)
- Vivado 2024.x (for synthesis + PnR)
- ARM cross-compiler (aarch64-none-elf)
- SD card FAT32 formatted

## References

- `SPEC.md` — Full MVP specification
- `BRIEF_CONTROL_SPEC.md` — Brief Control language specification
- `CLAUDE.md` — Developer guidelines
- `LITANIES.md` — Chancery manifest