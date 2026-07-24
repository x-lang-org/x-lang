#!/usr/bin/env python3
"""X Language Benchmarks: Interpreter vs Zig-backend vs Native vs Rust"""

import argparse
import os
import subprocess
import tempfile
import time
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
CLI_PATH = PROJECT_ROOT / "tools" / "x-cli"
RUST_RELEASE_DIR = PROJECT_ROOT / "tests" / "benchmarks_game" / "rust" / "target" / "release"
TOML_DIR = PROJECT_ROOT / "tests" / "benchmarks_game"
ZIG_OUT_DIR = TOML_DIR / "zig_output"
NATIVE_OUT_DIR = TOML_DIR / "native_output"

BENCHMARKS = [
    "binary-trees", "fannkuch-redux", "fasta", "k-nucleotide",
    "mandelbrot", "n-body", "pidigits", "regex-redux",
    "reverse-complement", "spectral-norm",
]

ELF_MAGIC = b"\x7fELF"


def is_runnable_elf(path: Path) -> bool:
    """Reject empty/corrupt compile outputs that would raise Exec format error."""
    try:
        if not path.is_file() or path.stat().st_size < 4:
            return False
        with open(path, "rb") as f:
            return f.read(4) == ELF_MAGIC
    except OSError:
        return False


def run_cmd(cmd, cwd=None, stdin=None, timeout=120):
    """Run a command; never raise — bad binaries / timeouts become returncode != 0."""
    start = time.perf_counter()
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            input=stdin,
            timeout=timeout,
            start_new_session=True,
        )
        elapsed = (time.perf_counter() - start) * 1000.0
        return elapsed, result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired as e:
        elapsed = (time.perf_counter() - start) * 1000.0
        out = e.stdout.decode("utf-8", errors="replace") if isinstance(e.stdout, bytes) else (e.stdout or "")
        err = e.stderr.decode("utf-8", errors="replace") if isinstance(e.stderr, bytes) else (e.stderr or "")
        return elapsed, out, (err + f"\n[timeout after {timeout}s]").strip(), 124
    except OSError as e:
        elapsed = (time.perf_counter() - start) * 1000.0
        return elapsed, "", f"[exec failed] {e}", 127


def cargo_x(args, cwd=None, stdin=None, timeout=120):
    """Prefer a built x binary; fall back to `cargo run`."""
    for candidate in (
        CLI_PATH / "target" / "release" / "x",
        CLI_PATH / "target" / "debug" / "x",
        PROJECT_ROOT / "target" / "release" / "x",
        PROJECT_ROOT / "target" / "debug" / "x",
    ):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return run_cmd([str(candidate), *args], cwd=cwd, stdin=stdin, timeout=timeout)
    return run_cmd(
        ["cargo", "run", "--quiet", "--", *args],
        cwd=CLI_PATH,
        stdin=stdin,
        timeout=timeout,
    )


def compile_and_run(name, temp_path, out_dir, target, stdin, timeout, run_timeout):
    binary_path = out_dir / name
    # Remove stale corrupt outputs so we never re-exec last run's junk.
    if binary_path.exists():
        binary_path.unlink()

    print(f"  [{name}] Compiling with {target} backend...", end=" ", flush=True)
    compile_time, _, comp_stderr, comp_code = cargo_x(
        ["compile", temp_path, "-o", str(binary_path), "--target", target],
        timeout=timeout,
    )
    if comp_code != 0:
        # Cargo/rustc warnings come first; the real error is usually at the end.
        tail = comp_stderr[-800:] if len(comp_stderr) > 800 else comp_stderr
        print(f"FAILED:\n{tail}")
        if binary_path.exists() and not is_runnable_elf(binary_path):
            binary_path.unlink(missing_ok=True)
        return "CompileError", False, "", ""
    if not is_runnable_elf(binary_path):
        print("FAILED: output is not a valid ELF executable")
        binary_path.unlink(missing_ok=True)
        return "BadBinary", False, "", ""

    print(f"OK ({compile_time:.0f}ms)")
    run_time, stdout, stderr, code = run_cmd(
        [str(binary_path)], stdin=stdin, timeout=run_timeout
    )
    if code != 0:
        kind = "Timeout" if code == 124 else ("Segfault" if code in (-11, 139) else f"exit {code}")
        print(f"  [{name}] {target} run FAILED ({kind}): {stderr[:200]}")
        return "RunError", False, stdout, stderr
    return f"{run_time:.1f} ms", True, stdout, stderr


def parse_args():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--only", nargs="+", metavar="NAME", help="Run only these benchmarks")
    p.add_argument("--skip-zig", action="store_true", help="Skip Zig backend")
    p.add_argument("--skip-native", action="store_true", help="Skip Native backend")
    p.add_argument("--skip-interp", action="store_true", help="Skip X interpreter")
    p.add_argument("--skip-rust", action="store_true", help="Skip Rust reference binaries")
    p.add_argument("--compile-timeout", type=int, default=300, help="Compile timeout (seconds)")
    p.add_argument("--run-timeout", type=int, default=60, help="Binary/interpreter run timeout")
    p.add_argument("--no-cleanup", action="store_true", help="Keep compiled binaries")
    return p.parse_args()


def main():
    args = parse_args()
    ZIG_OUT_DIR.mkdir(parents=True, exist_ok=True)
    NATIVE_OUT_DIR.mkdir(parents=True, exist_ok=True)

    names = args.only if args.only else BENCHMARKS
    unknown = [n for n in names if n not in BENCHMARKS]
    if unknown:
        print(f"Unknown benchmark(s): {', '.join(unknown)}")
        print(f"Available: {', '.join(BENCHMARKS)}")
        return 2

    print("=" * 80)
    print("   X Language Benchmarks: Interpreter vs Zig Backend vs Native vs Rust")
    print("=" * 80)

    results = []

    for name in names:
        toml_path = TOML_DIR / f"{name}.toml"
        if not toml_path.exists():
            print(f"  Skipping {name} (TOML not found)")
            continue

        with open(toml_path, "rb") as f:
            data = tomllib.load(f)
        source = data.get("source", "")
        stdin = data.get("stdin", None)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".x", delete=False, encoding="utf-8") as f:
            f.write(source)
            temp_path = f.name

        zig_time_str, zig_ok, zig_stdout = "skipped", False, ""
        native_time_str, native_ok, native_stdout = "skipped", False, ""
        x_time_str, x_stdout, x_code = "skipped", "", 1

        try:
            if not args.skip_zig:
                zig_time_str, zig_ok, zig_stdout, _ = compile_and_run(
                    name, temp_path, ZIG_OUT_DIR, "zig", stdin,
                    args.compile_timeout, args.run_timeout,
                )

            if not args.skip_native:
                native_time_str, native_ok, native_stdout, _ = compile_and_run(
                    name, temp_path, NATIVE_OUT_DIR, "native", stdin,
                    args.compile_timeout, args.run_timeout,
                )

            if not args.skip_interp:
                x_time, x_stdout, x_stderr, x_code = cargo_x(
                    ["run", temp_path], stdin=stdin, timeout=args.run_timeout
                )
                if x_code != 0:
                    x_time_str = f"Error: {x_stderr.strip()[:60]}"
                else:
                    x_time_str = f"{x_time:.1f} ms"
        finally:
            try:
                os.unlink(temp_path)
            except OSError:
                pass

        rust_time_str = "skipped"
        if not args.skip_rust:
            rust_binary = RUST_RELEASE_DIR / name
            if not rust_binary.exists():
                rust_time_str = "N/A"
            else:
                rust_time, _, _, rust_code = run_cmd(
                    [str(rust_binary)], stdin=stdin, timeout=args.run_timeout
                )
                rust_time_str = "Error" if rust_code != 0 else f"{rust_time:.1f} ms"

        if zig_ok and x_code == 0 and zig_stdout.strip() != x_stdout.strip():
            print(f"  ⚠  Output MISMATCH between Zig and Interpreter!")
            print(f"     Zig: {zig_stdout.strip()[:100]}")
            print(f"     Int: {x_stdout.strip()[:100]}")
        if native_ok and x_code == 0 and native_stdout.strip() != x_stdout.strip():
            print(f"  ⚠  Output MISMATCH between Native and Interpreter!")
            print(f"     Nat: {native_stdout.strip()[:100]}")
            print(f"     Int: {x_stdout.strip()[:100]}")

        results.append({
            "name": name,
            "x_time": x_time_str,
            "zig_time": zig_time_str,
            "native_time": native_time_str,
            "rust_time": rust_time_str,
        })

    print("\n" + "=" * 80)
    print(f"{'Benchmark':<18} {'X Interp':<14} {'Zig':<14} {'Native':<14} {'Rust':<14}")
    print("-" * 80)
    for r in results:
        print(f"{r['name']:<18} {r['x_time']:<14} {r['zig_time']:<14} {r['native_time']:<14} {r['rust_time']:<14}")
    print("=" * 80)

    if not args.no_cleanup:
        for d in (ZIG_OUT_DIR, NATIVE_OUT_DIR):
            for f in d.iterdir():
                if f.is_file() and f.name in BENCHMARKS:
                    f.unlink(missing_ok=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
