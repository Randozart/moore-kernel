use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

pub mod parser;
pub mod validator;
pub mod writ;

pub use parser::{BvcProgram, parse_bvc};
pub use validator::Validator;
pub use writ::{WritBuilder, WritMetadata, PartitionSpec, SecuritySpec, InterconnectDef, TetherDef};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlBlock {
    pub name: String,
    pub stmts: Vec<ControlStmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControlStmt {
    Target(Vec<String>),
    Partition(PartitionStmt),
    Route(RouteStmt),
    Mount(MountStmt),
    Unmount(UnmountStmt),
    Fence(FenceStmt),
    Timeout(TimeoutStmt),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionStmt {
    pub using_ref: String,
    pub tile_ref: String,
    pub slot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteStmt {
    pub route_name: String,
    pub from_tile: Option<String>,
    pub to_tile: Option<String>,
    pub port_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MountStmt {
    pub using_ref: String,
    pub tile_ref: String,
    pub slot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnmountStmt {
    pub using_ref: String,
    pub tile_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FenceStmt {
    pub slot_id: String,
    pub action: FenceAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FenceAction {
    Enable,
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutStmt {
    pub value: u64,
    pub unit: TimeoutUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeoutUnit {
    Ms,
    Sec,
    Min,
}

pub fn compile_bvc(bvc_path: &Path, ebv_path: &Path, out_path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(bvc_path)
        .with_context(|| format!("Failed to read .bvc file: {}", bvc_path.display()))?;

    let ebv_source = std::fs::read_to_string(ebv_path)
        .with_context(|| format!("Failed to read .ebv file: {}", ebv_path.display()))?;

    let program = parse_bvc(&source).context("Failed to parse .bvc")?;

    let ebv_data: EbvData = toml::from_str(&ebv_source)
        .context("Failed to parse .ebv as TOML")?;

    let validator = Validator::new(&ebv_data);
    validator.validate(&program).context("Validation failed")?;

    let mut builder = WritBuilder::new();
    let metadata = builder.build_from_program(&program, &ebv_data)?;
    let writ_bytes = builder.to_writ_bytes(&metadata)?;

    std::fs::write(out_path, writ_bytes)
        .with_context(|| format!("Failed to write .writ file: {}", out_path.display()))?;

    let manifest_path = out_path.with_extension("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&manifest_path, manifest_json)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct EbvData {
    pub ebv: EbvHeader,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EbvHeader {
    pub board: Option<String>,
    pub soc: Option<String>,
    pub partitions: Option<HashMap<String, PartitionDef>>,
    pub tethers: Option<HashMap<String, EbvTetherDef>>,
    pub moats: Option<HashMap<String, MoatDef>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionDef {
    #[serde(rename = "cells")]
    pub lut_count: Option<u32>,
    #[serde(rename = "name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EbvTetherDef {
    #[serde(rename = "type")]
    pub tether_type: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoatDef {
    #[serde(rename = "width")]
    pub width: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writ_magic_bytes() {
        let builder = WritBuilder::new();
        let metadata = WritMetadata {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            target: None,
            partitions: vec![],
            security: SecuritySpec {
                verified: true,
                signature: "test".to_string(),
                leakage_contract: None,
            },
            tethers: vec![],
            interconnects: vec![],
        };
        let bytes = builder.to_writ_bytes(&metadata).unwrap();
        assert_eq!(&bytes[0..4], b"WRIT");
    }
}