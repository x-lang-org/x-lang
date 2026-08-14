#!/usr/bin/env python3
"""Run a backend whose toolchain lives in docker or a local dir; verify outputs."""
import argparse, tomllib, subprocess, os, tempfile, sys

DIR = "/home/xiongdi/workspace/x-lang/tests/benchmarks_game"
BENCHMARKS = ["binary-trees", "fannkuch-redux", "fasta", "k-nucleotide",
              "mandelbrot", "n-body", "pidigits", "regex-redux",
              "reverse-complement", "spectral-norm"]
X = "/home/xiongdi/workspace/x-lang/tools/target/release/x"
ZIG = "/home/xiongdi/workspace/x-lang/.toolchains/zig-linux-x86_64-0.13.0/zig"
XRT = "/home/xiongdi/workspace/x-lang/library/runtime/xrt.c"

def run(cmd, stdin=None, timeout=300, cwd=None):
    try:
        r = subprocess.run(cmd, input=stdin, capture_output=True, text=True, timeout=timeout, cwd=cwd)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("backend")
    ap.add_argument("--only", nargs="+")
    args = ap.parse_args()
    backend = args.backend
    names = args.only or BENCHMARKS
    all_ok = True
    for name in names:
        with open(f"{DIR}/{name}.toml", "rb") as f:
            data = tomllib.load(f)
        stdin = data.get("stdin")
        contains = data.get("expect", {}).get("runtime", {}).get("output_contains", [])
        with tempfile.TemporaryDirectory() as td:
            src = os.path.join(td, "bench.x")
            with open(src, "w") as f:
                f.write(data["source"])
            out_base = os.path.join(td, "out")
            # The java backend writes Main.java into the CLI's cwd; use --no-link
            # so the CLI does not auto-run (stdin benchmarks would hang).
            cli_cwd = td if backend == "java" else None
            use_no_link = backend != "zig"
            args = [X, "compile", src, "-o", out_base, "--target", backend]
            if use_no_link:
                args.append("--no-link")
            rc, err, _ = run(args, timeout=300, cwd=cli_cwd)
            if rc != 0:
                print(f"[FAIL] {name}: compile failed: {err[-400:]}")
                all_ok = False
                continue
            gen = os.path.join(td, "Main.java") if backend == "java" else out_base + ext_map[backend]
            run_path = None
            rrc = -1; rout = ""; rerr = ""
            if backend == "zig":
                # The CLI links the executable itself (zig build-exe); nothing to do.
                rrc = 0
                run_path = out_base
            elif backend == "java":
                # javac produces Main.class in the temp dir; run with java.
                rrc, _, rerr = run(["javac", gen], timeout=300)
                if rrc == 0:
                    run_path = None  # run below via java Main
                    jrrc, rout, rerr = run(["java", "-cp", td, "Main"], stdin=stdin)
                    rrc = jrrc
            elif backend == "ts":
                # The backend emits real TypeScript (interfaces, type annotations);
                # transpile with tsc (installed on the fly) then run the JS output.
                rrc, rout, rerr = run(["docker", "run", "--rm", "-v", td + ":/w", "-w", "/w",
                                       "node:20-bookworm", "sh", "-c",
                                       "npm init -y >/dev/null 2>&1 && "
                                       "npm i typescript@5.4 @types/node >/dev/null 2>&1 && "
                                       "npx tsc --target es2020 --module commonjs --strict false --types node "
                                       "--skipLibCheck out.ts >/dev/null 2>&1 && node out.js"], stdin=stdin, timeout=600)
            elif backend == "erlang":
                rrc, _, rerr = run(["docker", "run", "--rm", "-v", td + ":/w", "-w", "/w",
                                    "erlang:27", "sh", "-c", "erlc out.erl && erl -noshell -s main main -s init stop"], timeout=300)
                if rrc == 0:
                    rout = ""
            elif backend == "csharp":
                rrc, rout, rerr = run(["docker", "run", "--rm", "-v", td + ":/w", "-w", "/w",
                                       "mcr.microsoft.com/dotnet/sdk:8.0", "sh", "-c",
                                       "mkdir -p app && cp out.cs app/Program.cs && cd app && dotnet new console --force -o . >/dev/null && dotnet run --no-restore 2>/dev/null || dotnet run"],
                                      stdin=stdin, timeout=600)
            elif backend == "swift":
                rrc, rout, rerr = run(["docker", "run", "--rm", "-v", td + ":/w", "-w", "/w",
                                       "swift:5.10", "swiftc", "-O", "out.swift", "-o", "outswift",
                                       "-L/usr/lib", "-lxrt", "-lstdc++"], timeout=600)
                if rrc == 0:
                    rrc2, rout, rerr = run(["docker", "run", "--rm", "-v", td + ":/w", "-w", "/w",
                                            "swift:5.10", "./outswift"], stdin=stdin)
                    rrc = rrc2
            elif backend == "rust":
                xrt_o = os.path.join(td, "xrt.o")
                run(["gcc", "-c", XRT, "-o", xrt_o], timeout=120)
                rrc, _, rerr = run(["rustc", "-O", gen, "-o", os.path.join(td, "bin"),
                                    "-C", "link-args=" + xrt_o], timeout=300)
                run_path = os.path.join(td, "bin")
            if rrc != 0:
                print(f"[FAIL] {name}: toolchain compile failed: {rerr[-300:]}")
                all_ok = False
                continue
            if run_path:
                rrc, rout, rerr = run([run_path], stdin=stdin)
            if rrc != 0:
                print(f"[FAIL] {name}: run failed rc={rrc}: {rerr[-200:]}")
                all_ok = False
                continue
            missing = [c for c in contains if c not in rout]
            rust = f"{DIR}/rust/target/release/{name}"
            rrust = None
            if os.path.exists(rust):
                rrc2, rout2, _ = run([rust], stdin=stdin)
                if rrc2 == 0:
                    rrust = rout2
            tag = ""
            if rrust is not None:
                tag = "MATCHES-RUST" if rout == rrust else f"DIFFERS-RUST({len(rout)}/{len(rrust)})"
            if missing:
                print(f"[FAIL] {name}: missing {missing[:3]} | {tag}")
                all_ok = False
            else:
                print(f"[PASS] {name}: {len(rout)} chars {tag}")
    print("ALL PASS" if all_ok else "SOME FAILED")
    return 0 if all_ok else 1

ext_map = {"zig": ".zig", "rust": ".rs", "python": ".py", "ts": ".ts", "typescript": ".ts", "java": ".java",
           "csharp": ".cs", "llvm": ".ll", "swift": ".swift", "erlang": ".erl"}

if __name__ == "__main__":
    sys.exit(main())
