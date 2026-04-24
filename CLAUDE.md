# Moore Kernel — Agent Guidelines

## Project Overview

The Moore Kernel is a bare-metal operating system for AMD/Xilinx Zynq UltraScale+ FPGAs that treats FPGA bitstreams as first-class physical processes. It is the foundational OS of a post-Von Neumann computing paradigm where programs are compiled into physical silicon and mounted onto reconfigurable fabric.

## Key Principles

**CONTRACT-FIRST**: Contracts are the source of truth. Never weaken contracts to match lazy code.

### Anti-Patterns (NEVER DO)
- Changing `[product > 0]` to `[true]` because code doesn't set product
- Using generic contracts like `[true]` that pass everything
- Adding postconditions that don't guarantee specific outcomes

### Correct Approach
- Keep contract `[product > 0]`
- Fix code: make buttons call product-specific transactions like `add_laptop`, `add_keyboard`

## File Types
- **.bv** - Pure Brief (specification only, no view)
- **.rbv** - Rendered Brief (Brief + View, compiles to frontend)
- **.ebv** - Embedded Brief (chip drivers, tethers, per-board)
- **.bvc** - Brief Control (orchestration layer, mount manifests)

## The Language Stack
- **Brief (.bv)**: Pure logic, contract-based, transpiles to SystemVerilog
- **Embedded Brief (.ebv)**: Chip drivers and tethers — the board's "birth certificate"
- **Brief Control (.bvc)**: Orchestration — the "Will" that commands the fabric
- **Moore Kernel**: The bare-metal OS — the "Body"
- **Moore Shell (msh)**: Predicate-logic CLI — declarative, no magic words

## Commands
- **Build**: `make build` or `cargo build --release`
- **Test**: `make test` or `cargo test --lib`
- **Deploy**: `make deploy DEVICE=/dev/sdX`
- **Monitor**: `make monitor` (UART @ 115200)

## Target Hardware
- **Primary**: Xilinx Kria KV260 (Zynq UltraScale+ MPSoC)
- **Toolchain**: Vivado for synthesis/PnR; Yosys+nextpnr for simulation

## Security Model
Security is non-negotiable from day one:
- All bitstreams must pass leakage contract verification before mounting
- Bitstreams are PUF-encrypted at rest
- Active fences (Ring Oscillators in moat zones) blind side-channel sensors
- Multi-tenant isolation enforced via SHELL/ROLE architecture

## Moore Shell (msh) Principles
- msh is a **Propositional Interface** — not a command line
- The user proposes states of the world; the kernel makes them true
- No magic words. All names come from `.ebv` and `.bvc` files
- Errors are **Failed Proofs** — mathematical traces of why a proposition cannot be satisfied
- Use `?` to discover what predicates are valid for any subject
