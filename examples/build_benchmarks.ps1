# 使用完整流水线（LLVM codegen）编译 examples/ 下 10 个 Benchmarks Game 测试
# 需先安装 LLVM 21 并设置 $env:LLVM_SYS_211_PREFIX
# Windows：若使用官方完整归档（clang+llvm-*-x86_64-pc-windows-msvc），链接时会需要 libxml2。
#   安装方式之一：安装 vcpkg 后执行 vcpkg install libxml2:x64-windows，
#   再在构建前设置 $env:LIB 包含 vcpkg 的 installed\x64-windows\lib（或使用 vcpkg 的 toolchain）。

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..

if (-not $env:LLVM_SYS_211_PREFIX) {
    $env:LLVM_SYS_211_PREFIX = "C:\Program Files\LLVM"
}

Write-Host "Building x with codegen..."
cargo build --features codegen
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo build --features codegen failed. Ensure LLVM 21 is installed and LLVM_SYS_211_PREFIX points to it (Windows installers often lack llvm-config.exe; consider WSL or vcpkg LLVM)."
    exit 1
}

New-Item -ItemType Directory -Force -Path build | Out-Null
$list = @("nbody","fannkuch_redux","spectral_norm","mandelbrot","fasta","knucleotide","revcomp","binary_trees","pidigits","regex_redux")
foreach ($name in $list) {
    Write-Host "Compiling examples/$name.x -> build/$name.o"
    cargo run --features codegen -- compile "examples/$name.x" -o "build/$name.o" --no-link
}
Write-Host "Done. Object files in build/"
