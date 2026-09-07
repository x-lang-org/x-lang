#!/usr/bin/env python3
"""X Language Benchmarks: All 10 Backends

Runs the benchmarks_game suite across all available backends:
- interp: Tree-walking interpreter
- zig: Zig backend (requires zig compiler)
- native: Native backend (ELF, Linux x86_64 only)
- rust: Rust backend (requires rustc)
- python: Python backend (requires python3)
- ts: TypeScript backend (requires node)
- java: Java backend (requires javac + java)
- csharp: C# backend (requires dotnet)
- swift: Swift backend (requires swiftc)
- erlang: Erlang backend (requires erlc + erl)
- llvm: LLVM backend (requires llvm tools)
- wasm: WASM backend (requires wasmtime or node)
"""

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
TOML_DIR = Path(__file__).resolve().parent

BENCHMARKS = [
    "binary-trees", "fannkuch-redux", "fasta", "k-nucleotide",
    "mandelbrot", "n-body", "pidigits", "regex-redux",
    "reverse-complement", "spectral-norm",
]

# Backend configurations: (target_name, compile_cmd, run_cmd, file_ext)
BACKEND_CONFIGS = {
    "interp": {
        "needs_compile": False,
        "run_cmd": ["run"],
    },
    "zig": {
        "needs_compile": True,
        "compile_cmd": ["zig", "build-exe", "-OReleaseFast"],
        "run_cmd": [],
        "ext": ".zig",
    },
    "native": {
        "needs_compile": True,
        "compile_cmd": None,  # Handled by x-cli
        "run_cmd": [],
        "ext": "",
        "output_is_executable": True,  # Native backend produces an executable directly
    },
    "rust": {
        "needs_compile": True,
        "compile_cmd": ["rustc", "-O"],
        "run_cmd": [],
        "ext": ".rs",
    },
    "python": {
        "needs_compile": False,
        "run_cmd": ["python3"],
        "ext": ".py",
    },
    "ts": {
        "needs_compile": False,
        "run_cmd": ["node"],
        "ext": ".js",
    },
    "java": {
        "needs_compile": True,
        "compile_cmd": ["javac"],
        "run_cmd": ["java", "Main"],
        "ext": ".java",
        "output_name": "Main.java",  # Java always outputs Main.java
    },
    "csharp": {
        "needs_compile": True,
        "compile_cmd": None,  # Handled by x-cli
        "run_cmd": [],
        "ext": ".cs",
    },
    "swift": {
        "needs_compile": True,
        "compile_cmd": ["swiftc", "-O"],
        "run_cmd": [],
        "ext": ".swift",
    },
    "erlang": {
        "needs_compile": True,
        "compile_cmd": ["erlc"],
        "run_cmd": ["erl", "-noshell", "-s", "main", "main", "-s", "init", "stop"],
        "ext": ".erl",
        "module_name": "main",  # Erlang requires file name to match module name
    },
    "llvm": {
        "needs_compile": True,
        "compile_cmd": None,  # Handled by x-cli
        "run_cmd": [],
        "ext": ".ll",
    },
}


def run_cmd(cmd, cwd=None, stdin=None, timeout=120):
    """Run a command, return (elapsed_ms, stdout, stderr, returncode)."""
    start = time.perf_counter()
    try:
        result = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True,
            input=stdin, timeout=timeout,
        )
        elapsed = (time.perf_counter() - start) * 1000.0
        return elapsed, result.stdout, result.stderr, result.returncode
    except subprocess.TimeoutExpired:
        elapsed = (time.perf_counter() - start) * 1000.0
        return elapsed, "", "timeout", 124
    except OSError as e:
        elapsed = (time.perf_counter() - start) * 1000.0
        return elapsed, "", str(e), 127


def tool_available(tool):
    """Check if a command-line tool is available."""
    try:
        subprocess.run([tool, "--version"], capture_output=True, timeout=5)
        return True
    except (OSError, subprocess.TimeoutExpired):
        return False


def cargo_x(args, cwd=None, stdin=None, timeout=120):
    """Run the x CLI, preferring a built binary."""
    for candidate in (
        CLI_PATH / "target" / "release" / "x",
        CLI_PATH / "target" / "debug" / "x",
    ):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return run_cmd([str(candidate), *args], cwd=cwd, stdin=stdin, timeout=timeout)
    return run_cmd(
        ["cargo", "run", "--quiet", "--", *args],
        cwd=CLI_PATH, stdin=stdin, timeout=timeout,
    )


def compile_and_run_benchmark(name, backend, run_timeout=120):
    """Compile and run a single benchmark on a single backend."""
    toml_path = TOML_DIR / f"{name}.toml"
    if not toml_path.exists():
        return "skip", "", "TOML not found"

    with open(toml_path, "rb") as f:
        data = tomllib.load(f)
    source = data.get("source", "").strip()
    stdin_data = data.get("stdin", None)

    config = BACKEND_CONFIGS.get(backend)
    if not config:
        return "skip", "", f"Unknown backend: {backend}"

    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)
        source_path = tmpdir / "benchmark.x"
        source_path.write_text(source, encoding="utf-8")

        if backend == "interp":
            # Run directly with interpreter
            elapsed, stdout, stderr, rc = cargo_x(
                ["run", str(source_path)],
                stdin=stdin_data, timeout=run_timeout,
            )
            if rc != 0:
                return "error", stderr[:200], f"exit code {rc}"
            return "ok", stdout, f"{elapsed:.1f} ms"

        # Generate backend code
        # x-cli adds extension to out_path, so use a base name without extension
        out_base = tmpdir / "output"
        target = backend if backend != "ts" else "ts"

        # For native backend, we need to link (not --no-link) to get an executable
        use_no_link = backend != "native"

        try:
            args = ["compile", str(source_path), "-o", str(out_base), "--target", target]
            if use_no_link:
                args.append("--no-link")
            elapsed, stdout, stderr, rc = cargo_x(
                args,
                timeout=run_timeout,
            )
            if rc != 0:
                return "error", stderr[:200], "compile failed"
        except Exception as e:
            return "error", str(e)[:200], "compile exception"

        # Determine actual output file path (x-cli adds extension)
        if backend == "python":
            out_path = Path(f"{out_base}.py")
        elif backend == "ts":
            out_path = Path(f"{out_base}.ts")
        elif backend == "java":
            out_path = tmpdir / "Main.java"  # Java backend always outputs Main.java
        elif backend == "csharp":
            out_path = Path(f"{out_base}.cs")
        elif backend == "swift":
            out_path = Path(f"{out_base}.swift")
        elif backend == "erlang":
            out_path = Path(f"{out_base}.erl")
        elif backend == "zig":
            out_path = Path(f"{out_base}.zig")
        elif backend == "rust":
            out_path = Path(f"{out_base}.rs")
        elif backend == "llvm":
            out_path = Path(f"{out_base}.ll")
        elif backend == "native":
            # Native backend produces an executable directly (no extension)
            out_path = out_base
        else:
            out_path = Path(f"{out_base}{config['ext']}")

        # Compile the generated code if needed
        if backend == "llvm":
            # LLVM: compile .ll → executable via clang, linking with xrt.c
            xrt_c = PROJECT_ROOT / "library" / "runtime" / "xrt.c"
            run_cmd(["clang", "-O2", str(out_path), str(xrt_c),
                     "-o", tmpdir / name, "-lc", "-lm"],
                    timeout=run_timeout)
            run_path = tmpdir / name
        elif config.get("compile_cmd"):
            compile_cmd = config["compile_cmd"]
            if backend == "zig":
                # Zig needs the runtime (xrt.c) and the output path
                xrt_c = PROJECT_ROOT / "library" / "runtime" / "xrt.c"
                run_cmd(["zig", "build-exe", "-OReleaseFast", str(out_path),
                         str(xrt_c), "-lc", f"-femit-bin={tmpdir}/{name}"],
                        cwd=tmpdir, timeout=run_timeout)
                run_path = tmpdir / name
            elif backend == "rust":
                # Rust needs the runtime (xrt.o)
                xrt_c = PROJECT_ROOT / "library" / "runtime" / "xrt.c"
                xrt_o = tmpdir / "xrt.o"
                run_cmd(["gcc", "-c", str(xrt_c), "-o", str(xrt_o)],
                        timeout=run_timeout)
                run_cmd(["rustc", "-O", str(out_path), "-o", tmpdir / name,
                         "-C", f"link-args={xrt_o}"],
                        timeout=run_timeout)
                run_path = tmpdir / name
            elif backend == "java":
                run_cmd(["javac", str(out_path)], cwd=tmpdir, timeout=run_timeout)
                run_path = None  # Java runs by class name
            elif backend == "swift":
                # Swift needs the runtime (xrt.o)
                xrt_c = PROJECT_ROOT / "library" / "runtime" / "xrt.c"
                xrt_o = tmpdir / "xrt.o"
                run_cmd(["gcc", "-c", str(xrt_c), "-o", str(xrt_o)],
                        timeout=run_timeout)
                run_cmd(["swiftc", "-O", str(out_path), str(xrt_o), "-o", tmpdir / name],
                        timeout=run_timeout)
                run_path = tmpdir / name
            elif backend == "llvm":
                # LLVM: compile .ll → executable via clang, linking with xrt.c
                xrt_c = PROJECT_ROOT / "library" / "runtime" / "xrt.c"
                run_cmd(["clang", "-O2", str(out_path), str(xrt_c),
                         "-o", tmpdir / name, "-lc", "-lm"],
                        timeout=run_timeout)
                run_path = tmpdir / name
            elif backend == "erlang":
                # Erlang requires file name to match module name
                # The generated file is main.erl, so we need to compile it as main
                run_cmd(["erlc", str(out_path)], cwd=tmpdir, timeout=run_timeout)
                run_path = None  # Erlang runs by module
            else:
                run_path = out_path
        else:
            run_path = out_path

        # Run the compiled output
        run_cmd_list = config.get("run_cmd", [])
        if backend == "python":
            elapsed, stdout, stderr, rc = run_cmd(
                ["python3", str(run_path)], stdin=stdin_data, timeout=run_timeout,
            )
        elif backend == "ts":
            # TypeScript backend generates .ts file, run it directly with node
            # Node.js can run TypeScript files directly (with type stripping)
            elapsed, stdout, stderr, rc = run_cmd(
                ["node", str(run_path)], stdin=stdin_data, timeout=run_timeout,
            )
        elif backend == "java":
            elapsed, stdout, stderr, rc = run_cmd(
                ["java", "Main"], cwd=tmpdir, stdin=stdin_data, timeout=run_timeout,
            )
        elif backend == "erlang":
            elapsed, stdout, stderr, rc = run_cmd(
                ["erl", "-noshell", "-s", "main", "main", "-s", "init", "stop"],
                cwd=tmpdir, stdin=stdin_data, timeout=run_timeout,
            )
        elif run_path and run_path.exists():
            elapsed, stdout, stderr, rc = run_cmd(
                [str(run_path)], stdin=stdin_data, timeout=run_timeout,
            )
        elif backend == "interp":
            # Already handled above
            pass
        else:
            return "error", "", "no run command or output not found"

        if rc != 0:
            return "error", stderr[:200], f"exit code {rc}"
        return "ok", stdout, f"{elapsed:.1f} ms"


def main():
    parser = argparse.ArgumentParser(description="Run X benchmarks on all backends")
    parser.add_argument("--backends", nargs="+", default=list(BACKEND_CONFIGS.keys()),
                        help="Backends to test")
    parser.add_argument("--benchmarks", nargs="+", default=BENCHMARKS,
                        help="Benchmarks to run")
    parser.add_argument("--run-timeout", type=int, default=120,
                        help="Timeout per benchmark in seconds")
    args = parser.parse_args()

    print("=" * 100)
    print("   X Language Benchmarks: All Backends")
    print("=" * 100)

    # Check available tools
    available_backends = []
    for backend in args.backends:
        config = BACKEND_CONFIGS[backend]
        if backend == "interp":
            available_backends.append(backend)
        elif backend in ("zig",):
            available_backends.append(backend if tool_available("zig") else None)
        elif backend in ("native",):
            available_backends.append(backend if os.name == "posix" else None)
        elif backend in ("rust",):
            available_backends.append(backend if tool_available("rustc") else None)
        elif backend in ("python",):
            available_backends.append(backend if tool_available("python3") else None)
        elif backend in ("ts",):
            available_backends.append(backend if tool_available("node") else None)
        elif backend in ("java",):
            available_backends.append(backend if tool_available("javac") and tool_available("java") else None)
        elif backend in ("csharp",):
            available_backends.append(backend if tool_available("dotnet") else None)
        elif backend in ("swift",):
            available_backends.append(backend if tool_available("swiftc") else None)
        elif backend in ("erlang",):
            available_backends.append(backend if tool_available("erlc") else None)
        elif backend in ("llvm",):
            available_backends.append(backend if tool_available("llvm-config") else None)
        else:
            available_backends.append(None)

    available_backends = [b for b in available_backends if b is not None]
    print(f"\nAvailable backends: {', '.join(available_backends)}")
    print(f"Benchmarks: {', '.join(args.benchmarks)}\n")

    results = {}
    for backend in available_backends:
        print(f"\n--- {backend.upper()} ---")
        results[backend] = {}
        for name in args.benchmarks:
            status, output, detail = compile_and_run_benchmark(
                name, backend, args.run_timeout,
            )
            results[backend][name] = (status, output, detail)
            status_mark = "✓" if status == "ok" else ("✗" if status == "error" else "-")
            print(f"  [{status_mark}] {name:<20} {detail}")

    # Summary table
    print("\n" + "=" * 100)
    print("SUMMARY")
    print("=" * 100)
    header = f"{'Benchmark':<18}"
    for backend in available_backends:
        header += f" {backend:<12}"
    print(header)
    print("-" * len(header))

    for name in args.benchmarks:
        row = f"{name:<18}"
        for backend in available_backends:
            status, _, detail = results[backend].get(name, ("skip", "", ""))
            if status == "ok":
                row += f" {detail:<12}"
            elif status == "error":
                row += f" {'FAIL':<12}"
            else:
                row += f" {'-':<12}"
        print(row)

    print("=" * 100)

    # Count passes
    all_pass = True
    for backend in available_backends:
        passes = sum(1 for name in args.benchmarks if results[backend].get(name, ("skip", "", ""))[0] == "ok")
        total = len(args.benchmarks)
        print(f"{backend}: {passes}/{total} passed")
        if passes < total:
            all_pass = False

    if all_pass:
        print("\n✓ ALL BENCHMARKS PASSED!")
        return 0
    else:
        print("\n✗ Some benchmarks failed.")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
