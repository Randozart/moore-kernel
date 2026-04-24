# Brief Control (.bvc) Specification
**Version:** 0.1.0
**Date:** 2026-04-24
**Status:** Draft

---

## 1. Overview

### 1.1 Purpose

Brief Control (`.bvc`) is the orchestration layer for the Moore Kernel. It is the "Will" of the system — the declarative layer that coordinates mounting bitstreams onto FPGA fabric, managing tile resources, and routing interconnects.

`.bvc` sits between the propositional intent (msh propositions) and the physical reality (FPGA fabric). It translates mount requests into PCAP configuration streams.

### 1.2 What .bvc Is Not

- `.bvc` is **not** a general-purpose language — it has exactly one job: describe how bitstreams are placed on the fabric
- `.bvc` does **not** execute at runtime on the FPGA — it compiles to a mount manifest consumed by the Kernel's TPU
- `.bvc` does **not** write application logic — that is the job of `.bv` (Brief)

### 1.3 Compilation Flow

```
.bvc source + .ebv (hardware description)
         │
         ▼
[ bvc-compiler ]
         │
         ├─ Parse .bvc syntax
         ├─ Validate against .ebv constraints
         ├─ Resolve bitstream references (.writ files)
         ├─ Generate mount manifest
         └─ Output: .writ (binary) + manifest.json
         │
         ▼
[ Moore Kernel TPU ]
         │
         ├─ Reads manifest
         ├─ Applies relocation patches
         ├─ Streams via PCAP
         └─ Activates fences
```

---

## 2. Syntax

### 2.1 Keywords

| Keyword | Purpose |
|---|---|
| `using` | Import bitstream package reference |
| `control` | Named control block |
| `target` | Specify target tile(s) |
| `partition` | Place bitstream on target tile |
| `route` | Establish interconnect between tiles |
| `mount` | Explicit mount directive |
| `unmount` | Explicit unmount directive |
| `verify` | Request signature verification |
| `fence` | Configure active fence for partition |
| `timeout` | Mount/unmount timeout threshold |

### 2.2 Grammar

```
bvc_program    ::= using_decls control_block*

using_decls    ::= using IDENT ('.' IDENT)* ';'
control_block  ::= 'control' IDENT '{' control_stmts '}'
control_stmts  ::= (target_stmt | partition_stmt | route_stmt | mount_stmt | unmount_stmt | fence_stmt ';')*

target_stmt    ::= 'target' tile_ref (',' tile_ref)*
tile_ref       ::= IDENT               // e.g., Tile_0
                 | IDENT '..' IDENT    // e.g., Tile_0..Tile_3 (range)

partition_stmt ::= 'partition' using_ref 'across' tile_ref
                 | 'partition' using_ref 'across' tile_ref 'as' slot_id
using_ref      ::= IDENT ('.' IDENT)*  // e.g., Imp_Core, Rendered_GPU

route_stmt     ::= 'route' route_name 'over' port_ref
                 | 'route' route_name 'from' tile_ref 'to' tile_ref 'over' port_ref
route_name     ::= IDENT
port_ref       ::= IDENT               // e.g., Port_0, aurora_0

mount_stmt     ::= 'mount' using_ref 'to' tile_ref ('as' slot_id)?
unmount_stmt   ::= 'unmount' using_ref ('from' tile_ref)?
fence_stmt     ::= 'fence' slot_id ('enable' | 'disable')
timeout_stmt   ::= 'timeout' DURATION '=' INTEGER ('ms' | 's' | 'min')
```

### 2.3 Example Programs

#### Minimal Mount

```brief
using Imp_Core;

control Fabric {
    target Tile_0;
    partition Imp_Core across Tile_0;
}
```

#### Full GPU Mount with Fence

```brief
using Rendered_GPU;
using Blank_RP1;

control Display {
    target Tile_0;
    partition Rendered_GPU across Tile_0 as RP_1;
    fence RP_1 enable;
    route pixel_link from Tile_0 to Tile_0 over Port_0;
}
```

#### Multi-Bitstream Coordination

```brief
using Imp_Core;
using Rendered_GPU;
using Neural_Core;

control Full_System {
    target Tile_0;
    partition Imp_Core across Tile_0 as RP_0;
    fence RP_0 enable;

    target Tile_1;
    partition Rendered_GPU across Tile_1 as RP_1;
    fence RP_1 enable;

    target Tile_2;
    partition Neural_Core across Tile_2 as RP_2;
    fence RP_2 enable;

    route high_speed_link from Tile_0 to Tile_2 over Port_0;
}
```

#### Explicit Mount/Unmount with Timeout

```brief
using Imp_Core;

control Boot_Sequence {
    timeout ms = 5000;

    mount Imp_Core to Tile_0 as RP_0;

    timeout ms = 10000;
    unmount Imp_Core from Tile_0;
}
```

---

## 3. Tether Discovery Protocol

### 3.1 Overview

Tethers are queryable hardware state exposed via `.ebv`. The Kernel's msh uses tethers to answer the question "what is the current state of the fabric?"

Tethers use a SQL-like query syntax defined in `.ebv` and answered by Rust handlers in the Kernel.

### 3.2 Tether Categories

| Category | Description | Example |
|---|---|---|
| `discovery` | Enumerate available tiles and their capacities | `SELECT tile_id, lut_count, dsp_count, bram_mb FROM tiles WHERE connected = true` |
| `status` | Current mount state of a tile | `SELECT slot, bitstream_name, active FROM mounts WHERE tile_id = 0` |
| `storage` | Available bitstream packages on SD card | `SELECT filename, size, verified FROM sdcard WHERE present = true` |
| `security` | Active fence state, PUF status | `SELECT fence_id, active, mode FROM fences WHERE tile_id = 0` |

### 3.3 Tether Query Response Format

All tether queries return JSON:

```json
{
    "query": "fabric_discovery",
    "timestamp": "2026-04-24T12:00:00Z",
    "result": {
        "tiles": [
            {
                "tile_id": 0,
                "lut_count": 256000,
                "dsp_count": 1248,
                "bram_mb": 7.2,
                "connected": true,
                "partitions": ["RP_0", "RP_1", "RP_2", "RP_3"]
            }
        ]
    }
}
```

### 3.4 msh Integration

The `?` (discovery interrogative) in msh uses tethers to enumerate available predicates for a subject:

```
> Tile_0 ?
TILE(0) is a [FABRIC_TILE].
Available predicates: { exists_on, absent, is_active, clear, probe }.
Tether query: SELECT predicates FROM tile_capabilities WHERE tile_id = 0;
```

---

## 4. .writ Format

### 4.1 Overview

`.writ` ("Writ of Execution") is the Moore Kernel's binary format for a mounted bitstream. It is self-describing: a JSON metadata header followed by the raw bitstream body.

### 4.2 Binary Layout

```
+------------------+
| Magic (4 bytes)  |  0x57 0x52 0x49 0x54 ("WRIT")
+------------------+
| Version (2 bytes)|  Little-endian u16
+------------------+
| Metadata Len (4)  |  Little-endian u32 (bytes, not including header)
+------------------+
| Metadata (JSON)   |  Variable length, UTF-8 JSON
+------------------+
| Body (bitstream)  |  Variable length, raw bitstream data
+------------------+
```

### 4.3 Metadata JSON Schema

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "type": "object",
    "required": ["name", "version", "target", "partitions", "security"],
    "properties": {
        "name": {
            "type": "string",
            "description": "Human-readable bitstream name"
        },
        "version": {
            "type": "string",
            "pattern": "^\\d+\\.\\d+\.\\d+$"
        },
        "target": {
            "type": "object",
            "required": ["board", "soc"],
            "properties": {
                "board": { "type": "string" },
                "soc": { "type": "string" }
            }
        },
        "partitions": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["slot", "lut_count", "bram_mb"],
                "properties": {
                    "slot": { "type": "string" },
                    "lut_count": { "type": "integer" },
                    "bram_mb": { "type": "number" },
                    "relocation": {
                        "type": "object",
                        "description": "Base address relocation for this partition"
                    }
                }
            }
        },
        "security": {
            "type": "object",
            "required": ["verified", "signature"],
            "properties": {
                "verified": { "type": "boolean" },
                "signature": { "type": "string" },
                "leakage_contract": { "type": "string" }
            }
        },
        "tethers": {
            "type": "array",
            "description": "Tether points exposed by this bitstream"
        },
        "interconnects": {
            "type": "array",
            "description": "AXI/Aurora interconnect definitions"
        }
    }
}
```

### 4.4 Example .writ Metadata

```json
{
    "name": "Rendered_GPU",
    "version": "0.1.0",
    "target": {
        "board": "Xilinx Kria KV260",
        "soc": "XCZU3CG-SFVC784"
    },
    "partitions": [
        {
            "slot": "RP_1",
            "lut_count": 80000,
            "bram_mb": 2.4,
            "relocation": {
                "base_address": "0x0000_0000"
            }
        }
    ],
    "security": {
        "verified": true,
        "signature": "base64EncodedSignatureHere",
        "leakage_contract": "Rendered_GPU_LC"
    },
    "tethers": [
        {
            "name": "framebuffer_ptr",
            "type": "pointer",
            "address": "0x4000_0000",
            "size": 1228800
        },
        {
            "name": "status_reg",
            "type": "register",
            "address": "0x4000_1000"
        }
    ],
    "interconnects": [
        {
            "name": "pixel_stream",
            "type": "AXI4Stream",
            "master": "GPU",
            "slave": "Kernel",
            "width": 32
        }
    ]
}
```

---

## 5. Error Codes

### 5.1 Mount Errors

| Code | Name | Description |
|---|---|---|
| 0x01 | `MOUNT_OK` | Mount succeeded |
| 0x02 | `MOUNT_ERR_NOT_FOUND` | Bitstream package not found in storage |
| 0x03 | `MOUNT_ERR_VERIFY_FAIL` | Signature verification failed |
| 0x04 | `MOUNT_ERR_LEAKAGE_CONTRACT_VIOLATION` | Bitstream failed leakage contract check |
| 0x05 | `MOUNT_ERR_TILE_NOT_AVAILABLE` | Target tile does not exist or is not connected |
| 0x06 | `MOUNT_ERR_PARTITION_CONFLICT` | Target slot already occupied |
| 0x07 | `MOUNT_ERR_RESOURCE_EXCEEDED` | LUT/BRAM/DSP requirements exceed tile capacity |
| 0x08 | `MOUNT_ERR_TIMEOUT` | PCAP configuration timed out |
| 0x09 | `MOUNT_ERR_DFX_DECOUPLE` | DFX decoupler failed to engage |
| 0x0A | `MOUNT_ERR_ICAP_BUSY` | ICAP/PCAP is busy with another operation |

### 5.2 Unmount Errors

| Code | Name | Description |
|---|---|---|
| 0x10 | `UNMOUNT_OK` | Unmount succeeded |
| 0x11 | `UNMOUNT_ERR_NOT_MOUNTED` | No bitstream mounted at specified slot |
| 0x12 | `UNMOUNT_ERR_TIMEOUT` | Blank bitstream streaming timed out |
| 0x13 | `UNMOUNT_ERR_FENCE_ACTIVE` | Cannot unmount while fence is active (must disable first) |

### 5.3 Tether Errors

| Code | Name | Description |
|---|---|---|
| 0x20 | `TETHER_OK` | Query succeeded |
| 0x21 | `TETHER_ERR_NOT_FOUND` | Requested tether does not exist |
| 0x22 | `TETHER_ERR_TIMEOUT` | Tether handler timed out |
| 0x23 | `TETHER_ERR_INVALID_QUERY` | Malformed SQL-like query |

### 5.4 Security Errors

| Code | Name | Description |
|---|---|---|
| 0x30 | `SEC_OK` | Security check passed |
| 0x31 | `SEC_ERR_PUF_NOT_READY` | PUF not initialized |
| 0x32 | `SEC_ERR_SIGNATURE_INVALID` | Cryptographic signature invalid |
| 0x33 | `SEC_ERR_KEYSTORELocked` | PUF keystore is locked |
| 0x34 | `SEC_ERR_FENCE_CONFLICT` | Fence configuration conflicts |

---

## 6. Tile Discovery Protocol

### 6.1 Discovery Sequence

```
HOST                    KERNEL                     FABRIC
 │                          │                          │
 │  fabric_discovery query  │                          │
 │─────────────────────────>│                          │
 │                          │  AXI read → tile_status  │
 │                          │─────────────────────────>│
 │                          │  <────────────────────────│
 │                          │  tile_state response     │
 │                          │                          │
 │  <────────────────────────│                          │
 │  JSON response           │                          │
 │  {tiles: [...]}          │                          │
```

### 6.2 Discovery Query Format

```
SELECT [property [, property ...]]
FROM tiles
[WHERE condition]
[ORDER BY property [ASC | DESC]]
[LIMIT count]
```

Supported properties:
- `tile_id` — Unique tile identifier
- `lut_count` — Available LUTs
- `dsp_count` — Available DSP slices
- `bram_mb` — Available BRAM in MB
- `connected` — Boolean, whether tile is connected
- `partitions` — Array of partition slot names
- `status` — Current status (IDLE, MOUNTED, ERROR)

Supported conditions:
- `connected = true`
- `lut_count >= <value>`
- `tile_id = <value>`

### 6.3 Example Discovery Queries

Get all available tiles:
```
SELECT tile_id, lut_count, dsp_count, bram_mb FROM tiles WHERE connected = true;
```

Find tiles with sufficient capacity for a bitstream:
```
SELECT tile_id, lut_count FROM tiles WHERE connected = true AND lut_count >= 80000;
```

Get mount status of specific tile:
```
SELECT slot, bitstream_name, active FROM mounts WHERE tile_id = 0;
```

---

## 7. Coordination Logic

### 7.1 Mount Sequence

```
1. Validate bitstream package exists in storage
2. Verify cryptographic signature against PUF-KEK
3. Check leakage contract (if multi-tenant)
4. Parse .writ metadata
5. Check target tile has required capacity
6. Check target slot is available
7. TPU calculates relocation coordinates
8. Activate DFX decoupler for target slot
9. Stream bitstream via PCAP
10. Wait for STARTUP sequence
11. Deactivate DFX decoupler
12. Assert ENA (enable) for partition
13. Activate active fence (if configured)
14. Return MOUNT_OK
```

### 7.2 Unmount Sequence

```
1. Deassert ENA for partition
2. Assert RST for partition
3. Activate DFX decoupler for target slot
4. Stream blanking bitstream via PCAP
5. Wait for STARTUP sequence
6. Deactivate DFX decoupler
7. Deactivate active fence (if active)
8. Update mount registry
9. Return UNMOUNT_OK
```

### 7.3 Conflict Detection

The bvc-compiler detects conflicts at compile time:

- **Partition conflict**: Two bitstreams targeting same slot
- **Resource overflow**: Bitstream requires more LUTs/BRAM than tile provides
- **Fence conflict**: Requesting fence on slot that doesn't support it
- **Route conflict**: Two routes using same port

The Kernel enforces at runtime:
- **Race condition**: Two mount requests for same slot (mutex)
- **Stale handle**: Unmount after already unmounted
- **Timeout**: Operation exceeds timeout threshold

---

## 8. Open Decisions

1. **Flat vs. hierarchical metadata**: Current .writ uses flat JSON. Should it use a nested structure matching the .ebv hierarchy?
2. **Binary vs. text manifest**: Current design uses binary .writ with JSON metadata. Should there also be a human-readable `.writ.txt` manifest?
3. **Pull vs. push pixel path**: Current design assumes kernel pushes pixels to GPU. Should GPU pull from shared BRAM?
4. **Tether authentication**: Should tethers require authentication (PUF-based) to prevent reconnaissance?
5. **Atomic multi-mount**: Should a single `.bvc` with multiple partitions mount all or nothing? (Current: all-or-nothing)

---

## 9. Reference Implementation

The bvc-compiler is implemented as `bvc-compiler` crate under `brief-control/bvc/`. It consumes:
- `.bvc` source files
- `.ebv` hardware descriptions
- `.writ` bitstream packages (as references)

And produces:
- Mount manifest (JSON)
- Patched `.writ` with relocation applied
- Security proof record

### 9.1 Build Command

```bash
cargo build -p bvc-compiler
```

### 9.2 Usage

```bash
./target/release/bvc-compiler build example.bvc --ebv kv260.ebv --out manifest.json
```

---

## 10. Relationship to Other Specifications

| Specification | File | Purpose |
|---|---|---|
| Moore Kernel MVP | `SPEC.md` | Full system overview |
| Brief Language | `brief-compiler/CLAUDE.md` | `.bv` syntax and semantics |
| KV260 Hardware | `ebv/kv260.ebv` | Hardware description + tethers |
| **Brief Control** | `BRIEF_CONTROL_SPEC.md` | **This document** — `.bvc` orchestration |

---

## 11. Revision History

| Version | Date | Changes |
|---|---|---|
| 0.1.0 | 2026-04-24 | Initial draft |
