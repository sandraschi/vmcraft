#Requires -Version 5.1
<#
.SYNOPSIS
  vmcraft workstation launcher: L3 egui app (release exe preferred, else cargo run).
#>
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$AppCrate = "vmcraft-app"
$AppBin = "vmcraft-app"
$releaseExe = Join-Path $RepoRoot "target\release\$AppBin.exe"
if (Test-Path $releaseExe) {
    & $releaseExe @args
    exit $LASTEXITCODE
}
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Set-Location $RepoRoot
& "$env:USERPROFILE\.cargo\bin\cargo.exe" run -p $AppCrate --release -- @args
exit $LASTEXITCODE
