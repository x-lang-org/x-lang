@echo off
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat" x64
cl /c /EHsc test_sys_ffi_en.c
link test_sys_ffi_en.obj /OUT:test_sys_ffi_en.exe
