; Vmcraft (Rust) v0.4.0 — current-user installer (no admin, no UAC).
; Built with NSIS 3 (makensis installer/installer.nsi, CWD = repo root).
; Installs the egui dashboard + the CLI/MCP server side by side.

!include "MUI2.nsh"

Name "Vmcraft (Rust) 0.4.0"
OutFile "..\target\release\bundle\Vmcraft-Rust-0.4.0-setup.exe"
InstallDir "$LOCALAPPDATA\Programs\rust-vmcraft-mcp"
RequestExecutionLevel user

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Binaries" SecBinaries
  SetOutPath "$INSTDIR"
  File "..\target\release\vmcraft-app.exe"
  File "..\target\release\rust-vmcraft-mcp.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp" \
    "DisplayName" "Vmcraft (Rust) 0.4.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp" \
    "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp" \
    "DisplayIcon" "$INSTDIR\vmcraft-app.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp" \
    "DisplayVersion" "0.4.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp" \
    "Publisher" "sandraschi"
SectionEnd

Section "Start Menu shortcuts" SecShortcuts
  CreateDirectory "$SMPROGRAMS\Vmcraft (Rust)"
  CreateShortcut "$SMPROGRAMS\Vmcraft (Rust)\Vmcraft (Rust).lnk" "$INSTDIR\vmcraft-app.exe"
  CreateShortcut "$SMPROGRAMS\Vmcraft (Rust)\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "uninstall"
  Delete "$INSTDIR\vmcraft-app.exe"
  Delete "$INSTDIR\rust-vmcraft-mcp.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Vmcraft (Rust)\Vmcraft (Rust).lnk"
  Delete "$SMPROGRAMS\Vmcraft (Rust)\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Vmcraft (Rust)"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\rust-vmcraft-mcp"
SectionEnd
