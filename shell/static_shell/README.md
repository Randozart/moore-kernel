# Static Shell - Xilinx Vivado Project for KV260
# This project defines the immutable body of the Moore Tile

## Overview

The Static Shell is pre-synthesized and never changes during runtime. It provides:
- All Reconfigurable Partition (RP) boundaries and floorplanning
- DFX Decoupler IP at every partition boundary
- AXI NoC / crossbar connecting PS to PL
- Memory controllers (DDR4)
- HDMI TX controller (fixed, not reconfigurable)
- PCAP configuration interface
- ICAP for autonomous reconfiguration

## Project Structure

```
shell/
└── static_shell/
    ├── static_shell.xpr          # Vivado project file
    ├── constraints/
    │   ├── io.xdc                 # Pin assignments
    │   ├── timing.xdc             # Timing constraints
    │   └── config.xdc              # Configuration constraints
    ├── ip/
    │   ├── dfx_decoupler/
    │   │   └── dfx_decoupler.xci
    │   ├── axi_crossbar/
    │   │   └── axi_crossbar.xci
    │   ├── pcap_controller/
    │   │   └── pcap_ctrl.xci
    │   └── hdmi_tx/
    │       └── hdmi_tx.xci
    ├── rtl/
    │   ├── shell_top.sv          # Top-level shell
    │   ├── axi_interconnect.sv   # AXI NoC
    │   ├── partition_boundary.sv # RP boundary logic
    │   └── fence_controller.sv   # Active fence control
    └── scripts/
        ├── build.tcl              # Build script
        └── synthesize.tcl         # Synthesis script
```

## Floorplanning

| Region | Coordinates | Size | Purpose |
|--------|-------------|------|---------|
| RP_0 | (0,0) - (50,50) | 40K LUTs | Kernel ops |
| RP_1 | (50,0) - (150,50) | 80K LUTs | App slot 1 |
| RP_2 | (150,0) - (250,50) | 80K LUTs | App slot 2 |
| RP_3 | (0,50) - (50,75) | 40K LUTs | Reserved |
| SHELL_NOC | (0,75) - (250,100) | 25K LUTs | AXI NoC |
| SHELL_DEC | (0,0) - (25,75) | 20K LUTs | Decouplers |

## AXI Address Map (PS side)

| Master | Base Address | Size | Purpose |
|--------|--------------|------|---------|
| DDR4_0 | 0x0000_0000 | 2 GiB | Main memory |
| OCM | 0xFFF0_0000 | 256 KiB | On-chip memory |
| PCAP | 0xFF0A_0000 | 16 KiB | Configuration |
| GPIO | 0xFF0B_0000 | 4 KiB | LEDs/Buttons |
| UART0 | 0xFF000000 | 4 KiB | Console |

## DFX Decoupler Configuration

Each RP boundary has a pair of DFX decouplers:
- Input decoupler: isolates RP inputs during configuration
- Output decoupler: isolates RP outputs during configuration

## Build Commands

```bash
# Open in Vivado
vivado -project static_shell.xpr

# Build using TCL
source scripts/build.tcl
```

## Notes

- This project requires Xilinx Vivado 2024.1 or later
- Synthesis takes 4-18 hours on high-end workstation
- DO NOT modify after initial synthesis - it's the immutable body