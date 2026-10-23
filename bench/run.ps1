<#
.SYNOPSIS
  Runs the SwiftPDF benchmark harness (T-010) and writes all results to one JSON file.

.DESCRIPTION
  Parts (skip any with -Skip):
    core    Criterion benches (bench/benches/first_page.rs): open + render page 1 of the 10 MB and
            the 1,000-page/300 MB file, and re-render of an open page. Steady state, warm caches.
            Then first-page-probe, ProbeRuns fresh processes per file: bind PDFium, open, render
            page 1, peak working set. Medians are reported; the first probe run is kept separately
            because it is the only one that may read the file from disk instead of the file cache.
    app     bench/cold-start.ps1 -Mode warm: process start -> visible window, -> content visible.
    memory  bench/memory.ps1: idle private working set of the app and all WebView2 processes.
    size    Uncompressed payload: swiftpdf.exe + pdfium.dll.

  -Affinity runs every part with the CPU affinity mask given (hex, for example 0x55 = 4 cores
  without SMT siblings on a 6-core/12-thread CPU). Child processes inherit it, including cargo's
  bench binary and the WebView2 processes. The build runs before the mask is applied.

  Each metric is { value, unit, better, n, ... }. bench/compare.ps1 checks the file against
  bench/budgets.json and bench/baseline.json; -Compare runs it and returns its exit code.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\run.ps1 -Compare
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\run.ps1 -Affinity 0x55 -OutJson D:\dev\swiftpdf-bench\results-4core.json
#>
param(
    [string]$Exe = 'D:\dev\swiftpdf-target\release\swiftpdf.exe',
    [string]$OutJson = '',
    [string]$Affinity = '',
    # Comma-separated parts to skip: core, app, memory, size (a string so it works with -File).
    [string]$Skip = '',
    [int]$ColdStartRuns = 20,
    [int]$ProbeRuns = 9,
    [int]$MemoryRuns = 5,
    [string]$Label = '',
    [switch]$Compare
)

$ErrorActionPreference = 'Stop'

$skipList = @($Skip.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ })
foreach ($part in $skipList) {
    if (@('core', 'app', 'memory', 'size') -notcontains $part) { throw "Unknown part to skip: $part (use core, app, memory, size)" }
}

$repoRoot = Split-Path $PSScriptRoot -Parent
$devRoot = if ($env:SWIFTPDF_DEV_ROOT) { $env:SWIFTPDF_DEV_ROOT } else { 'D:\dev' }
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repoRoot 'target' }
$corpusDir = if ($env:SWIFTPDF_CORPUS_DIR) { $env:SWIFTPDF_CORPUS_DIR } else { Join-Path $devRoot 'swiftpdf-corpus' }
$pdfiumDll = Join-Path $repoRoot 'src-tauri\pdfium\pdfium.dll'
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
if (-not $OutJson) { $OutJson = Join-Path $devRoot "swiftpdf-bench\results-$stamp.json" }
$rawDir = Join-Path (Split-Path $OutJson -Parent) "raw-$stamp"
New-Item -ItemType Directory -Force $rawDir | Out-Null

$metrics = [ordered]@{}

function Add-Metric([string]$key, $value, [string]$unit, [hashtable]$extra = @{}) {
    if ($null -eq $value) { return }
    $metric = [ordered]@{ value = [math]::Round([double]$value, 3); unit = $unit; better = 'lower' }
    foreach ($name in $extra.Keys) { $metric[$name] = $extra[$name] }
    $metrics[$key] = $metric
}

function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    $n = $sorted.Count
    if ($n -eq 0) { return $null }
    if ($n % 2 -eq 1) { return $sorted[($n - 1) / 2] }
    return ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2
}

function Invoke-Checked([string]$what, [scriptblock]$command) {
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$what failed with exit code $LASTEXITCODE" }
}

# "SSD (Apacer AS350)" style description of the disk that holds $path, or 'unknown'.
function Get-DiskKind([string]$path) {
    try {
        $letter = (Resolve-Path $path).Path.Substring(0, 1)
        $disk = Get-Partition -DriveLetter $letter | Get-Disk
        $physical = Get-PhysicalDisk | Where-Object { $_.DeviceId -eq [string]$disk.Number }
        return "$($letter): $($physical.MediaType) ($($physical.FriendlyName))"
    }
    catch { return 'unknown' }
}

function Get-MachineInfo {
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
    $system = Get-CimInstance Win32_ComputerSystem
    $os = Get-CimInstance Win32_OperatingSystem
    $webview2Dir = Join-Path ${env:ProgramFiles(x86)} 'Microsoft\EdgeWebView'
    return [ordered]@{
        cpu           = $cpu.Name.Trim()
        cores         = $cpu.NumberOfCores
        logical       = $cpu.NumberOfLogicalProcessors
        ram_gb        = [math]::Round($system.TotalPhysicalMemory / 1GB, 1)
        os            = "$($os.Caption) $($os.Version)"
        exe_disk      = if (Test-Path $Exe) { Get-DiskKind $Exe } else { 'n/a' }
        corpus_disk   = if (Test-Path $corpusDir) { Get-DiskKind $corpusDir } else { 'n/a' }
        webview2_disk = if (Test-Path $webview2Dir) { Get-DiskKind $webview2Dir } else { 'n/a' }
    }
}

function Get-Commit {
    try {
        $commit = (& git -C $repoRoot rev-parse --short HEAD 2>$null)
        if (-not $commit) { return 'unknown' }
        $dirty = (& git -C $repoRoot status --porcelain 2>$null)
        if ($dirty) { return "$commit+dirty" }
        return $commit
    }
    catch { return 'unknown' }
}

function Read-CriterionMedianMs([string]$benchName) {
    $file = Join-Path $targetDir "criterion\first_page\$benchName\new\estimates.json"
    if (-not (Test-Path $file)) { throw "Criterion estimates not found: $file" }
    $estimates = Get-Content $file -Raw | ConvertFrom-Json
    return [ordered]@{
        median_ms = $estimates.median.point_estimate / 1e6
        ci_low_ms = $estimates.median.confidence_interval.lower_bound / 1e6
        ci_high_ms = $estimates.median.confidence_interval.upper_bound / 1e6
    }
}

function Invoke-Probe([string]$fileName) {
    $probe = Join-Path $targetDir 'release\first-page-probe.exe'
    $pdf = Join-Path $corpusDir $fileName
    if (-not (Test-Path $pdf)) { throw "$pdf is missing: run 'cargo run --release -p swiftpdf-corpus-gen -- large'" }
    $runs = @()
    for ($run = 1; $run -le $ProbeRuns; $run++) {
        $line = & $probe $pdf --pdfium $pdfiumDll
        if ($LASTEXITCODE -ne 0) { throw "first-page-probe failed on $pdf" }
        $runs += ($line | ConvertFrom-Json)
    }
    $runs | ConvertTo-Json -Depth 3 | Set-Content -Encoding utf8 (Join-Path $rawDir "probe-$fileName.json")
    $firstPage = [double[]]@($runs | ForEach-Object { $_.first_page_ms })
    Write-Host ("probe {0}: first page median {1:N1} ms (first run {2:N1} ms), bind median {3:N1} ms, peak WS median {4:N1} MB" -f `
            $fileName, (Get-Median $firstPage), $runs[0].first_page_ms, (Get-Median ([double[]]@($runs | ForEach-Object { $_.bind_ms }))), `
            (Get-Median ([double[]]@($runs | ForEach-Object { $_.peak_working_set_mb }))))
    return $runs
}

function Measure-Core {
    $sizes = @{ '10mb' = 'large_10mb.pdf'; '300mb' = 'large_1000pages_300mb.pdf' }
    Invoke-Checked 'cargo bench' { cargo bench -p swiftpdf-bench --bench first_page }
    foreach ($size in @('10mb', '300mb')) {
        $criterion = Read-CriterionMedianMs "open_render_$size"
        Add-Metric "core.first_page_$($size)_ms" $criterion.median_ms 'ms' @{
            source = 'criterion first_page/open_render_' + $size; ci_low_ms = [math]::Round($criterion.ci_low_ms, 3); ci_high_ms = [math]::Round($criterion.ci_high_ms, 3)
        }
        $runs = Invoke-Probe $sizes[$size]
        Add-Metric "core.first_page_$($size)_fresh_ms" (Get-Median ([double[]]@($runs | ForEach-Object { $_.first_page_ms }))) 'ms' @{
            source = 'first-page-probe (fresh process)'; n = $runs.Count; first_run_ms = $runs[0].first_page_ms
            max_ms = ($runs | ForEach-Object { $_.first_page_ms } | Measure-Object -Maximum).Maximum
        }
        if ($size -eq '300mb') {
            Add-Metric 'core.peak_ws_300mb_first_page_mb' (Get-Median ([double[]]@($runs | ForEach-Object { $_.peak_working_set_mb }))) 'MB' @{
                source = 'first-page-probe peak working set'; n = $runs.Count
            }
        }
        if ($size -eq '10mb') {
            Add-Metric 'core.pdfium_bind_ms' (Get-Median ([double[]]@($runs | ForEach-Object { $_.bind_ms }))) 'ms' @{
                source = 'first-page-probe (load pdfium.dll + init)'; n = $runs.Count
            }
        }
    }
    $rerender = Read-CriterionMedianMs 'rerender_page1_10mb'
    Add-Metric 'core.rerender_page1_10mb_ms' $rerender.median_ms 'ms' @{
        source = 'criterion first_page/rerender_page1_10mb'; ci_low_ms = [math]::Round($rerender.ci_low_ms, 3); ci_high_ms = [math]::Round($rerender.ci_high_ms, 3)
    }
}

function Measure-App {
    $raw = Join-Path $rawDir 'cold-start-warm.json'
    & (Join-Path $PSScriptRoot 'cold-start.ps1') -Exe $Exe -Mode warm -Runs $ColdStartRuns -OutJson $raw
    $result = Get-Content $raw -Raw | ConvertFrom-Json
    $n = $ColdStartRuns - $result.timeouts
    Add-Metric 'cold_start.visible_window_ms' $result.wall_median_ms 'ms' @{ source = 'cold-start.ps1 warm, wall'; n = $n; p95_ms = $result.wall_p95_ms }
    Add-Metric 'cold_start.content_visible_ms' $result.internal_median_ms 'ms' @{ source = 'cold-start.ps1 warm, internal'; n = $n; p95_ms = $result.internal_p95_ms }
    Add-Metric 'cold_start.webview_built_ms' $result.webview_built_median_ms 'ms' @{ source = 'cold-start.ps1 warm, Tauri setup hook'; n = $n }
}

function Measure-Memory {
    $raw = Join-Path $rawDir 'memory-idle.json'
    & (Join-Path $PSScriptRoot 'memory.ps1') -Exe $Exe -Runs $MemoryRuns -OutJson $raw
    $result = Get-Content $raw -Raw | ConvertFrom-Json
    Add-Metric 'memory.idle_private_ws_mb' $result.private_ws_median_mb 'MB' @{
        source = 'memory.ps1, empty window, app + WebView2 private working set'; n = $result.runs; max_mb = $result.private_ws_max_mb
        webview2_procs = $result.webview2_procs_median
    }
    Add-Metric 'memory.idle_private_bytes_mb' $result.private_median_mb 'MB' @{ source = 'memory.ps1, private bytes (commit)'; n = $result.runs }
}

function Measure-Size {
    $exeBytes = (Get-Item $Exe).Length
    $dllBytes = (Get-Item $pdfiumDll).Length
    Add-Metric 'size.payload_mb' (($exeBytes + $dllBytes) / 1MB) 'MB' @{
        source = 'swiftpdf.exe + pdfium.dll, uncompressed'; exe_mb = [math]::Round($exeBytes / 1MB, 2); pdfium_mb = [math]::Round($dllBytes / 1MB, 2)
    }
}

if ($skipList -notcontains 'core') {
    if (-not (Test-Path $pdfiumDll)) { throw "$pdfiumDll is missing: run scripts\fetch-pdfium.ps1" }
    Invoke-Checked 'building the benches' { cargo bench -p swiftpdf-bench --bench first_page --no-run }
    Invoke-Checked 'building first-page-probe' { cargo build --release -p swiftpdf-bench --bin first-page-probe }
}
if (($skipList -notcontains 'app' -or $skipList -notcontains 'memory' -or $skipList -notcontains 'size') -and -not (Test-Path $Exe)) {
    throw "Exe not found: $Exe (build with 'npm run tauri build', or skip app, memory and size)"
}

$self = [Diagnostics.Process]::GetCurrentProcess()
$originalAffinity = $self.ProcessorAffinity
try {
    if ($Affinity) {
        $self.ProcessorAffinity = [IntPtr][Convert]::ToInt64($Affinity, 16)
        Write-Host "CPU affinity for this run and its child processes: $Affinity"
    }
    if ($skipList -notcontains 'core') { Measure-Core }
    if ($skipList -notcontains 'app') { Measure-App }
    if ($skipList -notcontains 'memory') { Measure-Memory }
    if ($skipList -notcontains 'size') { Measure-Size }
}
finally {
    $self.ProcessorAffinity = $originalAffinity
}

$results = [ordered]@{
    schema   = 1
    date     = (Get-Date).ToString('s')
    label    = $Label
    commit   = Get-Commit
    affinity = if ($Affinity) { $Affinity } else { 'all' }
    machine  = Get-MachineInfo
    settings = [ordered]@{
        cold_start_runs = $ColdStartRuns; probe_runs = $ProbeRuns; memory_runs = $MemoryRuns
        render_width_px = 1280; criterion_samples = 20; skipped = $skipList; raw_dir = $rawDir
    }
    metrics  = $metrics
}
$results | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 $OutJson
Write-Host ''
Write-Host "Wrote $OutJson"
foreach ($key in $metrics.Keys) { Write-Host ("  {0,-36} {1,10} {2}" -f $key, $metrics[$key].value, $metrics[$key].unit) }

if ($Compare) {
    & (Join-Path $PSScriptRoot 'compare.ps1') -Results $OutJson
    exit $LASTEXITCODE
}
