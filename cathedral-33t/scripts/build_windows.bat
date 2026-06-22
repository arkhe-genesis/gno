@echo off
set TARGET=x86_64-pc-windows-msvc
set FEATURES=tensor-backend-directml,ssm,symbolic

cargo build --release --target %TARGET% --features %FEATURES%

echo Build complete: target\%TARGET%\release\cathedral-arkhe-33t.exe
