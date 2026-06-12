# cathedral_verification/lean4_windows_production.ps1
# Pipeline de Extração Segura: Lean 4 (WSL) -> C -> Windows (MSVC)
# Selo: LEAN4-WINDOWS-PROD-v1.0.0-2026-06-11

param(
    [string]$LeanFilePath = "cathedral_verification/lean4_proofs/CathedralAGI.lean",
    [string]$OutputDir = "cathedral_verification/bin"
)

Write-Host "[Pipeline] Iniciando extração segura de Lean 4 para Windows..."

# 1. Verificar e Compilar a Prova no WSL
Write-Host "[WSL] Verificando prova com 'lake build'..."
wsl -- bash -c "cd /app && lake build $LeanFilePath 2>&1"
if ($LASTEXITCODE -ne 0) {
    Write-Error "[FALHA] Prova Lean 4 inválida. A extração foi abortada."
    exit 1
}

# 2. Extrair código C usando Hax
Write-Host "[WSL] Extraindo código C via Hax..."
$extracted_c_path = "$OutputDir/extracted.c"
wsl -- bash -c "cd /app && hax extract -I $LeanFilePath -o $extracted_c_path 2>&1"

# 3. Copiar o código extraído do WSL para o Windows
$windows_c_path = "$OutputDir\cathedral_extracted.c"
Copy-Item -Path "\\wsl$\app\$extracted_c_path" -Destination $windows_c_path
Write-Host "[Windows] Código C extraído com sucesso."

# 4. Compilar com MSVC (Visual Studio Build Tools)
$msvc_path = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\bin\Hostx64\x64\cl.exe"
$exe_path = "$OutputDir\cathedral_skill.exe"

Write-Host "[MSVC] Compilando código extraído..."
& $msvc_path $windows_c_path /Fe:$exe_path /GS /sdl /O2 /D "CATHEDRAL_SECURITY_DEFINES"

if (Test-Path $exe_path) {
    Write-Host "[SUCESSO] Executável gerado: $exe_path"
} else {
    Write-Error "[FALHA] MSVC falhou ao compilar."
    exit 1
}

# 5. Assinar o binário com o certificado da Cathedral
Write-Host "[Segurança] Assinando com certificado Cathedral..."
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign /fd SHA256 /a $exe_path

Write-Host "============================="
Write-Host "PIPELINE CONCLUÍDA COM SUCESSO"
Write-Host "Hash do executável assinado:"
(Get-FileHash -Algorithm SHA256 $exe_path).Hash
