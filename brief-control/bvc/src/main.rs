use anyhow::{Context, Result};
use std::path::PathBuf;
use bvc_compiler::compile_bvc;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1).peekable();

    match args.next().as_deref() {
        Some("build") => {
            let bvc_path = args.next()
                .context("Usage: bvc-compiler build <file.bvc> [--ebv <hardware.ebv>] [--out <output.writ>]")?;
            let ebv_path = parse_flag(&mut args, "ebv", PathBuf::from("kv260.ebv"))?;
            let out_path = parse_flag(&mut args, "out", PathBuf::from("output.writ"))?;

            let bvc = PathBuf::from(bvc_path);
            let ebv = ebv_path.unwrap_or_else(|| {
                let dir = bvc.parent().unwrap_or(&bvc);
                dir.join("kv260.ebv")
            });
            let out = out_path.unwrap_or_else(|| PathBuf::from("output.writ"));

            compile_bvc(&bvc, &ebv, &out)
                .with_context(|| format!("Failed to compile {}", bvc.display()))?;

            println!("Writ of Execution written to {}", out.display());
            Ok(())
        }
        Some("help") | None => {
            println!("bvc-compiler — Brief Control compiler");
            println!();
            println!("Usage:");
            println!("  bvc-compiler build <file.bvc> [--ebv <hardware.ebv>] [--out <output.writ>]");
            println!("  bvc-compiler help");
            println!();
            println!("Examples:");
            println!("  bvc-compiler build example.bvc --ebv ../ebv/kv260.ebv --out output.writ");
            Ok(())
        }
        Some(other) => {
            anyhow::bail!("Unknown command: {}. Use 'bvc-compiler help' for usage.", other)
        }
    }
}

fn parse_flag<I: Iterator<Item=String>>(args: &mut std::iter::Peekable<I>, name: &str, default: PathBuf) -> Result<Option<PathBuf>> {
    if args.peek().map(|s| s.as_str()) == Some(&format!("--{}", name)) {
        args.next();
        Ok(Some(args.next().map(PathBuf::from).unwrap_or(default)))
    } else {
        Ok(None)
    }
}