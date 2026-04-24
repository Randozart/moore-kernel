use crate::{BvcProgram, EbvData};
use anyhow::{bail, Result};

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
        if !partitions.contains_key(tile) && tile != "Tile_0" {
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
}