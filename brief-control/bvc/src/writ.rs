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

    #[test]
    fn test_writ_builder_basic() {
        let mut builder = WritBuilder::new();
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
}