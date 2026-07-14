# app/scripts/build_release.ps1 — 프런트엔드 + Tauri 릴리즈 빌드
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

if (-not (Test-Path (Join-Path $root "node_modules"))) {
  npm install
}

# --no-bundle: 바로가기는 raw exe만 있으면 되므로 NSIS/MSI 설치 도구체인 없이 빠르게 빌드
npm run tauri build -- --no-bundle

$exe = Join-Path $root "src-tauri\target\release\app.exe"
if (-not (Test-Path $exe)) {
  throw "빌드 실패: exe가 생성되지 않았습니다: $exe"
}
Write-Host "release exe: $exe"
