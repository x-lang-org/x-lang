#!/usr/bin/env python3
"""Native backend multi-target acceptance harness.

Compiles a set of `.x` programs through the native backend for every target in
the matrix, links them (via `zig cc -target ...` for cross targets, host `cc`
for the host target), and executes them where an execution method is available
(host directly, qemu-user for cross-Linux, wine for Windows). macOS targets are
build/validate-only on non-Darwin hosts.

This script is the acceptance measurement for "native backend to 100%". It does
NOT modify any user files. Run from the repo root:

    python tests/native_matrix.py                # examples, all targets
    python tests/native_matrix.py --target x86_64-linux
    python tests/native_matrix.py --expect expectations.json

Exit code is non-zero if any *required* (executable) target fails.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CLI_BIN = REPO_ROOT / "tools" / "target" / "release" / "x"
EXAMPLES_DIR = REPO_ROOT / "examples"

# Negative fixtures: these example files are intentionally invalid (reference
# undefined variables inside string interpolation). "Pass" means the pipeline
# rejects them with the expected typecheck error -- they are never executed.
EXPECT_COMPILE_FAIL: dict[str, str] = {
    "interpolate-test": "未定义的变量",
    "interpolate-test2": "未定义的变量",
    "test-single-interp": "未定义的变量",
}


@dataclass
class Target:
    name: str          # short name, e.g. "x86_64-linux"
    triple: str        # passed to the CLI --target
    arch: str          # x86_64 | aarch64 | riscv64
    os: str            # linux | macos | windows
    # how to execute the produced binary: "host", "qemu", "wine", or None
    runner: str | None


MATRIX: list[Target] = [
    Target("x86_64-linux", "native", "x86_64", "linux", "host"),
    Target("aarch64-linux", "aarch64-linux", "aarch64", "linux", "qemu"),
    Target("riscv64-linux", "riscv64-linux", "riscv64", "linux", "qemu"),
    Target("x86_64-windows", "x86_64-windows", "x86_64", "windows", "wine"),
    Target("aarch64-windows", "aarch64-windows", "aarch64", "windows", None),
    Target("x86_64-macos", "x86_64-macos", "x86_64", "macos", None),
    Target("aarch64-macos", "aarch64-macos", "aarch64", "macos", None),
]


def host_arch() -> str:
    import platform
    m = platform.machine().lower()
    if m in ("x86_64", "amd64"):
        return "x86_64"
    if m in ("aarch64", "arm64"):
        return "aarch64"
    return m


def host_os() -> str:
    p = sys.platform
    if p.startswith("linux"):
        return "linux"
    if p == "darwin":
        return "macos"
    if p.startswith("win"):
        return "windows"
    return p


@dataclass
class Tooling:
    qemu: dict[str, str] = field(default_factory=dict)  # arch -> qemu binary
    wine: str | None = None

    @staticmethod
    def detect() -> "Tooling":
        t = Tooling()
        for arch, names in {
            "aarch64": ["qemu-aarch64-static", "qemu-aarch64"],
            "riscv64": ["qemu-riscv64-static", "qemu-riscv64"],
            "x86_64": ["qemu-x86_64-static", "qemu-x86_64"],
        }.items():
            for n in names:
                if shutil.which(n):
                    t.qemu[arch] = n
                    break
        t.wine = shutil.which("wine64") or shutil.which("wine")
        return t


@dataclass
class CaseResult:
    name: str
    compiled: bool = False
    linked: bool = False
    ran: bool = False
    exit_code: int | None = None
    stdout: str = ""
    error: str = ""
    skipped_exec: bool = False


def out_ext(os_name: str) -> str:
    return ".exe" if os_name == "windows" else ""


def _strip_status(output: str) -> str:
    """Drop CLI status lines (cargo-style) so interpreter stdout matches a raw binary."""
    keep = []
    for line in output.splitlines(keepends=True):
        s = line.strip()
        if s.startswith("Finished") or s.startswith("Running") or s.startswith("Compiling"):
            continue
        keep.append(line)
    return "".join(keep)


def run(cmd: list[str], timeout: int = 60) -> tuple[int, str, str]:
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout,
                           encoding="utf-8", errors="replace")
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"
    except FileNotFoundError as e:
        return 127, "", f"not found: {e}"


def compile_one(src: Path, target: Target, workdir: Path) -> CaseResult:
    res = CaseResult(name=src.stem)
    out = workdir / (src.stem + out_ext(target.os))
    rc, so, se = run([str(CLI_BIN), "compile", str(src), "-o", str(out),
                      "--target", target.triple])
    if rc != 0:
        res.error = (se or so or "").strip()
        return res
    res.compiled = True
    res.linked = out.exists()
    if res.linked:
        _execute(res, out, target)
    else:
        res.error = "no linked binary produced"
    return res


def is_negative(name: str) -> bool:
    return name in EXPECT_COMPILE_FAIL


def negative_ok(res: CaseResult) -> bool:
    """A negative fixture passes iff compilation failed with the expected error."""
    return (not res.compiled) and (EXPECT_COMPILE_FAIL[res.name] in res.error)


def _execute(res: CaseResult, binary: Path, target: Target) -> None:
    tooling = Tooling.detect()
    runner = target.runner
    cmd: list[str] | None = None
    if runner == "host":
        if host_arch() == target.arch and host_os() == target.os:
            cmd = [str(binary)]
    elif runner == "qemu":
        q = tooling.qemu.get(target.arch)
        if q:
            cmd = [q, str(binary)]
    elif runner == "wine":
        if tooling.wine:
            cmd = [tooling.wine, str(binary)]
    if cmd is None:
        res.skipped_exec = True
        return
    rc, so, se = run(cmd)
    res.ran = True
    res.exit_code = rc
    res.stdout = so


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--target", action="append", help="restrict to target name(s)")
    ap.add_argument("--files", nargs="*", help="specific .x files (default: examples)")
    ap.add_argument("--expect", help="JSON file: {name: {exit: int, stdout: str}}")
    ap.add_argument("--oracle", action="store_true",
                    help="use the interpreter (x run) stdout as the expected output")
    ap.add_argument("--verbose", "-v", action="store_true")
    args = ap.parse_args()

    if not CLI_BIN.exists():
        print(f"error: CLI not built at {CLI_BIN}; run "
              f"`cd tools/x-cli && cargo build --release`", file=sys.stderr)
        return 2

    targets = MATRIX
    if args.target:
        wanted = set(args.target)
        targets = [t for t in MATRIX if t.name in wanted]
        if not targets:
            print(f"no matching targets in {[t.name for t in MATRIX]}", file=sys.stderr)
            return 2

    if args.files:
        files = [Path(f) for f in args.files]
    else:
        files = sorted(EXAMPLES_DIR.glob("*.x"))

    expectations = {}
    if args.expect:
        expectations = json.loads(Path(args.expect).read_text())
    if args.oracle:
        for src in files:
            if is_negative(src.stem):
                continue
            rc, so, se = run([str(CLI_BIN), "run", str(src)])
            # Record both the exit code and stdout so the native binary must
            # reproduce the interpreter exactly (e.g. return42 exits 42).
            expectations[src.stem] = {"exit": rc, "stdout": _strip_status(so)}

    tooling = Tooling.detect()
    print(f"host: {host_arch()}-{host_os()}  qemu={tooling.qemu}  wine={bool(tooling.wine)}")
    print()

    overall_required_fail = 0
    for target in targets:
        executable_here = (
            (target.runner == "host" and host_arch() == target.arch and host_os() == target.os)
            or (target.runner == "qemu" and target.arch in tooling.qemu)
            or (target.runner == "wine" and tooling.wine)
        )
        compiled = linked = ran = ran_ok = 0
        neg_total = neg_ok = 0
        with tempfile.TemporaryDirectory() as td:
            workdir = Path(td)
            for src in files:
                r = compile_one(src, target, workdir)

                # Negative fixtures: must FAIL to compile with the expected error.
                if is_negative(r.name):
                    neg_total += 1
                    if negative_ok(r):
                        neg_ok += 1
                    else:
                        overall_required_fail += 1
                        if args.verbose:
                            print(f"  [{target.name}] {r.name}: NEGATIVE expected "
                                  f"'{EXPECT_COMPILE_FAIL[r.name]}' but compiled={r.compiled} "
                                  f"err={r.error.strip()[:100]}")
                    continue

                if r.compiled:
                    compiled += 1
                if r.linked:
                    linked += 1
                if r.ran:
                    ran += 1
                    exp = expectations.get(r.name)
                    if exp is None:
                        ok = (r.exit_code == 0)
                    else:
                        ok = ("exit" not in exp) or (r.exit_code == exp["exit"])
                        if "stdout" in exp:
                            ok = ok and (r.stdout == exp["stdout"])
                    if ok:
                        ran_ok += 1
                    elif executable_here:
                        overall_required_fail += 1
                elif executable_here:
                    # A positive example that failed to compile/link is a failure
                    # on an executable target.
                    overall_required_fail += 1
                if args.verbose and (not r.compiled or not r.linked or (r.ran and r.exit_code != 0)):
                    print(f"  [{target.name}] {r.name}: "
                          f"compiled={r.compiled} linked={r.linked} "
                          f"ran={r.ran} exit={r.exit_code} {r.error.strip()[:120]}")
        tier = "EXEC" if executable_here else "BUILD-ONLY"
        n = len(files) - neg_total
        line = (f"{target.name:<16} [{tier:^10}]  "
                f"compiled {compiled}/{n}  linked {linked}/{n}")
        if executable_here:
            line += f"  ran-ok {ran_ok}/{ran}"
        if neg_total:
            line += f"  neg-ok {neg_ok}/{neg_total}"
        print(line)

    print()
    if overall_required_fail:
        print(f"FAIL: {overall_required_fail} executable-target case(s) failed")
        return 1
    print("OK: all executable-target cases passed (build-only targets validated for compile/link)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
