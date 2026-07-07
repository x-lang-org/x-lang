#!/usr/bin/env python3
"""X Language Benchmarks: Interpreter vs Zig-backend vs Rust"""

import os, sys, time, subprocess, tempfile
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
CLI_PATH = PROJECT_ROOT / "tools" / "x-cli"
RUST_RELEASE_DIR = PROJECT_ROOT / "tests" / "benchmarks_game" / "rust" / "target" / "release"
TOML_DIR = PROJECT_ROOT / "tests" / "benchmarks_game"
OUT_DIR = TOML_DIR / "zig_output"
OUT_DIR.mkdir(parents=True, exist_ok=True)

BENCHMARKS = [
    "binary-trees", "fannkuch-redux", "fasta", "k-nucleotide",
    "mandelbrot", "n-body", "pidigits", "regex-redux",
    "reverse-complement", "spectral-norm"
]

def run_cmd(cmd, cwd=None, stdin=None, timeout=120):
    start = time.perf_counter()
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True,
                            input=stdin, timeout=timeout)
    end = time.perf_counter()
    return (end - start) * 1000.0, result.stdout, result.stderr, result.returncode

def main():
    print("=" * 70)
    print("   X Language Benchmarks: Interpreter vs Zig Backend vs Rust")
    print("=" * 70)

    results = []

    for name in BENCHMARKS:
        toml_path = TOML_DIR / f"{name}.toml"
        if not toml_path.exists():
            print(f"  Skipping {name} (TOML not found)")
            continue

        with open(toml_path, "rb") as f:
            data = tomllib.load(f)
        source = data.get("source", "")
        stdin = data.get("stdin", None)

        # --- Step 1: Compile with Zig backend ---
        binary_path = OUT_DIR / name
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            print(f"\n  [{name}] Compiling with Zig backend...", end=" ", flush=True)
            compile_time, comp_stdout, comp_stderr, comp_code = run_cmd(
                ["cargo", "run", "--", "compile", temp_path, "-o", str(binary_path), "--target", "zig"],
                cwd=CLI_PATH
            )
            if comp_code != 0:
                print(f"FAILED:\n{comp_stderr[:500]}")
                zig_time_str = "CompileError"
                zig_ok = False
            else:
                print(f"OK ({compile_time:.0f}ms)")
                zig_ok = True

            # --- Step 2: Run Zig-compiled binary ---
            if zig_ok and binary_path.exists():
                zig_time, zig_stdout, zig_stderr, zig_code = run_cmd(
                    [str(binary_path)], stdin=stdin
                )
                if zig_code != 0:
                    print(f"  [{name}] Zig binary run FAILED (exit {zig_code}): {zig_stderr[:200]}")
                    zig_time_str = "RunError"
                else:
                    zig_time_str = f"{zig_time:.1f} ms"
            else:
                zig_time_str = "N/A"

            # --- Step 3: Run X Interpreter ---
            x_time, x_stdout, x_stderr, x_code = run_cmd(
                ["cargo", "run", "--", "run", temp_path],
                cwd=CLI_PATH, stdin=stdin
            )
            if x_code != 0:
                x_time_str = f"Error: {x_stderr.strip()[:60]}"
            else:
                x_time_str = f"{x_time:.1f} ms"

        finally:
            os.unlink(temp_path)

        # --- Step 4: Run Rust binary ---
        rust_binary = RUST_RELEASE_DIR / name
        if not rust_binary.exists():
            rust_time_str = "N/A"
            zig_vs_rust = "N/A"
        else:
            rust_time, rust_stdout, rust_stderr, rust_code = run_cmd(
                [str(rust_binary)], stdin=stdin
            )
            if rust_code != 0:
                rust_time_str = "Error"
            else:
                rust_time_str = f"{rust_time:.1f} ms"

        # Compare outputs
        if zig_ok and zig_time_str not in ("N/A", "RunError") and x_code == 0:
            if zig_stdout.strip() != x_stdout.strip():
                print(f"  ⚠  Output MISMATCH between Zig and Interpreter!")
                print(f"     Zig: {zig_stdout.strip()[:100]}")
                print(f"     Int: {x_stdout.strip()[:100]}")

        results.append({
            "name": name,
            "x_time": x_time_str,
            "zig_time": zig_time_str,
            "rust_time": rust_time_str,
        })

    # Summary
    print("\n" + "=" * 70)
    print(f"{'Benchmark':<20} {'X Interp':<15} {'Zig Backend':<15} {'Rust':<15}")
    print("-" * 70)
    for r in results:
        print(f"{r['name']:<20} {r['x_time']:<15} {r['zig_time']:<15} {r['rust_time']:<15}")
    print("=" * 70)

    # Cleanup
    for f in OUT_DIR.iterdir():
        if f.is_file() and f.name in BENCHMARKS:
            f.unlink()

if __name__ == "__main__":
    main()
