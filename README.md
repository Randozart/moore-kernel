# Moore Kernel
<img src="moore-seal.svg" alt="Brief" width="200"/> 

A bare-metal operating system that treats FPGA bitstreams as first-class physical processes. Programs are written in Brief, transpiled to SystemVerilog, synthesized into bitstreams, and mounted onto live FPGA fabric.

**Status:** MVP development — not production-ready

## A Brief Manifesto

For seventy years, operating systems have been managers of *time*. The Von Neumann architecture relies on a rigid, unchangeable pipe where massive amounts of energy are wasted fetching, decoding, and predicting sequential instructions. It is, fundamentally, a heat engine that does computation. This is awfully reductive, I am aware, but bear with my rhetoric device.

The Moore Kernel attempts to abandon this bottleneck. It is an operating system that manages physical space. 

By compiling Brief’s declarative contracts directly into physical circuit topology, the instruction-fetch bottleneck is eliminated. Data does not wait in memory; it flows continuously through the hardware. This shift unlocks three core principles:

* **The Ecological Imperative:** Traditional software turns raw power into wasted clock cycles. Spatial computing allows highly complex logic to run directly through physical gates at a fraction of a watt. It provides an underlying architecture capable of running complex ecological, biomimetic, and systemic models at speeds and efficiencies traditional CPUs cannot reach.

* **Immortal Hardware:** When traditional software outgrows a fixed chip, millions of tons of silicon become e-waste. The Moore Kernel decouples the *function* from the *factory*. Because the hardware fabric is fluid, a single piece of silicon can be a dedicated network router in the morning, dynamically reconfigure into a spatial simulation engine in the afternoon, and become a medical controller at night. Silicon no longer dies, but is allowed to adapt.

* **Opinionated Systems Engineering:** We are standing at the footsteps of a new era. The execution barrier has collapsed. If I, a single person behind a PC can design a kernel and OS by simply learning about systems architecture, and with AI pattern matchers removing the friction of boilerplate, architecture is now bounded only by our hopes, dreams, and curiosities. 

The Moore Kernel assumes infinite extensibility as a baseline. It is an invitation to build the "GeoCities of silicon". To return to an era of weird, beautiful, highly-opinionated computing where hardware is a canvas, and we simply propose the reality we wish to see.

If the future is as wonderful and whimsical as I hope it to be, then a few years from now:

* People will build bespoke operating systems that only exist to run a single synthesizer in their bedroom.

* Someone will write an OS where the file system isn't a hierarchy of folders, but a literal 3D spatial map they navigate with a joystick.

* Someone else will build a kernel that completely deletes itself and rebuilds from scratch every time the sun sets.

And here stands Moore, an OS where the hardware itself melts and reconfigures based on propositional logic. If not for my engineering skill, I at least invite you to dream with me of an era where systems and technology are a playground, to those willing to learn them.

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
