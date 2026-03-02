# Benchmarks Game 测试方式：运行各示例并 diff 输出与 expected/*.txt
# 用法：在项目根目录执行 .\examples\run_benchmarks_game_tests.ps1
# 可选：$env:CARGO_TARGET_DIR = "target_examples_test" 避免 target\debug\x.exe 占用

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$ExamplesDir = Join-Path $Root "examples"
$ExpectedDir = Join-Path $ExamplesDir "expected"

# 优先使用已构建的 x（可设置 CARGO_TARGET_DIR）
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
$XExe = Join-Path $Root "$TargetDir\debug\x.exe"
if (-not (Test-Path $XExe)) {
    Write-Host "构建 x (no-default-features)..."
    Set-Location $Root
    & cargo build --no-default-features 2>&1 | Out-Null
    if (-not (Test-Path $XExe)) {
        Write-Error "构建后仍找不到 $XExe"
    }
}

$Tests = @(
    "nbody", "fannkuch_redux", "spectral_norm", "mandelbrot", "fasta",
    "revcomp", "binary_trees", "knucleotide", "pidigits", "regex_redux"
)
$Failed = 0
foreach ($name in $Tests) {
    $prog = Join-Path $ExamplesDir "$name.x"
    $exp = Join-Path $ExpectedDir "$name.txt"
    if (-not (Test-Path $prog)) { Write-Warning "跳过（不存在）: $prog"; continue }
    if (-not (Test-Path $exp)) { Write-Warning "跳过（无预期）: $exp"; continue }

    $out = & $XExe run -q $prog 2>&1
    $exit = $LASTEXITCODE
    $expected = Get-Content $exp -Raw
    $actual = $out -join "`n"
    if ($actual.Length -gt 0 -and $actual[-1] -ne "`n") { $actual += "`n" }

    if ($exit -ne 0) {
        Write-Host "FAIL $name (exit $exit)" -ForegroundColor Red
        $Failed++
    } elseif ($actual -ne $expected) {
        Write-Host "FAIL $name (output diff)" -ForegroundColor Red
        Write-Host "expected: $expected"
        Write-Host "actual:   $actual"
        $Failed++
    } else {
        Write-Host "ok  $name" -ForegroundColor Green
    }
}
if ($Failed -gt 0) {
    Write-Host "`n$Failed 个失败" -ForegroundColor Red
    exit 1
}
Write-Host "`n全部通过（Benchmarks Game 方式：run + diff）" -ForegroundColor Green
