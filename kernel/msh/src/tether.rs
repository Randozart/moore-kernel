use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetherResponse {
    pub query: String,
    pub timestamp: String,
    pub result: TetherResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TetherResult {
    Tiles(Vec<TileInfo>),
    Mounts(Vec<MountInfo>),
    Storage(Vec<StorageInfo>),
    Fences(Vec<FenceInfo>),
    Memory(Vec<MemoryInfo>),
    Kernel(KernelInfo),
    Fabric(Vec<FabricState>),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileInfo {
    pub tile_id: u32,
    pub lut_count: u32,
    pub dsp_count: u32,
    pub bram_mb: f64,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    pub slot: String,
    pub bitstream_name: Option<String>,
    pub active: bool,
    pub lut_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub filename: String,
    pub size: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenceInfo {
    pub fence_id: String,
    pub active: bool,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub region: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    pub version: String,
    pub build_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricState {
    pub tile_id: u32,
    pub state: String,
    pub mounted_bitstream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropositionalContext {
    pub kernel_version: String,
    pub total_luts: u32,
    pub available_luts: u32,
    pub mounted: Vec<MountInfo>,
    pub storage: Vec<StorageInfo>,
}

impl PropositionalContext {
    pub fn new() -> Self {
        Self {
            kernel_version: "0.1.0".to_string(),
            total_luts: 256_000,
            available_luts: 256_000,
            mounted: vec![],
            storage: vec![],
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        s.push_str("MOORE SHELL v0.1\n");
        s.push_str("══════════════════════════════════════════════════════════\n\n");
        s.push_str("FABRIC:\n");
        s.push_str(&format!("- TOTAL CAPACITY: {} LUTs\n", self.total_luts));
        s.push_str(&format!("- AVAILABLE:        {} LUTs\n", self.available_luts));
        s.push_str("\nSTORAGE (SD Card):\n");
        if self.storage.is_empty() {
            s.push_str("- (empty)\n");
        } else {
            for info in &self.storage {
                s.push_str(&format!("- {}  [{} bytes] [{}]\n",
                    info.filename,
                    info.size,
                    if info.verified { "VERIFIED" } else { "UNVERIFIED" }
                ));
            }
        }
        s.push_str("\nMOUNTED:\n");
        if self.mounted.is_empty() {
            s.push_str("- None.\n");
        } else {
            for m in &self.mounted {
                if let Some(name) = &m.bitstream_name {
                    s.push_str(&format!("- {} on {}  [{} LUTs] [{}]\n",
                        name, m.slot, m.lut_count,
                        if m.active { "ACTIVE" } else { "INACTIVE" }
                    ));
                }
            }
        }
        s.push_str("\nPROPOSITIONAL CONTEXT READY.\n");
        s.push_str("WHAT IS YOUR PROPOSITION?\n");
        s
    }
}

impl Default for PropositionalContext {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TetherEngine {
    fabric: HashMap<String, TileInfo>,
    mounts: HashMap<String, MountInfo>,
    storage: Vec<StorageInfo>,
    fences: HashMap<String, FenceInfo>,
}

impl TetherEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            fabric: HashMap::new(),
            mounts: HashMap::new(),
            storage: vec![],
            fences: HashMap::new(),
        };
        engine.init_kv260();
        engine
    }

    fn init_kv260(&mut self) {
        self.fabric.insert("Tile_0".to_string(), TileInfo {
            tile_id: 0,
            lut_count: 256_000,
            dsp_count: 1248,
            bram_mb: 7.2,
            connected: true,
        });
        self.fabric.insert("RP_0".to_string(), TileInfo {
            tile_id: 0,
            lut_count: 40_000,
            dsp_count: 0,
            bram_mb: 1.8,
            connected: true,
        });
        self.fabric.insert("RP_1".to_string(), TileInfo {
            tile_id: 0,
            lut_count: 80_000,
            dsp_count: 0,
            bram_mb: 2.4,
            connected: true,
        });
        self.mounts.insert("RP_0".to_string(), MountInfo {
            slot: "RP_0".to_string(),
            bitstream_name: None,
            active: false,
            lut_count: 40_000,
        });
        self.mounts.insert("RP_1".to_string(), MountInfo {
            slot: "RP_1".to_string(),
            bitstream_name: None,
            active: false,
            lut_count: 80_000,
        });
        self.storage.push(StorageInfo {
            filename: "Imp_Core.writ".to_string(),
            size: 1_234_567,
            verified: true,
        });
        self.storage.push(StorageInfo {
            filename: "Rendered_GPU.writ".to_string(),
            size: 5_678_901,
            verified: true,
        });
    }

    pub fn get_context(&self) -> PropositionalContext {
        let mut ctx = PropositionalContext::new();
        ctx.mounted = self.mounts.values()
            .filter(|m| m.bitstream_name.is_some())
            .cloned()
            .collect();
        ctx.storage = self.storage.clone();
        if let Some(tile) = self.fabric.get("Tile_0") {
            ctx.total_luts = tile.lut_count;
            let used: u32 = self.mounts.values()
                .filter(|m| m.bitstream_name.is_some())
                .map(|m| m.lut_count)
                .sum();
            ctx.available_luts = tile.lut_count - used;
        }
        ctx
    }

    pub fn query(&self, tether_name: &str) -> Result<TetherResponse> {
        let result = match tether_name {
            "fabric_state" => {
                TetherResult::Fabric(self.fabric.iter().map(|(k, v)| FabricState {
                    tile_id: v.tile_id,
                    state: if v.connected { "CONNECTED" } else { "DISCONNECTED" }.to_string(),
                    mounted_bitstream: self.mounts.get(k).and_then(|m| m.bitstream_name.clone()),
                }).collect())
            }
            "mount_table" => {
                TetherResult::Mounts(self.mounts.values().cloned().collect())
            }
            "storage_list" => {
                TetherResult::Storage(self.storage.clone())
            }
            "fence_status" => {
                TetherResult::Fences(self.fences.values().cloned().collect())
            }
            "kernel_version" => {
                TetherResult::Kernel(KernelInfo {
                    version: "0.1.0".to_string(),
                    build_date: "2026-04-24".to_string(),
                })
            }
            _ => bail!("Unknown tether: {}", tether_name),
        };
        Ok(TetherResponse {
            query: tether_name.to_string(),
            timestamp: "2026-04-24T12:00:00Z".to_string(),
            result,
        })
    }

    pub fn mount_bitstream(&mut self, slot: &str, name: &str, lut_count: u32) -> Result<()> {
        if let Some(m) = self.mounts.get_mut(slot) {
            m.bitstream_name = Some(name.to_string());
            m.lut_count = lut_count;
            m.active = true;
            Ok(())
        } else {
            bail!("Unknown slot: {}", slot)
        }
    }

    pub fn unmount_bitstream(&mut self, slot: &str) -> Result<()> {
        if let Some(m) = self.mounts.get_mut(slot) {
            m.bitstream_name = None;
            m.active = false;
            Ok(())
        } else {
            bail!("Unknown slot: {}", slot)
        }
    }
}

impl Default for TetherEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context() {
        let engine = TetherEngine::new();
        let ctx = engine.get_context();
        assert_eq!(ctx.total_luts, 256_000);
        assert_eq!(ctx.storage.len(), 2);
    }

    #[test]
    fn test_mount_unmount() {
        let mut engine = TetherEngine::new();
        engine.mount_bitstream("RP_1", "GPU", 80000).unwrap();
        let ctx = engine.get_context();
        assert_eq!(ctx.mounted.len(), 1);
        engine.unmount_bitstream("RP_1").unwrap();
        let ctx = engine.get_context();
        assert_eq!(ctx.mounted.len(), 0);
    }

    #[test]
    fn test_tether_query() {
        let engine = TetherEngine::new();
        let resp = engine.query("kernel_version").unwrap();
        match resp.result {
            TetherResult::Kernel(k) => assert_eq!(k.version, "0.1.0"),
            _ => panic!("Expected Kernel result"),
        }
    }
}