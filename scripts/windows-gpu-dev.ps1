#Requires -Version 5.1
<#
.SYNOPSIS
  Prepare Windows GPU env for TBox embedded LLM (CUDA default) and run a command.

.DESCRIPTION
  llama-cpp CUDA/Vulkan builds on this repo need:
  - VS Build Tools x64 (vcvars64)
  - Ninja (CMAKE_GENERATOR=Ninja is also set in src-tauri/.cargo/config.toml)
  - Short CARGO_TARGET_DIR (avoids Windows ~250-char object path limits)
  - CUDA toolkit on PATH (Windows default feature via tauri.windows.conf.json)

  Examples:
    .\scripts\windows-gpu-dev.ps1
    .\scripts\windows-gpu-dev.ps1 -Command 'pnpm tauri build'
    .\scripts\windows-gpu-dev.ps1 -Backend vulkan -Command 'cargo check --features vulkan'
#>
param(
  [ValidateSet('cuda', 'vulkan')]
  [string]$Backend = 'cuda',
  [string]$TargetDir = 'D:\tbx',
  [string]$Command = 'pnpm tauri dev'
)

$ErrorActionPreference = 'Stop'

function Find-Vcvars64 {
  $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
  if (Test-Path $vswhere) {
    $vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vs) {
      $p = Join-Path $vs 'VC\Auxiliary\Build\vcvars64.bat'
      if (Test-Path $p) { return $p }
    }
  }
  $fallback = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
  if (Test-Path $fallback) { return $fallback }
  throw 'vcvars64.bat not found. Install VS 2022 Build Tools with C++ workload.'
}

function Ensure-Ninja {
  $ninja = Get-Command ninja -ErrorAction SilentlyContinue
  if ($ninja) { return }
  $scoopNinja = Join-Path $env:USERPROFILE 'scoop\apps\ninja\current\ninja.exe'
  if (Test-Path $scoopNinja) {
    $env:Path = "$(Split-Path $scoopNinja);$env:Path"
    return
  }
  throw 'ninja not on PATH. Install: scoop install ninja'
}

$vcvars = Find-Vcvars64
Ensure-Ninja

if (-not (Test-Path $TargetDir)) {
  New-Item -ItemType Directory -Path $TargetDir | Out-Null
}
$env:CARGO_TARGET_DIR = $TargetDir
$env:CMAKE_GENERATOR = 'Ninja'

if ($Backend -eq 'cuda') {
  if ($env:CUDA_PATH) { $env:CUDA_PATH = $env:CUDA_PATH.Trim() }
  if (-not $env:CUDA_PATH -or -not (Test-Path (Join-Path $env:CUDA_PATH 'bin\nvcc.exe'))) {
    $guess = 'C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.5'
    if (Test-Path (Join-Path $guess 'bin\nvcc.exe')) { $env:CUDA_PATH = $guess }
  }
  if (-not $env:CUDA_PATH -or -not (Test-Path (Join-Path $env:CUDA_PATH 'bin\nvcc.exe'))) {
    throw 'CUDA toolkit (nvcc) not found. Install CUDA Toolkit or use -Backend vulkan.'
  }
  $env:Path = "$(Join-Path $env:CUDA_PATH 'bin');$env:Path"
  $env:CMAKE_CUDA_COMPILER = Join-Path $env:CUDA_PATH 'bin\nvcc.exe'
  Write-Host "GPU backend: cuda ($($env:CUDA_PATH))"
}
else {
  if (-not $env:VULKAN_SDK) {
    $scoopVk = Join-Path $env:USERPROFILE 'scoop\apps\vulkan\current'
    if (Test-Path $scoopVk) { $env:VULKAN_SDK = $scoopVk }
  }
  if (-not $env:VULKAN_SDK) {
    throw 'VULKAN_SDK not set. Install Vulkan SDK (e.g. scoop install vulkan) or set VULKAN_SDK.'
  }
  $env:Path = "$(Join-Path $env:VULKAN_SDK 'Bin');$env:Path"
  Write-Host "GPU backend: vulkan ($($env:VULKAN_SDK))"
}

Write-Host "CARGO_TARGET_DIR=$($env:CARGO_TARGET_DIR)"
Write-Host "Running: $Command"

$envLines = @()
if ($env:CUDA_PATH) { $envLines += "set `"CUDA_PATH=$($env:CUDA_PATH)`"" }
if ($env:VULKAN_SDK) { $envLines += "set `"VULKAN_SDK=$($env:VULKAN_SDK)`"" }
if ($env:CMAKE_CUDA_COMPILER) { $envLines += "set `"CMAKE_CUDA_COMPILER=$($env:CMAKE_CUDA_COMPILER)`"" }

$bat = @"
@echo off
call "$vcvars" || exit /b 1
$($envLines -join "`r`n")
set "CARGO_TARGET_DIR=$($env:CARGO_TARGET_DIR)"
set "CMAKE_GENERATOR=Ninja"
set "PATH=$($env:Path)"
cd /d "$($PSScriptRoot)\.."
$Command
exit /b %ERRORLEVEL%
"@

$batPath = Join-Path $env:TEMP 'tbox-windows-gpu-dev.bat'
Set-Content -Path $batPath -Value $bat -Encoding ASCII
cmd /c "`"$batPath`""
exit $LASTEXITCODE
