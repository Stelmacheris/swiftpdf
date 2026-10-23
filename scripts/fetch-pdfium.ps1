<#
.SYNOPSIS
  Downloads the pinned PDFium build (bblanchon/pdfium-binaries, no V8, no XFA) into src-tauri/pdfium/.

.DESCRIPTION
  - The release and the SHA-256 of every archive are pinned in $Pinned below (the only place to change).
  - The archive is verified against that SHA-256 before it is unpacked; a mismatch stops the script
    with exit code 1 and deletes the download.
  - args.gn inside the archive must say pdf_enable_v8 = false and pdf_enable_xfa = false, and the
    target_cpu must match, or the script stops.
  - Idempotent: if src-tauri/pdfium/ already holds the same version, architecture and archive hash, and
    pdfium.dll still has the recorded hash, nothing is downloaded.
  - Unattended: no prompts; exit code 0 on success, 1 on any error. Works in Windows PowerShell 5.1 and
    PowerShell 7.
  - The download is staged inside the destination folder (gitignored), never in %TEMP%.

  Result (about 8 MB):
    src-tauri/pdfium/pdfium.dll        the library, loaded at run time
    src-tauri/pdfium/LICENSE           PDFium license (BSD-3-Clause / Apache-2.0), shipped with the app
    src-tauri/pdfium/licenses/         licenses of the libraries bundled into pdfium.dll
    src-tauri/pdfium/VERSION, args.gn  build version and build flags, as published
    src-tauri/pdfium/.fetch-stamp      what was installed (read by the idempotency check)

.PARAMETER Arch
  x64 (default) or arm64.

.PARAMETER Destination
  Target folder. Default: src-tauri/pdfium next to this repository's scripts folder.

.PARAMETER Force
  Download and reinstall even if the pinned build is already present.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts\fetch-pdfium.ps1
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts\fetch-pdfium.ps1 -Arch arm64
#>
param(
    [ValidateSet('x64', 'arm64')]
    [string]$Arch = 'x64',
    [string]$Destination = '',
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$staging = ''
# Invoke-WebRequest in PowerShell 5.1 is very slow while it draws a progress bar.
$ProgressPreference = 'SilentlyContinue'

# ---- Pinned release: the only place where the version and checksums live. ----
# To update: pick a release at https://github.com/bblanchon/pdfium-binaries/releases, take the
# non-V8 "pdfium-win-<arch>.tgz" assets, and copy their SHA-256 (shown on the release page as the
# asset digest; check it with `Get-FileHash` on a download).
$Pinned = @{
    Release = 'chromium/8111'
    Assets  = @{
        x64   = @{ File = 'pdfium-win-x64.tgz'; Sha256 = '3452d11031050ec65e57f180215b57b9e6d465e3fbedafa8a77ad90885c7d23e' }
        arm64 = @{ File = 'pdfium-win-arm64.tgz'; Sha256 = '99ec8b76d5ac30602a8488da745397393ff6cddc824ef36f398146f6b4f76d4e' }
    }
}
# ------------------------------------------------------------------------------

function Get-Sha256([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Read-Stamp([string]$Path) {
    $values = @{}
    if (-not (Test-Path -LiteralPath $Path)) { return $values }
    foreach ($line in Get-Content -LiteralPath $Path) {
        $parts = $line.Split('=', 2)
        if ($parts.Count -eq 2) { $values[$parts[0].Trim()] = $parts[1].Trim() }
    }
    return $values
}

# Returns the build flags in args.gn as a hashtable, for example pdf_enable_v8 -> false.
function Read-GnArgs([string]$Path) {
    $values = @{}
    foreach ($line in Get-Content -LiteralPath $Path) {
        $parts = $line.Split('=', 2)
        if ($parts.Count -eq 2) { $values[$parts[0].Trim()] = $parts[1].Trim().Trim('"') }
    }
    return $values
}

function Assert-NoV8NoXfa([string]$ArgsPath, [string]$ExpectedCpu) {
    if (-not (Test-Path -LiteralPath $ArgsPath)) { throw "args.gn is missing from the archive; cannot prove it is a non-V8, non-XFA build." }
    $gn = Read-GnArgs $ArgsPath
    if ($gn['pdf_enable_v8'] -ne 'false') { throw "This PDFium build has V8 (JavaScript) enabled (pdf_enable_v8 = $($gn['pdf_enable_v8'])). Only the non-V8 build is allowed." }
    if ($gn['pdf_enable_xfa'] -ne 'false') { throw "This PDFium build has XFA enabled (pdf_enable_xfa = $($gn['pdf_enable_xfa'])). Only the non-XFA build is allowed." }
    if ($gn['target_cpu'] -ne $ExpectedCpu) { throw "Archive is built for target_cpu '$($gn['target_cpu'])', expected '$ExpectedCpu'." }
}

function Get-TarExe {
    # Use Windows' bsdtar explicitly; a Git-for-Windows tar earlier in PATH cannot handle C:\ paths.
    $system = Join-Path $env:SystemRoot 'System32\tar.exe'
    if (Test-Path -LiteralPath $system) { return $system }
    return 'tar'
}

try {
    if (-not $Destination) {
        $Destination = Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\pdfium'
    }
    $Destination = [IO.Path]::GetFullPath($Destination)
    $asset = $Pinned.Assets[$Arch]
    $expectedSha = $asset.Sha256.ToLowerInvariant()
    $dllPath = Join-Path $Destination 'pdfium.dll'
    $stampPath = Join-Path $Destination '.fetch-stamp'

    # Idempotency: the same pinned build is installed and the DLL is untouched.
    if (-not $Force) {
        $stamp = Read-Stamp $stampPath
        $sameBuild = $stamp['release'] -eq $Pinned.Release -and $stamp['arch'] -eq $Arch -and $stamp['archive_sha256'] -eq $expectedSha
        if ($sameBuild -and (Test-Path -LiteralPath $dllPath) -and (Get-Sha256 $dllPath) -eq $stamp['dll_sha256']) {
            Write-Host "PDFium $($Pinned.Release) win-$Arch is already in $Destination (verified). Nothing to do."
            exit 0
        }
    }

    $url = 'https://github.com/bblanchon/pdfium-binaries/releases/download/{0}/{1}' -f [Uri]::EscapeDataString($Pinned.Release), $asset.File
    $staging = Join-Path $Destination '.download'
    if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $staging | Out-Null
    $archive = Join-Path $staging $asset.File

    Write-Host "Downloading $url"
    # PowerShell 5.1 defaults to TLS 1.0; GitHub needs TLS 1.2.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing

    $actualSha = Get-Sha256 $archive
    if ($actualSha -ne $expectedSha) {
        throw "SHA-256 MISMATCH for $($asset.File) ($($Pinned.Release)).`n  expected $expectedSha`n  actual   $actualSha`nThe download was deleted. Do not update the pinned hash unless you have checked the new release on purpose."
    }
    Write-Host "SHA-256 verified: $actualSha"

    $unpacked = Join-Path $staging 'unpacked'
    New-Item -ItemType Directory -Force -Path $unpacked | Out-Null
    & (Get-TarExe) -xzf $archive -C $unpacked
    if ($LASTEXITCODE -ne 0) { throw "tar failed to unpack $archive (exit code $LASTEXITCODE)." }

    Assert-NoV8NoXfa (Join-Path $unpacked 'args.gn') $Arch
    $newDll = Join-Path $unpacked 'bin\pdfium.dll'
    foreach ($required in @($newDll, (Join-Path $unpacked 'LICENSE'))) {
        if (-not (Test-Path -LiteralPath $required)) { throw "The archive does not contain $required." }
    }

    # Replace the previous install. Only files this script owns are removed.
    foreach ($old in @('pdfium.dll', 'LICENSE', 'licenses', 'VERSION', 'args.gn', '.fetch-stamp')) {
        $oldPath = Join-Path $Destination $old
        if (Test-Path -LiteralPath $oldPath) { Remove-Item -LiteralPath $oldPath -Recurse -Force }
    }
    Copy-Item -LiteralPath $newDll -Destination $Destination
    foreach ($item in @('LICENSE', 'licenses', 'VERSION', 'args.gn')) {
        $source = Join-Path $unpacked $item
        if (Test-Path -LiteralPath $source) { Copy-Item -LiteralPath $source -Destination $Destination -Recurse }
    }

    $stampLines = @(
        "release=$($Pinned.Release)",
        "arch=$Arch",
        "archive_sha256=$expectedSha",
        "dll_sha256=$(Get-Sha256 $dllPath)"
    )
    # Written last, so an interrupted run is never mistaken for a complete install.
    Set-Content -LiteralPath $stampPath -Value $stampLines -Encoding ascii
    Remove-Item -LiteralPath $staging -Recurse -Force

    Write-Host "Installed PDFium $($Pinned.Release) win-$Arch (no V8, no XFA) into $Destination"
    exit 0
}
catch {
    [Console]::Error.WriteLine("fetch-pdfium: ERROR: $($_.Exception.Message)")
    # Never leave a rejected or half-unpacked download behind.
    if ($staging -and (Test-Path -LiteralPath $staging)) { Remove-Item -LiteralPath $staging -Recurse -Force -ErrorAction SilentlyContinue }
    exit 1
}
