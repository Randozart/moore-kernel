# Moore Kernel MVP Specification
**Version:** 0.1.0-draft
**Date:** 2026-04-24
**Status:** Proposed

---

## 1. Overview

### 1.1 Purpose

This document defines the Minimum Viable Product for the Moore Kernel — a bare-metal operating system that treats FPGA bitstreams as first-class physical processes. The MVP demonstrates the complete end-to-end loop: writing Brief logic, transpiling it to a bitstream, mounting it onto the KV260 FPGA fabric, and rendering output via HDMI.

### 1.2 Success Criteria

1. Moore Kernel boots bare-metal on KV260 from SD card with no host CPU dependency
2. A Brief Control (.bvc) script can mount a Rendered Brief GPU bitstream onto a Reconfigurable Partition
3. The mounted GPU bitstream drives HDMI output, proving the physical execution loop works
4. Moore Shell (msh) is the only CLI — declarative, predicate-logic based, zero magic words
5. Security is multi-tenant from day one: active fences, leakage contracts, PUF root of trust
6. Brief code that compiles to bitstream is formally verified before mounting

### 1.3 Out of Scope for MVP

- Aurora 64B/66B interconnect (v2)
- Multi-board fabric orchestration (v2)
- Linux-based kernel (ruled out — bare-metal only)
- JIT compilation (ruled out — all bitstreams are pre-compiled)

---

## 2. Hardware Target

### 2.1 Xilinx Kria KV260 Vision AI Starter Kit

| Parameter | Value |
|---|---|
| **FPGA** | Zynq UltraScale+ MPSoC (XCZU3CG-SFVC784) |
| **Logic Cells** | 256,000 |
| **DSP Slices** | 1,248 |
| **BRAM** | 7.2 Mb |
| **ARM Cores** | Cortex-A53 quad-core + Cortex-R5F dual-core |
| **DRAM** | 2 GB DDR4 |
| **Network** | Gigabit Ethernet (RGMII) |
| **Video** | HDMI 2.0 TX ( DisplayPort over USB-C) |
| **Storage** | SD card (primary boot) |
| **Price** | ~$199–283 |

### 2.2 Static Shell (Pre-Synthesized)

The Static Shell is a Vivado project that is **never re-synthesized during runtime**. It defines:

- All Reconfigurable Partition (RP) boundaries and floorplanning
- DFX Decoupler IP blocks at every partition boundary
- AXI NoC / crossbar connecting PS to PL
- Memory controllers (DDR4)
- HDMI TX controller (fixed, not reconfigurable)
- PCAP configuration interface
- ICAP for autonomous reconfiguration
- One pinned RP slot for the Moore Kernel's own operations

**The Static Shell is the immutable body. It never changes.**

### 2.3 Reconfigurable Partitions

The fabric is divided into fixed virtual sockets:

| Slot | Size (LUTs approx) | Intended Use |
|---|---|---|
| RP-0 | 40,000 | Moore Kernel / msh operations |
| RP-1 | 80,000 | Application bitstream (GPU, etc.) |
| RP-2 | 80,000 | Application bitstream |
| RP-3 | 40,000 | Reserved for expansion |

Slot sizes are pre-floorplanned to prevent 2D spatial fragmentation in the MVP. All bitstreams targeting the fabric must conform to one of these fixed footprints.

---

## 3. The Language Stack

### 3.1 Brief (.bv) — Pure Logic

The foundation language. Declarative, contract-based, proof-assistant.

- **File extension:** `.bv`
- **Transpiles to:** SystemVerilog
- **Verification:** Bounded model checking + SMT solvers (LEAKAGE_CONTRACT, CT_CONTRACT)
- **Domain:** Pure application logic (AI models, codecs, custom accelerators)

A Brief contract is defined as:

```
contract add_laptop(product: int, price: int) -> int {
    pre  product >= 0
    post result == product + price
}
```

All state transitions are formally captured. Side-channel leakage atoms are part of the contract.

### 3.2 Embedded Brief (.ebv) — Chip Drivers / Tethers

Written specifically per board model. Establishes the physical reality of the tile.

- **File extension:** `.ebv`
- **Content:** Pin mappings, clock trees, RAM controllers, transceiver configs, `.bvc` tethers
- **Job:** Converts a generic FPGA into a verified "Moore Tile"
- **For KV260:** A single `kv260.ebv` is the genesis block for all Moore operations on this hardware

The `.ebv` is the **birth certificate of the tile.** It is flashed once and rarely changes.

### 3.3 Brief Control (.bvc) — Orchestration Layer

The "Will" of the system. Coordinates mounting, resource negotiation, and inter-board links.

- **File extension:** `.bvc`
- **Compiles to:** A mount manifest + partial bitstream relocation patches
- **Drives:** TPU coordinate calculation, PCAP streaming, virtual socket allocation
- **Security:** Enforces that all mounted bitstreams have verified leakage contracts

Example:

```brief
using Imp_Core;

control Fabric {
    target Tile_0;
    partition Imp_Core across Tile_0;
    route high_speed_link over Port_0;
}
```

In the MVP, Brief Control operates on a single KV260. Multi-board orchestration is v2.

### 3.4 Rendered Brief — Hardware UI

HTML/CSS written directly in Brief, transpiled to physical framebuffer circuits.

- **File extension:** `.br` (pure spec, no view) or `.rbv` (Brief + View)
- **Compiled to:** Fixed spatial layout circuit — no DOM, no browser engine
- **Output:** Pixel-routing pathways wired to HDMI TX pins
- **Advantage:** UI is a direct physical manifestation of machine state; zero latency, zero attack surface

---

## 4. Moore Kernel

### 4.1 Architecture

Runs bare-metal on the ARM Cortex-A53. No Linux, no MMU virtualization.

```
┌──────────────────────────────────────────────────────────┐
│                   Moore Kernel (bare-metal)                │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Bitstream  │  │   Task       │  │    Mount /      │  │
│  │ Repository │  │ Preparation  │  │    Unmount      │  │
│  │ (SD Card)  │  │    Unit      │  │    Manager      │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   Moore    │  │  Security   │  │    Active      │  │
│  │  Shell     │  │  Manager    │  │    Fence       │  │
│  │  (msh)     │  │             │  │    Manager     │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└──────────────────────────────────────────────────────────┘
          │                   │                    │
    ┌─────┴─────┐      ┌─────┴─────┐        ┌─────┴─────┐
    │   PCAP   │      │  AXI NoC  │        │   ICAP   │
    └──────────┘      └───────────┘        └───────────┘
```

### 4.2 Kernel Subsystems

#### 4.2.1 Bitstream Repository
- Stored on SD card as signed `.bvc` packages
- Each package: `{name}.bvc` + `{name}.bv.sig` + `{name}.bv.enc`
- Packages are PUF-encrypted at rest
- Loaded on demand via SD card DMA

#### 4.2.2 Task Preparation Unit (TPU)
- Reads the .bvc mount manifest
- Calculates 2D spatial coordinates for the target RP
- Applies relocation patches to the partial bitstream
- Outputs a ready-to-stream bitstream to the Configuration Packet Processor

#### 4.2.3 Mount / Unmount Manager
- Mount sequence: TPU prep → Decoupler activate → PCAP stream → STARTUP wait → Decoupler deactivate → ENA assert
- Unmount sequence: ENA deassert → RST assert → Decoupler activate → blanking bitstream stream → STARTUP wait → Decoupler deactivate
- Blanking bitstreams are pre-compiled greybox images for each RP slot

#### 4.2.4 Security Manager
- Enforces that only cryptographically signed bitstreams are mounted
- Verifies `.bv.enc` against the board's PUF-derived KEK
- Maintains active fence state for every mounted partition
- Rejects bitstreams that fail leakage contract verification

#### 4.2.5 Active Fence Manager
- Instantiates Ring Oscillator arrays in moat zones surrounding every active RP
- Fences produce randomized electromagnetic and power noise
- Purpose: blind side-channel sensors on co-resident or malicious bitstreams
- Each fence is independently clocked and randomized

#### 4.2.6 Moore Shell (msh)
See Section 6.

### 4.3 Boot Sequence

1. POR reset releases ARM cores
2. Boot ROM reads SD card MBR
3. First-stage bootloader (FSBL) loads `moore.bin` from SD FAT32 partition
4. Moore Kernel jumps to `moore.bin` entry point
5. Kernel initializes: clocks, DDR4, SD card, PCAP, AXI NoC, msh
6. msh prints Propositional Context to UART/HDMI
7. System is ready to accept propositions

---

## 5. Formal Verification Pipeline

### 5.1 Brief Compiler + Verifier

The Brief toolchain (runs on host PC, not on KV260):

```
.bv source
    │
    ▼
[ Brief Compiler ]
    │  - Preconditions / Postconditions parsed
    │  - State machine extracted
    ▼
[ Formal Verification ]
    │
    ├─ Combinational: Boolean rewriting (automated)
    ├─ Sequential: LTL + BMC via SAT/SMT (automated)
    └─ Iterated: Higher-order induction (guided)
    │
    ▼
[ Leakage Contract Check ]
    │  - Symbolic IFT across all execution paths
    │  - SAT: does secret data influence observable trace?
    ▼
[ SystemVerilog Output ] ──── or ──── [ COMPILATION ABORTED ]
    │                                        │
    │                              Counterexample trace printed
    ▼
[ Vivado P&R ]
    │  NP-hard: 4–18 hours on high-end workstation
    ▼
[ Signed Bitstream Package ]
    │  {name}.bvc  +  {name}.bv.sig  +  {name}.bv.enc
    ▼
[ Deployed to SD card ]
```

### 5.2 Verification Gate

**Rule:** If Brief code compiles to a bitstream, it is verified. No unverified bitstream may be mounted.

The verification gate is enforced by the Security Manager. The signed package includes a verification proof record that the kernel checks before mounting.

### 5.3 Leakage Contracts

Every Brief module targeting a multi-tenant fabric must declare:

```brief
leakage contract Imp_Core {
    secret: [key_material, internal_weights];
    observable: [execution_time, power_trace];
    guarantee: secret does_not_flow_to observable;
}
```

If the SMT solver detects that any secret operand influences any observable trace, compilation fails.

---

## 6. Moore Shell (msh)

### 6.1 Philosophy

Moore Shell is a **Propositional Interface.** It is not a command line. It is a dialogue in predicate logic. The user does not issue commands; the user proposes states of the world, and the kernel makes them true.

### 6.2 Opening State

When msh boots, it presents the **Moorean Summary** — a declarative statement of what is true:

```
MOORE SHELL v0.1
══════════════════════════════════════════════════════════

FABRIC:
- TILE(0) IS [CONNECTED] AND [VERIFIED].
- FABRIC_CAPACITY: 256,000 LUTs.
- AVAILABLE:        200,000 LUTs.

STORAGE (SD Card):
- Imp_Core.bvc       [VERIFIED]  [READY]
- Rendered_GPU.bvc    [VERIFIED]  [READY]
- Blank_RP1.bvc       [VERIFIED]  [READY]

MOUNTED:
- None.

PROPOSITIONAL CONTEXT READY.
WHAT IS YOUR PROPOSITION?
> _
```

### 6.3 Predicate Syntax

All interaction is Subject-Predicate-Object. No magic words. No abbreviations.

| Proposition | Meaning |
|---|---|
| `Imp_Core exists_on Tile_0.` | Mount Imp_Core.bvc to RP-0 |
| `Imp_Core absent.` | Unmount Imp_Core from wherever it is mounted |
| `Rendered_GPU exists_on Tile_0.` | Mount Rendered GPU to RP-0 |
| `Tile_0 is_active.` | Probe current state of Tile 0 |
| `Storage contains Imp_Core.` | Check if Imp_Core is in SD card repository |

### 6.4 The Discovery Interrogative (`?`)

Never memorize a command. Discover what is possible from the subjects themselves.

```
> Tile_0 ?
TILE(0) is a [FABRIC_TILE].
Accepts predicates: { exists_on, absent, is_active, clear, probe }.

> Imp_Core ?
IMP_CORE is a [LOGIC_MODULE].
Requires: { 400 LUTs, 50 DSPs, 150MHz_clock }.
Currently: [ABSENT].
Accepts predicates: { exists_on, absent, is_verified }.

> exists_on ?
exists_on [Predicate] requires { Module_Name from Storage, Target_Tile }.
```

### 6.5 Proof-Based Error Messages

When a proposition cannot be satisfied, msh returns a **Failed Proof** — a mathematical trace of exactly why.

```
> Windows_95 exists_on Tile_0.
[PROOF FAILED]
Cannot satisfy: ExistsOn(Windows_95, Tile_0).
REASON:
  1. Windows_95 requires [logic_tiles: 800].
  2. Tile_0 available capacity is [logic_tiles: 200].
  3. (800 <= 200) is FALSE.
SUGGESTION:
  Clear RP-1 to free 600 tiles on Tile_0?
  > clear RP-1. [Y/N]
```

### 6.6 Multi-Step Transactions

The kernel performs compound work to satisfy a single proposition:

```
> Imp_Core exists_on Tile_0.
WORK REQUIRED:
  1. Load Imp_Core.bvc from SD card.         [OK]
  2. Verify signature against PUF-KEK.         [OK]
  3. Check leakage contract.                  [VERIFIED]
  4. TPU calculates relocation coordinates.   [OK]
  5. Mount to RP-0 via PCAP.                 [OK]
  6. Activate active fence around RP-0.       [OK]
RESULT: Imp_Core exists_on Tile_0. [TRUE]
```

---

## 7. First Application: Rendered Brief GPU

### 7.1 What It Does

The first mounted bitstream is a Rendered Brief GPU — a circuit that:
1. Receives pixel data from the Moore Kernel via AXI
2. Drives HDMI output directly (no software rendering engine)
3. Exposes its framebuffer as a hardware state that msh can probe

### 7.2 Pixel Data Path

```
Moore Kernel → AXI4-Stream → Rendered GPU (bitstream on RP-1) → HDMI TX → Display
```

The most efficient path: the kernel writes pixels directly to the GPU's BRAM via AXI. The GPU reads its own BRAM and drives HDMI. No intermediate framebuffer copy.

### 7.3 Rendered Brief Source (conceptual)

```brief
rendered gpu_240p {
    output resolution: 640x480 @ 60Hz;
    pixel_clock: 25.175 MHz;

    framebuffer BRAM: 640 * 480 * 4 bytes;

    render rstruct PixelBuffer {
        color: [r: 8, g: 8, b: 8];
        alpha: 8;
    }

    view {
        div id="frame" {
            style: "width: 640px; height: 480px;"
        }
    }
}
```

This compiles to a spatial layout circuit — not a DOM in memory. The div and its CSS become fixed wire routes and BRAM address generators.

---

## 8. Security Architecture

### 8.1 Threat Model

| Threat | Mitigation |
|---|---|
| Side-channel extraction (power/thermal analysis) | Active fences + leakage contracts |
| Bitstream tampering / supply chain attack | PUF-encrypted bitstreams + cryptographic signatures |
| Malicious bitstream on shared fabric | Leakage contract verification gate |
| JTAG extraction | JTAG disabled in `.ebv`; PUF locks configuration |
| DMA attack from compromised FPGA | APAC-style cryptographic pointer authentication on AXI |

### 8.2 PUF Root of Trust

Each KV260 die has unique manufacturing variations. At first boot:
1. PUF measures silicon delay characteristics
2. Derives a Device-Unique Key Encryption Key (KEK)
3. KEK is used to decrypt configuration bitstreams
4. KEK never leaves the FPGA silicon; no software-exposed key material

### 8.3 Active Fences

Every mounted RP is surrounded by a moat zone. Inside the moat:
- Ring Oscillator arrays toggle at randomized frequencies
- Randomized power draw generates EM and thermal noise
- This blinds any co-tenant attempting power analysis or thermal covert channels

### 8.4 Multi-Tenant Isolation

The SHELL (static logic) is physically isolated from all ROLEs (reconfigurable partitions):
- No ROLE bitstream can address host bus or monopolize I/O pins
- AXI NoC enforces QoS rate limiting per partition
- All interconnect traffic passes through drawbridge routing tunnels controlled by SHELL

---

## 9. Build System

### 9.1 File Structure

```
moore-kernel/
├── SPEC.md
├── README.md
├── CLAUDE.md
├── docs/
│   ├── decisions/          # Architecture Decision Records
│   └── spec/              # Detailed subsystem specs (v2)
├── brief/
│   ├── compiler/           # Brief language compiler
│   ├── verifier/           # Formal verification pipeline
│   └── stdlib/            # Standard Brief library
├── brief-control/
│   ├── bvc/               # Brief Control compiler + tethers
│   └── orchestrator/       # Mount manifest generator
├── kernel/
│   ├── moore/             # Bare-metal Moore Kernel
│   ├── msh/               # Moore Shell (msh)
│   ├── drivers/           # Board drivers (DDR4, SD card, PCAP)
│   └── security/          # PUF, active fences, APAC
├── bitstreams/
│   ├── gpu/               # Rendered Brief GPU
│   └── blanks/            # Blanking bitstreams per RP
├── shell/
│   └── static_shell/      # Vivado project for Static Shell
├── ebv/
│   └── kv260.ebv          # Embedded Brief for KV260
└── tools/
    ├── build.py           # Unified build script
    └── sign.py            # Bitstream signing + encryption
```

### 9.2 Build Commands

```bash
# Build everything
make build

# Build kernel only
make kernel

# Build Brief toolchain only
make brief-toolchain

# Build a specific bitstream
./target/release/brief-compiler build --target kv260 examples/rendered_gpu.bv

# Sign and package a bitstream for deployment
./tools/sign.py --key puf --input rendered_gpu.bvc --output sdcard/

# Deploy to SD card
make deploy DEVICE=/dev/sdX

# Run tests
make test

# Connect to msh over UART
make monitor
```

### 9.3 Test Strategy

```bash
# Unit tests for Brief compiler
cargo test --lib

# Formal verification unit tests
cargo test --package brief-verifier

# Kernel module tests (QEMU emulation for ARM)
cargo test --package moore-kernel

# Integration tests (requires hardware)
make test-integration TARGET=kv260
```

---

## 10. Acceptance Criteria Checklist

| # | Criterion | Verification |
|---|---|---|
| 1 | Moore Kernel boots bare-metal on KV260 from SD card | UART output confirms boot |
| 2 | msh presents Propositional Context on boot | UART shows fabric summary |
| 3 | `Imp_Core.bvc exists_on Tile_0.` mounts bitstream to RP-0 | msh returns `RESULT: True` |
| 4 | `Rendered_GPU.bvc exists_on Tile_0.` drives HDMI display | Physical display shows output |
| 5 | Leakage contract violation causes compile-time abort | `brief-compiler` returns error with counterexample |
| 6 | Active fence activates when bitstream mounts | Logic analyzer shows ROs active in moat |
| 7 | PUF rejects tampered bitstream | Mount fails with cryptographic signature error |
| 8 | Unmount + blanking bitstream reclaims RP cleanly | `absent.` succeeds; RP shows as available |
| 9 | `?` discovery interrogative lists available predicates | Manual verification in msh |
| 10 | Proof-based error message on impossible proposition | `Windows_95 exists_on Tile_0.` → Failed Proof |

---

## 11. Open Decisions

These are intentionally left open for the team to resolve during implementation:

1. **Kernel entry point:** ARM exception vectors — which address does `moore.bin` jump to?
2. **AXI address map:** Which AXI master IDs does the kernel use vs. which are reserved for mounted bitstreams?
3. **SD card partition layout:** Exact FAT32 / raw partition schema for bitstream storage
4. **PCAP vs. ICAP:** Use PCAP (CPU-driven) for MVP simplicity; ICAP (autonomous) for v2 speed
5. **Rendered GPU pixel path:** Kernel push vs. GPU pull from shared BRAM — benchmark both
6. **msh output device:** UART (primary) + HDMI framebuffer (optional) for the propositional context display
7. **Brief compiler language:** Implementation language for the compiler itself (Rust preferred for safety, or OCaml for formal correctness)
8. **Tether protocol format:** How does the `.ebv` expose discoverable properties to msh? (Flat struct? Typed interface?)
