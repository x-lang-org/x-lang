# LLVM compile to binary then run and diff (Benchmarks Game style)
# Requires: LLVM 21, clang/gcc. Run from repo root: .\examples\run_benchmarks_game_tests_compile.ps1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$ExamplesDir = Join-Path $Root "examples"
$ExpectedDir = Join-Path $ExamplesDir "expected"
$OutDir = Join-Path $ExamplesDir "out"

$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
$XExe = Join-Path $Root "$TargetDir\debug\x.exe"
if (-not (Test-Path $XExe)) {
    Write-Host "Building x with codegen..."
    Set-Location $Root
    & cargo build -p x-cli --features codegen --no-default-features 2>&1 | Out-Null
}
if (-not (Test-Path $XExe)) { Write-Error "Build x with codegen first and have LLVM 21 + clang/gcc" }

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }

$Tests = @("nbody", "fannkuch_redux", "spectral_norm", "mandelbrot", "fasta", "revcomp", "binary_trees", "knucleotide", "pidigits", "regex_redux")
$Failed = 0
foreach ($name in $Tests) {
    $prog = Join-Path $ExamplesDir "$name.x"
    $exp = Join-Path $ExpectedDir "$name.txt"
    $exe = Join-Path $OutDir "$name.exe"
    if (-not (Test-Path $prog)) { Write-Warning "Skip (missing): $prog"; continue }
    if (-not (Test-Path $exp)) { Write-Warning "Skip (no expected): $exp"; continue }

    & $XExe compile $prog -o (Join-Path $OutDir $name) 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Host "FAIL $name (compile failed)" -ForegroundColor Red; $Failed++; continue }
    if (-not (Test-Path $exe)) { Write-Host "FAIL $name (exe not produced)" -ForegroundColor Red; $Failed++; continue }

    $out = & $exe 2>&1
    $exit = $LASTEXITCODE
    $expected = Get-Content $exp -Raw
    $actual = $out -join "`n"
    if ($actual.Length -gt 0 -and $actual[-1] -ne "`n") { $actual += "`n" }

    if ($exit -ne 0) { Write-Host "FAIL $name (exit $exit)" -ForegroundColor Red; $Failed++ }
    elseif ($actual -ne $expected) { Write-Host "FAIL $name (output diff)" -ForegroundColor Red; Write-Host "expected: $expected"; Write-Host "actual:   $actual"; $Failed++ }
    else { Write-Host "ok  $name" -ForegroundColor Green }
}
if ($Failed -gt 0) { Write-Host "`n$Failed failed" -ForegroundColor Red; exit 1 }
Write-Host "`nAll passed (LLVM compile + run + diff)" -ForegroundColor Green
