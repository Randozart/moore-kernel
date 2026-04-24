#!/usr/bin/env python3
"""
build.py - Unified build script for Moore Kernel

Usage:
    python tools/build.py --all
    python tools/build.py --kernel
    python tools/build.py --toolchain
    python tools/build.py --bitstream <name>
    python tools/build.py --deploy <device>
"""

import argparse
import os
import sys
import subprocess
import json
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
TARGET_DIR = PROJECT_ROOT / "target" / "release"

def run_cmd(cmd, cwd=None, check=True):
    """Run a command and return success status."""
    print(f"  Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    if check and result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
        return False
    return result.returncode == 0

def build_toolchain():
    """Build the Brief toolchain (counsel + bvc-compiler)."""
    print("\n=== Building Brief Toolchain ===")
    os.chdir(PROJECT_ROOT)

    if not run_cmd(["cargo", "build", "-p", "counsel", "--release"]):
        return False

    if not run_cmd(["cargo", "build", "-p", "bvc-compiler"]):
        return False

    counsel = TARGET_DIR / "counsel"
    bvc = TARGET_DIR / "bvc-compiler"

    if counsel.exists():
        print(f"  counsel built: {counsel}")
    if bvc.exists():
        print(f"  bvc-compiler built: {bvc}")

    return True

def build_msh():
    """Build Moore Shell (msh)."""
    print("\n=== Building Moore Shell ===")
    os.chdir(PROJECT_ROOT)

    if not run_cmd(["cargo", "build", "-p", "msh"]):
        return False

    msh = PROJECT_ROOT / "target" / "debug" / "msh"
    if msh.exists():
        print(f"  msh built: {msh}")

    return True

def build_drivers():
    """Build kernel drivers (PCAP, security)."""
    print("\n=== Building Kernel Drivers ===")
    os.chdir(PROJECT_ROOT)

    if not run_cmd(["cargo", "build", "-p", "pcap-driver"]):
        return False

    if not run_cmd(["cargo", "build", "-p", "security"]):
        return False

    return True

def build_kernel():
    """Build complete kernel."""
    print("\n=== Building Moore Kernel ===")
    return build_msh() and build_drivers()

def build_bitstream(name):
    """Build a specific bitstream."""
    print(f"\n=== Building Bitstream: {name} ===")

    bv_file = PROJECT_ROOT / "bitstreams" / name / "main.bv"
    ebv_file = PROJECT_ROOT / "bitstreams" / name / "hardware.toml"

    if not bv_file.exists():
        bv_file = PROJECT_ROOT / f"bitstreams/{name}.bv"
        if not bv_file.exists():
            print(f"Error: {name}.bv not found")
            return False

    hw_flag = ["--hw", str(PROJECT_ROOT / "ebv" / "kv260.ebv")]
    out_flag = ["--out", f"/tmp/{name}_out"]

    counsel = TARGET_DIR / "counsel"
    if not counsel.exists():
        print(f"Error: counsel not found at {counsel}")
        return False

    cmd = [str(counsel), "verilog", str(bv_file)] + hw_flag + out_flag

    if not run_cmd(cmd):
        return False

    print(f"  Generated: /tmp/{name}_out/{name}.sv")
    return True

def build_all():
    """Build everything."""
    print("\n=== Building All ===")
    return (build_toolchain() and build_kernel())

def deploy_to_device(device):
    """Deploy kernel to SD card device."""
    print(f"\n=== Deploying to {device} ===")

    if not Path(device).exists():
        print(f"Error: Device {device} not found")
        return False

    moore_bin = PROJECT_ROOT / "kernel" / "moore" / "moore.bin"

    print("  Mounting SD card...")
    print("  Copying moore.bin to FAT32 partition...")
    print("  (Deployment implementation pending)")
    return True

def main():
    parser = argparse.ArgumentParser(description="Moore Kernel Build System")
    parser.add_argument("--all", action="store_true", help="Build all")
    parser.add_argument("--toolchain", action="store_true", help="Build toolchain only")
    parser.add_argument("--kernel", action="store_true", help="Build kernel only")
    parser.add_argument("--msh", action="store_true", help="Build msh only")
    parser.add_argument("--drivers", action="store_true", help="Build drivers only")
    parser.add_argument("--bitstream", metavar="NAME", help="Build specific bitstream")
    parser.add_argument("--deploy", metavar="DEVICE", help="Deploy to device")
    parser.add_argument("--test", action="store_true", help="Run tests")

    args = parser.parse_args()

    success = True

    if args.test:
        print("\n=== Running Tests ===")
        success = run_cmd(["cargo", "test", "--lib"])

    if args.all or (not any([args.toolchain, args.kernel, args.msh, args.drivers, args.bitstream, args.deploy, args.test])):
        success = build_all()

    if args.toolchain:
        success = build_toolchain() and success

    if args.kernel:
        success = build_kernel() and success

    if args.msh:
        success = build_msh() and success

    if args.drivers:
        success = build_drivers() and success

    if args.bitstream:
        success = build_bitstream(args.bitstream) and success

    if args.deploy:
        success = deploy_to_device(args.deploy) and success

    if success:
        print("\n=== BUILD SUCCESSFUL ===")
        return 0
    else:
        print("\n=== BUILD FAILED ===")
        return 1

if __name__ == "__main__":
    sys.exit(main())