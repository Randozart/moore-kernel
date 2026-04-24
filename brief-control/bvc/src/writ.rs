// bvc/writ.rs - BVC Binary Format Writer
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Writes .writ binary files from BVC programs

use crate::{BvcProgram, ControlBlock, ControlStmt, EbvData, PartitionDef, TimeoutUnit};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub struct WritBuilder {
    partition_counter: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritMetadata {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TargetSpec>,
    pub partitions: Vec<PartitionSpec>,
    pub security: SecuritySpec,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tethers: Vec<TetherDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interconnects: Vec<InterconnectDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSpec {
    pub board: String,
    pub soc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionSpec {
    pub slot: String,
    pub lut_count: u32,
    pub bram_mb: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relocation: Option<RelocationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationSpec {
    pub base_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub verified: bool,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leakage_contract: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetherDef {
    pub name: String,
    #[serde(rename = "type")]
    pub tether_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterconnectDef {
    pub name: String,
    #[serde(rename = "type")]
    pub interconnect_type: String,
    pub master: String,
    pub slave: String,
    pub width: u32,
}

impl WritBuilder {
    pub fn new() -> Self {
        Self { partition_counter: 0 }
    }

    pub fn build_from_program(&mut self, program: &BvcProgram, ebv: &EbvData) -> Result<WritMetadata> {
        let first_control = program.control_blocks.first()
            .ok_or_else(|| anyhow::anyhow!("No control blocks in .bvc program"))?;

        let mut partitions = Vec::new();
        let mut tethers = Vec::new();
        let mut interconnects = Vec::new();

        for stmt in &first_control.stmts {
            match stmt {
                ControlStmt::Partition(p) => {
                    let slot = p.slot_id.clone().unwrap_or_else(|| format!("RP_{}", self.partition_counter));
                    self.partition_counter += 1;

                    let (lut_count, bram_mb) = self.lookup_partition_capacity(&slot, ebv)?;

                    partitions.push(PartitionSpec {
                        slot,
                        lut_count,
                        bram_mb,
                        relocation: None,
                    });
                }
                ControlStmt::Route(r) => {
                    interconnects.push(InterconnectDef {
                        name: r.route_name.clone(),
                        interconnect_type: "AXI4Stream".to_string(),
                        master: r.from_tile.clone().unwrap_or_default(),
                        slave: r.to_tile.clone().unwrap_or_default(),
                        width: 32,
                    });
                }
                _ => {}
            }
        }

        Ok(WritMetadata {
            name: first_control.name.clone(),
            version: "0.1.0".to_string(),
            target: Some(TargetSpec {
                board: ebv.ebv.board.clone().unwrap_or_else(|| "Unknown".to_string()),
                soc: ebv.ebv.soc.clone().unwrap_or_else(|| "Unknown".to_string()),
            }),
            partitions,
            security: SecuritySpec {
                verified: false,
                signature: String::new(),
                leakage_contract: None,
            },
            tethers,
            interconnects,
        })
    }

    fn lookup_partition_capacity(&self, slot: &str, ebv: &EbvData) -> Result<(u32, f64)> {
        let partitions = ebv.ebv.partitions.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No partitions in .ebv"))?;

        let def = partitions.get(slot)
            .ok_or_else(|| anyhow::anyhow!("Partition '{}' not found in .ebv", slot))?;

        let lut_count = def.lut_count.unwrap_or(40000);
        let bram_mb = 2.4;

        Ok((lut_count, bram_mb))
    }

    pub fn to_writ_bytes(&self, metadata: &WritMetadata) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        output.write_all(b"WRIT")?;

        let version: u16 = 1;
        output.write_all(&version.to_le_bytes())?;

        let json = serde_json::to_string(metadata)?;
        let json_bytes = json.as_bytes();
        let json_len = json_bytes.len() as u32;
        output.write_all(&json_len.to_le_bytes())?;

        output.write_all(json_bytes)?;

        output.write_all(&[0x00; 16][..])?;

        Ok(output)
    }
}

impl Default for WritBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_bvc, EbvData, EbvHeader, PartitionDef, HashMap};

    fn make_test_ebv() -> EbvData {
        let mut partitions = HashMap::new();
        partitions.insert("Tile_0".to_string(), PartitionDef { lut_count: Some(40000), name: Some("kernel_ops".to_string()) });
        partitions.insert("Tile_1".to_string(), PartitionDef { lut_count: Some(80000), name: Some("app_slot_1".to_string()) });
        partitions.insert("RP_0".to_string(), PartitionDef { lut_count: Some(40000), name: Some("kernel_ops".to_string()) });
        partitions.insert("RP_1".to_string(), PartitionDef { lut_count: Some(80000), name: Some("app_slot_1".to_string()) });

        let mut tethers = HashMap::new();
        tethers.insert("Port_0".to_string(), crate::EbvTetherDef { tether_type: Some("axi".to_string()), query: None });
        tethers.insert("aurora_0".to_string(), crate::EbvTetherDef { tether_type: Some("aurora".to_string()), query: None });

        EbvData {
            ebv: EbvHeader {
                board: Some("KV260".to_string()),
                soc: Some("XCZU3CG".to_string()),
                partitions: Some(partitions),
                tethers: Some(tethers),
                moats: None,
            }
        }
    }

    #[test]
    fn test_writ_builder_basic() {
        let builder = WritBuilder::new();
        let metadata = WritMetadata {
            name: "TestModule".to_string(),
            version: "0.1.0".to_string(),
            target: Some(TargetSpec {
                board: "KV260".to_string(),
                soc: "XCZU3CG".to_string(),
            }),
            partitions: vec![],
            security: SecuritySpec {
                verified: true,
                signature: "sig".to_string(),
                leakage_contract: None,
            },
            tethers: vec![],
            interconnects: vec![],
        };
        let bytes = builder.to_writ_bytes(&metadata).unwrap();
        assert_eq!(&bytes[0..4], b"WRIT");
        assert!(bytes.len() > 22);
    }

    #[test]
    fn test_writ_magic_bytes() {
        let builder = WritBuilder::new();
        let metadata = WritMetadata {
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            target: None,
            partitions: vec![],
            security: SecuritySpec {
                verified: false,
                signature: String::new(),
                leakage_contract: None,
            },
            tethers: vec![],
            interconnects: vec![],
        };
        let bytes = builder.to_writ_bytes(&metadata).unwrap();
        assert_eq!(&bytes[0..4], b"WRIT");
    }

    #[test]
    fn test_writ_version_bytes() {
        let builder = WritBuilder::new();
        let metadata = WritMetadata {
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            target: None,
            partitions: vec![],
            security: SecuritySpec {
                verified: false,
                signature: String::new(),
                leakage_contract: None,
            },
            tethers: vec![],
            interconnects: vec![],
        };
        let bytes = builder.to_writ_bytes(&metadata).unwrap();
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        assert_eq!(version, 1);
    }

    #[test]
    fn test_writ_json_metadata() {
        let builder = WritBuilder::new();
        let metadata = WritMetadata {
            name: "TestModule".to_string(),
            version: "0.1.0".to_string(),
            target: Some(TargetSpec {
                board: "KV260".to_string(),
                soc: "XCZU3CG".to_string(),
            }),
            partitions: vec![
                PartitionSpec {
                    slot: "RP_0".to_string(),
                    lut_count: 40000,
                    bram_mb: 2.4,
                    relocation: None,
                },
            ],
            security: SecuritySpec {
                verified: true,
                signature: "abc123".to_string(),
                leakage_contract: Some("TestLC".to_string()),
            },
            tethers: vec![],
            interconnects: vec![],
        };
        let bytes = builder.to_writ_bytes(&metadata).unwrap();
        assert_eq!(&bytes[0..4], b"WRIT");
        let json_len = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;
        assert!(json_len > 0);
        assert!(json_len < 10000);
        let json_bytes = &bytes[10..10 + json_len];
        let json_str = String::from_utf8(json_bytes.to_vec()).unwrap();
        assert!(json_str.contains("TestModule"));
        assert!(json_str.contains("KV260"));
    }

    #[test]
    fn test_build_from_program_single_partition() {
        let source = r#"
using Imp_Core;
control Fabric {
    partition Imp_Core across Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert_eq!(metadata.name, "Fabric");
        assert_eq!(metadata.partitions.len(), 1);
        assert_eq!(metadata.partitions[0].slot, "RP_0");
        assert_eq!(metadata.partitions[0].lut_count, 40000);
    }

    #[test]
    fn test_build_from_program_multiple_partitions() {
        let source = r#"
using Imp_Core;
using Rendered_GPU;
control Full_System {
    partition Imp_Core across Tile_0 as RP_0;
    partition Rendered_GPU across Tile_1 as RP_1;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert_eq!(metadata.partitions.len(), 2);
        assert_eq!(metadata.partitions[0].slot, "RP_0");
        assert_eq!(metadata.partitions[1].slot, "RP_1");
    }

    #[test]
    fn test_build_from_program_with_routes() {
        let source = r#"
using Imp_Core;
control Fabric {
    route pixel_link from Tile_0 to Tile_0 over Port_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert_eq!(metadata.interconnects.len(), 1);
        assert_eq!(metadata.interconnects[0].name, "pixel_link");
        assert_eq!(metadata.interconnects[0].interconnect_type, "AXI4Stream");
    }

    #[test]
    fn test_build_from_program_auto_slot_naming() {
        let source = r#"
using Imp_Core;
control Fabric {
    partition Imp_Core across Tile_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert_eq!(metadata.partitions.len(), 1);
        assert_eq!(metadata.partitions[0].slot, "RP_0");
    }

    #[test]
    fn test_build_from_program_target_info() {
        let source = r#"
using Imp_Core;
control Fabric {
    partition Imp_Core across Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert!(metadata.target.is_some());
        assert_eq!(metadata.target.as_ref().unwrap().board, "KV260");
        assert_eq!(metadata.target.as_ref().unwrap().soc, "XCZU3CG");
    }

    #[test]
    fn test_build_from_program_security_defaults() {
        let source = r#"
using Imp_Core;
control Fabric {
    partition Imp_Core across Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let mut builder = WritBuilder::new();
        let metadata = builder.build_from_program(&program, &ebv).unwrap();

        assert!(!metadata.security.verified);
        assert!(metadata.security.signature.is_empty());
    }
}