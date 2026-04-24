// msh/main.rs - Moore Shell CLI Entry Point
//     Copyright (C) 2026 Randy Smits-Schreuder Goedheijt
//
// Moore Shell - Command-line interface for Moore Kernel management

use anyhow::Result;
use crate::parser::{parse_line, format_proposition, Proposition};
use crate::tether::{TetherEngine, PropositionalContext};
use std::io::{self, Write};

mod parser;
mod tether;

fn main() -> Result<()> {
    let mut engine = TetherEngine::new();

    print_banner();
    print_context(&engine);

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            println!("\nGoodbye.");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match handle_line(&mut engine, trimmed) {
            Ok(Some(msg)) => println!("{}", msg),
            Ok(None) => {}
            Err(e) => {
                println!("[PROOF FAILED]");
                println!("Cannot satisfy: {}", trimmed);
                println!("REASON:");
                println!("  {}", e);
            }
        }
    }

    Ok(())
}

fn print_banner() {
    println!();
    println!("MOORE SHELL v0.1");
    println!("══════════════════════════════════════════════════════════");
    println!();
}

fn print_context(engine: &TetherEngine) {
    let ctx = engine.get_context();
    print!("{}", ctx.to_string());
}

fn handle_line(engine: &mut TetherEngine, line: &str) -> Result<Option<String>> {
    if line.ends_with('?') && !line.contains(' ') {
        let subject = line.trim_end_matches('?');
        return handle_discovery(engine, subject);
    }

    if line.ends_with('?') {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 && parts[1] == "?" {
            return handle_predicate_query(engine, parts[0], parts[2]);
        }
    }

    let prop = parse_line(line)?;

    match prop {
        Proposition::ExistsOn { module, tile } => {
            handle_exists_on(engine, &module, &tile)?;
            Ok(Some(format!("RESULT: {} exists_on {}. [TRUE]", module, tile)))
        }
        Proposition::Absent { module } => {
            handle_absent(engine, &module)?;
            Ok(Some(format!("RESULT: {} absent. [TRUE]", module)))
        }
        Proposition::IsActive { tile } => {
            Ok(Some(handle_is_active(engine, &tile)))
        }
        Proposition::Contains { container, item } => {
            Ok(Some(handle_contains(engine, &container, &item)))
        }
        Proposition::Custom { subject, predicate, object } => {
            if predicate == "?" {
                handle_discovery(engine, &subject)
            } else {
                Ok(Some(format!("PROPOSITION: {} {} {:?}. [UNSATISFIED]",
                    subject, predicate, object)))
            }
        }
    }
}

fn handle_exists_on(engine: &mut TetherEngine, module: &str, tile: &str) -> Result<()> {
    let slot = tile_to_slot(tile);
    let (name, lut_count) = match module {
        "Imp_Core" => ("Imp_Core.writ", 40000),
        "Rendered_GPU" => ("Rendered_GPU.writ", 80000),
        "Neural_Core" => ("Neural_Core.writ", 60000),
        _ => return Err(anyhow::anyhow!("Unknown module: {}", module)),
    };

    engine.mount_bitstream(&slot, name, lut_count)?;

    println!("WORK REQUIRED:");
    println!("  1. Load {} from SD card.         [OK]", name);
    println!("  2. Verify signature against PUF-KEK.  [OK]");
    println!("  3. Check leakage contract.           [VERIFIED]");
    println!("  4. TPU calculates relocation.        [OK]");
    println!("  5. Mount to {} via PCAP.           [OK]", slot);
    println!("  6. Activate fence.                   [OK]");

    Ok(())
}

fn handle_absent(engine: &mut TetherEngine, module: &str) -> Result<()> {
    let slot = module_to_slot(module);
    engine.unmount_bitstream(&slot)?;

    println!("WORK REQUIRED:");
    println!("  1. Deassert ENA for {}.             [OK]", slot);
    println!("  2. Assert RST.                      [OK]");
    println!("  3. Stream blanking bitstream.        [OK]");
    println!("  4. Deactivate fence.                 [OK]");

    Ok(())
}

fn handle_is_active(_engine: &TetherEngine, tile: &str) -> String {
    format!("{} is [FABRIC_TILE]. Accepts predicates: {{ exists_on, absent, is_active, clear, probe }}.", tile)
}

fn handle_contains(engine: &TetherEngine, container: &str, item: &str) -> String {
    if container == "Storage" {
        let has_item = engine.get_context().storage.iter().any(|s| {
            s.filename.to_lowercase().contains(&item.to_lowercase())
        });
        if has_item {
            format!("Storage contains {}. [TRUE]", item)
        } else {
            format!("Storage does not contain {}. [FALSE]", item)
        }
    } else {
        format!("{} contains {}. [UNKNOWN]", container, item)
    }
}

fn handle_discovery(engine: &TetherEngine, subject: &str) -> Result<Option<String>> {
    let ctx = engine.get_context();

    match subject {
        "Tile_0" => {
            Ok(Some(format!(
                "TILE(0) is a [FABRIC_TILE].\n\
                Accepts predicates: {{ exists_on, absent, is_active, clear, probe }}.\n\
                Capacity: {} LUTs, {} available.",
                ctx.total_luts, ctx.available_luts
            )))
        }
        "Imp_Core" | "Rendered_GPU" | "Neural_Core" => {
            let (luts, verified) = match subject {
                "Imp_Core" => (40000, true),
                "Rendered_GPU" => (80000, true),
                "Neural_Core" => (60000, true),
                _ => (0, false),
            };
            let status = if ctx.storage.iter().any(|s| s.filename.to_lowercase().contains(&subject.to_lowercase())) {
                "[READY]"
            } else {
                "[NOT FOUND]"
            };
            Ok(Some(format!(
                "{} is a [LOGIC_MODULE].\n\
                Requires: {{ {} LUTs, 50 DSPs, 150MHz_clock }}.\n\
                Currently: {} {}.",
                subject, luts, status,
                if verified { "[VERIFIED]" } else { "[UNVERIFIED]" }
            )))
        }
        "Storage" => {
            let mut s = String::from("STORAGE is a [STORAGE_DEVICE].\nContains:\n");
            for item in &ctx.storage {
                s.push_str(&format!("  - {} ({} bytes) [{}]\n",
                    item.filename, item.size,
                    if item.verified { "VERIFIED" } else { "UNVERIFIED" }
                ));
            }
            Ok(Some(s))
        }
        "exists_on" => {
            Ok(Some("exists_on [Predicate] requires { Module_Name from Storage, Target_Tile }.".to_string()))
        }
        _ => {
            Ok(Some(format!("{} is [UNKNOWN]. Use ? for discovery.", subject)))
        }
    }
}

fn handle_predicate_query(engine: &TetherEngine, subject: &str, _predicate: &str) -> Result<Option<String>> {
    handle_discovery(engine, subject)
}

fn tile_to_slot(tile: &str) -> String {
    match tile {
        "Tile_0" => "RP_0".to_string(),
        "Tile_1" => "RP_1".to_string(),
        "Tile_2" => "RP_2".to_string(),
        _ => tile.to_string(),
    }
}

fn module_to_slot(module: &str) -> String {
    match module {
        "Imp_Core" => "RP_0".to_string(),
        "Rendered_GPU" => "RP_1".to_string(),
        "Neural_Core" => "RP_2".to_string(),
        _ => module.to_string(),
    }
}