<#
.SYNOPSIS
  Memory sampler (T-010): samples the memory of SwiftPDF and all of its descendant processes,
  including the WebView2 msedgewebview2.exe processes, while the app is idle.

.DESCRIPTION
  Launch mode (default): starts the release exe, waits SettleMs, then takes one sample every
  IntervalMs for DurationMs, closes the app and repeats Runs times. Attach mode (-AppPid):
  samples an already running app (for example with a document open) once, without closing it.

  Each sample sums over the process tree:
    private_ws_mb  Private working set (Task Manager's "Memory" column). This is the number
                   checked against the §6.1 memory budgets. Shared DLL pages are not counted.
    working_set_mb Plain working set; counts shared DLL pages once per process, so it overstates.
    private_mb     Private bytes (commit charge).
  The process list is re-read for every sample, so WebView2 processes started late are included.

  Result: per run the median and max of the samples; overall the median of the per-run medians.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\memory.ps1 -Runs 5 -OutJson D:\dev\swiftpdf-bench\memory.json
.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\memory.ps1 -AppPid 1234 -DurationMs 5000
#>
param(
    [string]$Exe = 'D:\dev\swiftpdf-target\release\swiftpdf.exe',
    [int]$AppPid = 0,
    [int]$Runs = 5,
    [int]$SettleMs = 5000,
    [int]$DurationMs = 10000,
    [int]$IntervalMs = 1000,
    [string]$OutJson = ''
)

$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

public static class MemoryBench
{
    [StructLayout(LayoutKind.Sequential)]
    struct MemoryCountersEx2
    {
        public uint cb, PageFaultCount;
        public UIntPtr PeakWorkingSetSize, WorkingSetSize, QuotaPeakPagedPoolUsage, QuotaPagedPoolUsage;
        public UIntPtr QuotaPeakNonPagedPoolUsage, QuotaNonPagedPoolUsage, PagefileUsage, PeakPagefileUsage;
        public UIntPtr PrivateUsage, PrivateWorkingSetSize, SharedCommitUsage;
    }
    [DllImport("psapi.dll")] static extern bool GetProcessMemoryInfo(IntPtr process, out MemoryCountersEx2 counters, uint size);

    // { private working set, working set, private bytes } in bytes, or null if the process is gone.
    public static long[] Counters(int pid)
    {
        try
        {
            using (var process = Process.GetProcessById(pid))
            {
                MemoryCountersEx2 counters;
                uint size = (uint)Marshal.SizeOf(typeof(MemoryCountersEx2));
                if (!GetProcessMemoryInfo(process.Handle, out counters, size)) return null;
                return new long[] {
                    (long)counters.PrivateWorkingSetSize.ToUInt64(),
                    (long)counters.WorkingSetSize.ToUInt64(),
                    (long)counters.PrivateUsage.ToUInt64()
                };
            }
        }
        catch (Exception) { return null; }
    }
}
'@

function Get-ProcessTree([int]$rootPid) {
    $all = @(Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId, Name, CommandLine)
    $tree = @($all | Where-Object { $_.ProcessId -eq $rootPid })
    $frontier = @($rootPid)
    while ($frontier.Count -gt 0) {
        $children = @($all | Where-Object { $frontier -contains $_.ParentProcessId -and $_.ProcessId -ne $rootPid })
        $tree += $children
        $frontier = @($children | ForEach-Object { $_.ProcessId })
    }
    return $tree
}

# Chromium process role from its command line ("browser" when there is no --type).
function Get-Role($process) {
    if ($process.Name -ne 'msedgewebview2.exe') { return $process.Name }
    if ($process.CommandLine -match '--type=([a-z-]+)') {
        $role = $Matches[1]
        if ($role -eq 'utility' -and $process.CommandLine -match '--utility-sub-type=([A-Za-z.]+)') { $role = "utility:$($Matches[1])" }
        return "webview2:$role"
    }
    return 'webview2:browser'
}

function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    $n = $sorted.Count
    if ($n -eq 0) { return $null }
    if ($n % 2 -eq 1) { return $sorted[($n - 1) / 2] }
    return ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2
}

function Get-TreeSample([int]$rootPid) {
    $tree = @(Get-ProcessTree $rootPid)
    $privateWs = 0; $workingSet = 0; $private = 0
    $processes = @()
    foreach ($process in $tree) {
        $counters = [MemoryBench]::Counters([int]$process.ProcessId)
        if (-not $counters) { continue }
        $privateWs += $counters[0]; $workingSet += $counters[1]; $private += $counters[2]
        $processes += [pscustomobject]@{
            pid           = [int]$process.ProcessId
            role          = Get-Role $process
            private_ws_mb = [math]::Round($counters[0] / 1MB, 1)
        }
    }
    return [pscustomobject]@{
        processes      = $processes.Count
        webview2_procs = @($tree | Where-Object { $_.Name -eq 'msedgewebview2.exe' }).Count
        private_ws_mb  = [math]::Round($privateWs / 1MB, 1)
        working_set_mb = [math]::Round($workingSet / 1MB, 1)
        private_mb     = [math]::Round($private / 1MB, 1)
        breakdown      = $processes
    }
}

# Samples the tree of $rootPid for DurationMs and returns the run summary.
function Measure-Tree([int]$rootPid, [int]$run) {
    $samples = @()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    do {
        if (-not (Get-Process -Id $rootPid -ErrorAction SilentlyContinue)) { throw "Process $rootPid exited while sampling." }
        $samples += Get-TreeSample $rootPid
        $remaining = $IntervalMs - ($clock.ElapsedMilliseconds % $IntervalMs)
        if ($clock.ElapsedMilliseconds + $remaining -le $DurationMs) { Start-Sleep -Milliseconds $remaining }
    } while ($clock.ElapsedMilliseconds + 1 -lt $DurationMs)

    $privateWs = [double[]]@($samples | ForEach-Object { $_.private_ws_mb })
    $last = $samples[-1]
    $summary = [pscustomobject]@{
        run                  = $run
        samples              = $samples.Count
        processes            = $last.processes
        webview2_procs       = $last.webview2_procs
        private_ws_median_mb = Get-Median $privateWs
        private_ws_max_mb    = ($privateWs | Measure-Object -Maximum).Maximum
        working_set_median_mb = Get-Median ([double[]]@($samples | ForEach-Object { $_.working_set_mb }))
        private_median_mb    = Get-Median ([double[]]@($samples | ForEach-Object { $_.private_mb }))
        breakdown            = @($last.breakdown | Sort-Object private_ws_mb -Descending)
    }
    Write-Host ("run {0}: private WS median {1} MB, max {2} MB | working set {3} MB | private bytes {4} MB | {5} processes ({6} WebView2)" -f `
            $run, $summary.private_ws_median_mb, $summary.private_ws_max_mb, $summary.working_set_median_mb, `
            $summary.private_median_mb, $summary.processes, $summary.webview2_procs)
    return $summary
}

function Stop-AppTree([int]$rootPid) {
    $childPids = @(Get-ProcessTree $rootPid | Where-Object { $_.ProcessId -ne $rootPid } | ForEach-Object { [int]$_.ProcessId })
    $process = Get-Process -Id $rootPid -ErrorAction SilentlyContinue
    if ($process) {
        $null = $process.CloseMainWindow()
        if (-not $process.WaitForExit(3000)) { & taskkill.exe /PID $rootPid /T /F | Out-Null }
    }
    $deadline = (Get-Date).AddSeconds(5)
    foreach ($childPid in $childPids) {
        $child = Get-Process -Id $childPid -ErrorAction SilentlyContinue
        while ($child -and -not $child.HasExited -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 50 }
        if ($child -and -not $child.HasExited) { Stop-Process -Id $childPid -Force -ErrorAction SilentlyContinue }
    }
}

$runResults = @()
if ($AppPid -gt 0) {
    $runResults += Measure-Tree $AppPid 1
}
else {
    if (-not (Test-Path $Exe)) { throw "Exe not found: $Exe (build with 'npm run tauri build')" }
    if (Get-Process swiftpdf -ErrorAction SilentlyContinue) { throw 'Close all running swiftpdf.exe instances first.' }
    for ($run = 1; $run -le $Runs; $run++) {
        $process = Start-Process -FilePath $Exe -PassThru
        try {
            Start-Sleep -Milliseconds $SettleMs
            $runResults += Measure-Tree $process.Id $run
        }
        finally {
            Stop-AppTree $process.Id
            Start-Sleep -Milliseconds 1000
        }
    }
}

$medians = [double[]]@($runResults | ForEach-Object { $_.private_ws_median_mb })
$result = [pscustomobject]@{
    date                 = (Get-Date).ToString('s')
    mode                 = if ($AppPid -gt 0) { 'attach' } else { 'launch' }
    exe                  = $Exe
    runs                 = $runResults.Count
    settle_ms            = $SettleMs
    duration_ms          = $DurationMs
    interval_ms          = $IntervalMs
    private_ws_median_mb = Get-Median $medians
    private_ws_max_mb    = ($runResults | ForEach-Object { $_.private_ws_max_mb } | Measure-Object -Maximum).Maximum
    working_set_median_mb = Get-Median ([double[]]@($runResults | ForEach-Object { $_.working_set_median_mb }))
    private_median_mb    = Get-Median ([double[]]@($runResults | ForEach-Object { $_.private_median_mb }))
    webview2_procs_median = Get-Median ([double[]]@($runResults | ForEach-Object { $_.webview2_procs }))
    run_results          = $runResults
}

Write-Host ''
Write-Host ("idle memory (process tree incl. WebView2): private WS median {0} MB, max {1} MB | working set {2} MB | private bytes {3} MB | WebView2 procs {4}" -f `
        $result.private_ws_median_mb, $result.private_ws_max_mb, $result.working_set_median_mb, $result.private_median_mb, $result.webview2_procs_median)

if ($OutJson) {
    $dir = Split-Path $OutJson -Parent
    if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
    $result | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 $OutJson
    Write-Host "Wrote $OutJson"
}
