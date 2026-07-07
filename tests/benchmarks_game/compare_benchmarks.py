#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys
import time
import subprocess
import tempfile
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
CLI_PATH = PROJECT_ROOT / "tools" / "x-cli"
RUST_RELEASE_DIR = PROJECT_ROOT / "tests" / "benchmarks_game" / "rust" / "target" / "release"
TOML_DIR = PROJECT_ROOT / "tests" / "benchmarks_game"

BENCHMARKS = [
    "binary-trees",
    "fannkuch-redux",
    "fasta",
    "k-nucleotide",
    "mandelbrot",
    "n-body",
    "pidigits",
    "regex-redux",
    "reverse-complement",
    "spectral-norm"
]

def run_cmd(cmd, cwd=None, stdin=None):
    start = time.perf_counter()
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        input=stdin,
        timeout=60
    )
    end = time.perf_counter()
    return (end - start) * 1000.0, result.stdout, result.stderr, result.returncode

def main():
    print("=" * 60)
    print("   X Language vs Rust Benchmarks Game Comparison")
    print("=" * 60)
    
    results = []
    
    for name in BENCHMARKS:
        toml_path = TOML_DIR / f"{name}.toml"
        if not toml_path.exists():
            print(f"Skipping {name} (TOML not found)")
            continue
            
        with open(toml_path, "rb") as f:
            data = tomllib.load(f)
            
        source = data.get("source", "")
        stdin = data.get("stdin", None)
        
        # 1. Run X Language (Interpreter)
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name
            
        try:
            x_cmd = ["cargo", "run", "--", "run", temp_path]
            x_time, x_stdout, x_stderr, x_code = run_cmd(x_cmd, cwd=CLI_PATH, stdin=stdin)
        finally:
            os.unlink(temp_path)
            
        if x_code != 0:
            print(f"X Language failed to run {name}:\n{x_stderr}")
            x_time_str = "Error"
        else:
            x_time_str = f"{x_time:.1f} ms"
            
        # 2. Run Rust (Release binary)
        rust_binary = RUST_RELEASE_DIR / name
        if not rust_binary.exists():
            print(f"Rust binary for {name} not found at {rust_binary}")
            rust_time_str = "N/A"
            ratio_str = "N/A"
        else:
            rust_time, rust_stdout, rust_stderr, rust_code = run_cmd([str(rust_binary)], stdin=stdin)
            if rust_code != 0:
                print(f"Rust failed to run {name}:\n{rust_stderr}")
                rust_time_str = "Error"
                ratio_str = "N/A"
            else:
                rust_time_str = f"{rust_time:.1f} ms"
                if x_code == 0:
                    ratio = x_time / rust_time
                    ratio_str = f"{ratio:.1f}x"
                else:
                    ratio_str = "N/A"
                    
        results.append({
            "name": name,
            "x_time": x_time_str,
            "rust_time": rust_time_str,
            "ratio": ratio_str
        })
        print(f"Finished {name:20}: X = {x_time_str:>10}, Rust = {rust_time_str:>10}, Ratio = {ratio_str:>6}")
        
    print("\n" + "=" * 60)
    print("                     SUMMARY TABLE")
    print("=" * 60)
    print(f"| {'Benchmark Name':<20} | {'X (Interpreter)':<15} | {'Rust (Release)':<15} | {'Ratio (X/Rust)':<12} |")
    print(f"|{'-'*22}|{'-'*17}|{'-'*17}|{'-'*14}|")
    for r in results:
        print(f"| {r['name']:<20} | {r['x_time']:>15} | {r['rust_time']:>15} | {r['ratio']:>12} |")
    print("=" * 60)

if __name__ == "__main__":
    main()
