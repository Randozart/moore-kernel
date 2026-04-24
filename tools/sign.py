#!/usr/bin/env python3
"""
sign.py - Bitstream signing and encryption for Moore Kernel
    Copyright (C) 2026 Randy Smits-Schreuder Goedheijt

Usage:
    python tools/sign.py --input <writ_file> --output <signed_file>
    python tools/sign.py --verify <file> --signature <sig_file>
    python tools/sign.py --encrypt <file> --key <key_file>
"""

import argparse
import hashlib
import hmac
import os
import sys
import json
import base64
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent

def compute_hash(data: bytes) -> str:
    """Compute BLAKE3 hash of data."""
    return hashlib.blake3(data).hexdigest()

def sign_data(data: bytes, key: bytes) -> bytes:
    """Sign data with HMAC-SHA256."""
    signature = hmac.new(key, data, hashlib.sha256).digest()
    return signature

def verify_signature(data: bytes, signature: bytes, key: bytes) -> bool:
    """Verify HMAC signature."""
    expected = hmac.new(key, data, hashlib.sha256).digest()
    return hmac.compare_digest(signature, expected)

def encrypt_data(data: bytes, kek: bytes) -> bytes:
    """Encrypt data with AES-256-GCM (simplified)."""
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM

    aesgcm = AESGCM(kek)
    nonce = os.urandom(12)
    ciphertext = aesgcm.encrypt(nonce, data, None)
    return nonce + ciphertext

def decrypt_data(data: bytes, kek: bytes) -> bytes:
    """Decrypt data with AES-256-GCM (simplified)."""
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM

    nonce = data[:12]
    ciphertext = data[12:]
    aesgcm = AESGCM(kek)
    return aesgcm.decrypt(nonce, ciphertext, None)

def read_writ_metadata(writ_path: Path) -> dict:
    """Read metadata from .writ file."""
    with open(writ_path, "rb") as f:
        magic = f.read(4)
        if magic != b"WRIT":
            raise ValueError("Invalid .writ file: bad magic")

        version = int.from_bytes(f.read(2), "little")
        metadata_len = int.from_bytes(f.read(4), "little")
        metadata_json = f.read(metadata_len).decode("utf-8")

        return json.loads(metadata_json)

def create_signed_package(writ_path: Path, output_path: Path, signing_key: bytes):
    """Create a signed .writ package."""
    with open(writ_path, "rb") as f:
        writ_data = f.read()

    metadata = read_writ_metadata(writ_path)

    signature = sign_data(writ_data, signing_key)
    blake3_hash = compute_hash(writ_data)

    package = {
        "metadata": metadata,
        "signature": base64.b64encode(signature).decode("ascii"),
        "hash": blake3_hash,
        "format_version": 1,
    }

    package_path = output_path.with_suffix(".writ.sig")
    with open(package_path, "w") as f:
        json.dump(package, f, indent=2)

    print(f"  Signed package created: {package_path}")
    print(f"  BLAKE3 hash: {blake3_hash[:16]}...")
    print(f"  Signature: {base64.b64encode(signature).decode('ascii')[:32]}...")

    return package_path

def verify_package(writ_path: Path, sig_path: Path, key: bytes) -> bool:
    """Verify a signed .writ package."""
    with open(sig_path, "r") as f:
        package = json.load(f)

    with open(writ_path, "rb") as f:
        writ_data = f.read()

    signature = base64.b64decode(package["signature"])

    if not verify_signature(writ_data, signature, key):
        print("  Signature verification FAILED")
        return False

    expected_hash = compute_hash(writ_data)
    if expected_hash != package["hash"]:
        print("  Hash mismatch FAILED")
        return False

    print("  Signature verification: OK")
    print("  Hash verification: OK")
    print(f"  Package name: {package['metadata']['name']}")
    print(f"  Version: {package['metadata']['version']}")

    return True

def encrypt_package(writ_path: Path, output_path: Path, kek: bytes):
    """Encrypt a .writ file."""
    with open(writ_path, "rb") as f:
        writ_data = f.read()

    encrypted = encrypt_data(writ_data, kek)

    enc_path = output_path.with_suffix(".writ.enc")
    with open(enc_path, "wb") as f:
        f.write(encrypted)

    print(f"  Encrypted package created: {enc_path}")

    return enc_path

def decrypt_package(enc_path: Path, output_path: Path, kek: bytes):
    """Decrypt an encrypted .writ file."""
    with open(enc_path, "rb") as f:
        encrypted_data = f.read()

    decrypted = decrypt_data(encrypted_data, kek)

    with open(output_path, "wb") as f:
        f.write(decrypted)

    print(f"  Decrypted package written: {output_path}")

def main():
    parser = argparse.ArgumentParser(description="Moore Kernel Bitstream Signing")
    parser.add_argument("--input", type=Path, help="Input .writ file")
    parser.add_argument("--output", type=Path, help="Output path")
    parser.add_argument("--key", type=Path, help="Signing/encryption key (hex or file)")
    parser.add_argument("--sign", action="store_true", help="Sign the package")
    parser.add_argument("--verify", type=Path, metavar="WRIT", help="Verify against .writ file")
    parser.add_argument("--signature", type=Path, metavar="SIG", help="Signature file for verification")
    parser.add_argument("--encrypt", action="store_true", help="Encrypt the package")
    parser.add_argument("--decrypt", action="store_true", help="Decrypt the package")

    args = parser.parse_args()

    if args.key:
        with open(args.key, "rb") as f:
            key_data = f.read().strip()
        if len(key_data) == 64 and all(c in '0123456789abcdefABCDEF' for c in key_data.decode('ascii', errors='ignore')):
            signing_key = bytes.fromhex(key_data.decode('ascii'))
        else:
            signing_key = key_data[:32] if len(key_data) >= 32 else key_data.ljust(32, b'\x00')
    else:
        print("Using default development key (NOT FOR PRODUCTION)")
        signing_key = b"development_key_do_not_use_in_production"

    if args.sign:
        if not args.input or not args.output:
            print("Error: --input and --output required for --sign")
            return 1

        create_signed_package(args.input, args.output, signing_key)
        print("  Signing complete")

    if args.verify:
        if not args.signature:
            print("Error: --signature required for --verify")
            return 1

        if verify_package(args.verify, args.signature, signing_key):
            print("Verification SUCCESS")
            return 0
        else:
            print("Verification FAILED")
            return 1

    if args.encrypt:
        if not args.input or not args.output:
            print("Error: --input and --output required for --encrypt")
            return 1

        encrypt_package(args.input, args.output, signing_key)
        print("  Encryption complete")

    if args.decrypt:
        if not args.input or not args.output:
            print("Error: --input and --output required for --decrypt")
            return 1

        decrypt_package(args.input, args.output, signing_key)
        print("  Decryption complete")

    if not any([args.sign, args.verify, args.encrypt, args.decrypt]):
        print("No action specified. Use --help for usage.")
        return 1

    return 0

if __name__ == "__main__":
    sys.exit(main())