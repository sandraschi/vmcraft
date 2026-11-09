$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
& "C:\Users\sandr\.cargo\bin\cargo.exe" run -p vmcraft-app --release
