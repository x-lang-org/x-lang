#!/usr/bin/env python3
"""Verify native benchmark outputs against TOML expectations and Rust reference."""
import tomllib, subprocess, os

DIR = "/home/xiongdi/workspace/x-lang/tests/benchmarks_game"
BENCHMARKS = ["binary-trees", "fannkuch-redux", "fasta", "k-nucleotide",
              "mandelbrot", "n-body", "pidigits", "regex-redux",
              "reverse-complement", "spectral-norm"]

def run(binary, stdin=None, timeout=60):
    try:
        r = subprocess.run([binary], input=stdin, capture_output=True, text=True, timeout=timeout)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"

all_ok = True
for name in BENCHMARKS:
    with open(f"{DIR}/{name}.toml", "rb") as f:
        data = tomllib.load(f)
    stdin = data.get("stdin")
    expect = data.get("expect", {}).get("runtime", {})
    contains = expect.get("output_contains", [])
    binary = f"{DIR}/native_output/{name}"
    if not os.path.exists(binary):
        print(f"{name}: MISSING BINARY"); all_ok = False; continue
    rc, out, err = run(binary, stdin)
    ok = rc == 0
    missing = []
    for c in contains:
        if c not in out:
            missing.append(c)
    if missing:
        ok = False
    status = "PASS" if ok else "FAIL"
    if not ok:
        all_ok = False
        print(f"[{status}] {name} (rc={rc})")
        if missing:
            print("   missing expected:", missing[:3])
            print("   output head:", repr(out[:200]))
    else:
        print(f"[{status}] {name}")
        # also compare with rust reference
        rust = f"{DIR}/rust/target/release/{name}"
        if os.path.exists(rust):
            rrc, rout, _ = run(rust, stdin)
            if rrc == 0 and rout == out:
                print(f"         matches Rust reference exactly ({len(out)} chars)")
            else:
                # find first diff
                for i, (a, b) in enumerate(zip(out, rout)):
                    if a != b:
                        print(f"         DIFFERS from Rust at char {i}: native={out[max(0,i-30):i+30]!r} rust={rout[max(0,i-30):i+30]!r}")
                        break
                else:
                    print(f"         DIFFERS from Rust (len {len(out)} vs {len(rout)})")
print()
print("ALL PASS" if all_ok else "SOME FAILED")
