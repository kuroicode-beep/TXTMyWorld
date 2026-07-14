# app/scripts/install_desktop_shortcut.ps1 — 바탕화면 바로가기
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$exe = Join-Path $root "src-tauri\target\release\app.exe"
if (-not (Test-Path $exe)) {
  throw "exe가 없습니다. 먼저 scripts\build_release.ps1을 실행하세요: $exe"
}
$desktop = [Environment]::GetFolderPath("Desktop")
$lnkPath = Join-Path $desktop "TXTMyWorld.lnk"
$wsh = New-Object -ComObject WScript.Shell
$lnk = $wsh.CreateShortcut($lnkPath)
$lnk.TargetPath = $exe
$lnk.WorkingDirectory = Split-Path $exe -Parent
$lnk.Description = "TXTMyWorld — TXT 패밀리 연결·생성 레이어"
$lnk.Save()
Write-Host "shortcut=$lnkPath"
