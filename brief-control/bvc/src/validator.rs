// bvc/validator.rs - BVC Program Validator
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Validates BVC programs against EBV hardware descriptions

use crate::{ControlStmt, BvcProgram, EbvData};
use anyhow::{bail, Result};
use std::collections::HashSet;

pub struct Validator<'a> {
    ebv: &'a EbvData,
}

impl<'a> Validator<'a> {
    pub fn new(ebv: &'a EbvData) -> Self {
        Self { ebv }
    }

    pub fn validate(&self, program: &BvcProgram) -> Result<()> {
        for block in &program.control_blocks {
            self.validate_control_block(block)?;
        }
        self.validate_no_partition_conflicts(program)?;
        self.validate_no_route_conflicts(program)?;
        Ok(())
    }

    fn validate_control_block(&self, block: &crate::ControlBlock) -> Result<()> {
        for stmt in &block.stmts {
            self.validate_stmt(stmt)?;
        }
        Ok(())
    }

    fn validate_stmt(&self, stmt: &crate::ControlStmt) -> Result<()> {
        match stmt {
            crate::ControlStmt::Target(tiles) => {
                for tile in tiles {
                    self.validate_tile_exists(tile)?;
                }
            }
            crate::ControlStmt::Partition(partition) => {
                self.validate_tile_exists(&partition.tile_ref)?;
                self.validate_slot_available(&partition.slot_id)?;
            }
            crate::ControlStmt::Route(route) => {
                if let Some(from) = &route.from_tile {
                    self.validate_tile_exists(from)?;
                }
                if let Some(to) = &route.to_tile {
                    self.validate_tile_exists(to)?;
                }
                self.validate_port_exists(&route.port_ref)?;
            }
            crate::ControlStmt::Mount(mount) => {
                self.validate_tile_exists(&mount.tile_ref)?;
                self.validate_slot_available(&mount.slot_id)?;
            }
            crate::ControlStmt::Unmount(_unmount) => {}
            crate::ControlStmt::Fence(fence) => {
                self.validate_slot_exists(&fence.slot_id)?;
            }
            crate::ControlStmt::Timeout(_) => {}
        }
        Ok(())
    }

    fn validate_tile_exists(&self, tile: &str) -> Result<()> {
        let partitions = self.ebv.ebv.partitions.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No partitions defined in .ebv"))?;
        if !partitions.contains_key(tile) {
            bail!("Tile '{}' not found in .ebv partitions", tile);
        }
        Ok(())
    }

    fn validate_slot_available(&self, slot_id: &Option<String>) -> Result<()> {
        if slot_id.is_none() {
            return Ok(());
        }
        let slot = slot_id.as_ref().unwrap();
        let partitions = self.ebv.ebv.partitions.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No partitions defined in .ebv"))?;
        if !partitions.contains_key(slot) {
            bail!("Slot '{}' not found in .ebv partitions", slot);
        }
        Ok(())
    }

    fn validate_slot_exists(&self, slot_id: &str) -> Result<()> {
        let partitions = self.ebv.ebv.partitions.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No partitions defined in .ebv"))?;
        if !partitions.contains_key(slot_id) {
            bail!("Slot '{}' not found in .ebv partitions", slot_id);
        }
        Ok(())
    }

    fn validate_port_exists(&self, port: &str) -> Result<()> {
        let tethers = self.ebv.ebv.tethers.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No tethers defined in .ebv"))?;
        if !tethers.contains_key(port) {
            bail!("Port '{}' not found in .ebv tethers", port);
        }
        Ok(())
    }

    fn validate_no_partition_conflicts(&self, program: &BvcProgram) -> Result<()> {
        let mut used_slots: HashSet<String> = HashSet::new();
        let mut used_tiles: HashSet<String> = HashSet::new();

        for block in &program.control_blocks {
            for stmt in &block.stmts {
                match stmt {
                    ControlStmt::Partition(p) => {
                        if let Some(slot) = &p.slot_id {
                            if used_slots.contains(slot) {
                                bail!("Partition conflict: slot '{}' used multiple times", slot);
                            }
                            used_slots.insert(slot.clone());
                        }
                        if used_tiles.contains(&p.tile_ref) {
                            bail!("Tile conflict: tile '{}' has multiple partitions", p.tile_ref);
                        }
                        used_tiles.insert(p.tile_ref.clone());
                    }
                    ControlStmt::Mount(m) => {
                        if let Some(slot) = &m.slot_id {
                            if used_slots.contains(slot) {
                                bail!("Mount conflict: slot '{}' already in use", slot);
                            }
                            used_slots.insert(slot.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn validate_no_route_conflicts(&self, program: &BvcProgram) -> Result<()> {
        let mut used_ports: HashSet<String> = HashSet::new();

        for block in &program.control_blocks {
            for stmt in &block.stmts {
                if let ControlStmt::Route(r) = stmt {
                    if used_ports.contains(&r.port_ref) {
                        bail!("Route conflict: port '{}' used by multiple routes", r.port_ref);
                    }
                    used_ports.insert(r.port_ref.clone());
                }
            }
        }
        Ok(())
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
        partitions.insert("Tile_2".to_string(), PartitionDef { lut_count: Some(80000), name: Some("app_slot_2".to_string()) });
        partitions.insert("RP_0".to_string(), PartitionDef { lut_count: Some(40000), name: Some("kernel_ops".to_string()) });
        partitions.insert("RP_1".to_string(), PartitionDef { lut_count: Some(80000), name: Some("app_slot_1".to_string()) });
        partitions.insert("RP_2".to_string(), PartitionDef { lut_count: Some(80000), name: Some("app_slot_2".to_string()) });
        partitions.insert("RP_3".to_string(), PartitionDef { lut_count: Some(40000), name: Some("reserved".to_string()) });

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
    fn test_validate_valid_program() {
        let source = r#"
using Imp_Core;
control Fabric {
    target Tile_0;
    partition Imp_Core across Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        assert!(validator.validate(&program).is_ok());
    }

    #[test]
    fn test_validate_invalid_tile() {
        let source = r#"
using Imp_Core;
control Fabric {
    target Invalid_Tile;
    partition Imp_Core across Invalid_Tile;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        assert!(validator.validate(&program).is_err());
    }

    #[test]
    fn test_validate_partition_conflict() {
        let source = r#"
using Imp_Core;
using Rendered_GPU;
control Fabric {
    partition Imp_Core across Tile_0 as RP_0;
    partition Rendered_GPU across Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        let result = validator.validate(&program);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Partition conflict") || err_msg.contains("already in use"));
    }

    #[test]
    fn test_validate_tile_conflict() {
        let source = r#"
using Imp_Core;
using Rendered_GPU;
control Fabric {
    partition Imp_Core across Tile_0 as RP_0;
    partition Rendered_GPU across Tile_0 as RP_1;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        let result = validator.validate(&program);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Tile conflict") || err_msg.contains("multiple partitions"));
    }

    #[test]
    fn test_validate_route_conflict() {
        let source = r#"
using Imp_Core;
control Fabric {
    route link1 over Port_0;
    route link2 over Port_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        let result = validator.validate(&program);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Route conflict") || err_msg.contains("used by multiple"));
    }

    #[test]
    fn test_validate_unknown_port() {
        let source = r#"
using Imp_Core;
control Fabric {
    route link1 over Unknown_Port;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        let result = validator.validate(&program);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Port") && err_msg.contains("not found"));
    }

    #[test]
    fn test_validate_unknown_slot() {
        let source = r#"
using Imp_Core;
control Fabric {
    partition Imp_Core across Tile_0 as Unknown_Slot;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        let result = validator.validate(&program);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_validate_fence_on_valid_slot() {
        let source = r#"
using Imp_Core;
control Fabric {
    fence RP_0 enable;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        assert!(validator.validate(&program).is_ok());
    }

    #[test]
    fn test_validate_multi_tile_program() {
        let source = r#"
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
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        assert!(validator.validate(&program).is_ok());
    }

    #[test]
    fn test_validate_mount_without_conflict() {
        let source = r#"
using Imp_Core;
control Boot {
    mount Imp_Core to Tile_0 as RP_0;
}
"#;
        let program = parse_bvc(source).unwrap();
        let ebv = make_test_ebv();
        let validator = Validator::new(&ebv);
        assert!(validator.validate(&program).is_ok());
    }
}