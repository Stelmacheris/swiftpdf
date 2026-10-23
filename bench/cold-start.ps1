<#
.SYNOPSIS
  Measures SwiftPDF start-up: process start -> visible main window, process start -> content
  painted, plus WebView2 process count and memory. Used by T-005, T-111 and later by the benchmark
  harness (T-010).

.DESCRIPTION
  Each run launches the release exe, then measures two numbers:
    wall_ms     External clock: from just before CreateProcess until a visible top-level window
                owned by the process exists (polled about every 1 ms).
                Since T-111 the window is created visible with the theme background, so this is
                the "visible window" time (an empty dark frame while WebView2 starts).
    internal_ms "Content visible": from the kernel's process creation time until the UI reports its
                first paint (`window_ready` writes the `content_ready` mark). Written by the app only
                when SWIFTPDF_STARTUP_LOG is set (off by default). Builds before T-111 wrote
                `shown_by_ui` / `shown_by_fallback` instead; those marks are still read.
  After the window is visible the script waits SettleMs, then samples the app and all of its
  descendant processes (the WebView2 msedgewebview2.exe processes): count, working set, private bytes.

  Modes:
    warm          Normal relaunch; exe, WebView2 runtime and profile are in the file cache.
    fresh-profile Deletes the app's WebView2 user data folder (EBWebView) before every run, like the
                  first launch after install.
    evict         Before every run, fills free + standby RAM with touched pages so Windows drops its
                  file cache, then frees it. Approximates a post-reboot start without admin rights.
                  Files mapped by running processes (for example a WebView2 runtime used by another
                  app) stay resident, so this is "as cold as possible", not a true reboot.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\cold-start.ps1 -Mode warm -Runs 20 -OutJson D:\dev\swiftpdf-bench\warm.json
#>
param(
    [string]$Exe = 'D:\dev\swiftpdf-target\release\swiftpdf.exe',
    [ValidateSet('warm', 'fresh-profile', 'evict')]
    [string]$Mode = 'warm',
    [int]$Runs = 20,
    [int]$SettleMs = 2000,
    [int]$TimeoutMs = 10000,
    [string]$OutJson = '',
    # Title of the main window; other visible top-level windows of the process (tao's hidden
    # event-target window reports itself as visible) are ignored.
    [string]$WindowTitle = 'SwiftPDF',
    # Optional extra Chromium switches for experiments, passed via WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS.
    [string]$BrowserArgs = '',
    # evict mode: pause after freeing the filler memory so other processes can page back in first.
    [int]$EvictSettleMs = 5000
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path $Exe)) { throw "Exe not found: $Exe (build with 'npm run tauri build')" }

# The WebView2 user data folder Tauri uses for identifier org.swiftpdf.viewer.
$profileDir = Join-Path $env:LOCALAPPDATA 'org.swiftpdf.viewer\EBWebView'

Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

public static class StartBench
{
    delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr hWnd, uint cmd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int max);

    [StructLayout(LayoutKind.Sequential)]
    class MemoryStatusEx
    {
        public uint dwLength = (uint)Marshal.SizeOf(typeof(MemoryStatusEx));
        public uint dwMemoryLoad;
        public ulong ullTotalPhys, ullAvailPhys, ullTotalPageFile, ullAvailPageFile;
        public ulong ullTotalVirtual, ullAvailVirtual, ullAvailExtendedVirtual;
    }
    [DllImport("kernel32.dll")] static extern bool GlobalMemoryStatusEx([In, Out] MemoryStatusEx status);

    [StructLayout(LayoutKind.Sequential)]
    struct MemoryCountersEx2
    {
        public uint cb, PageFaultCount;
        public UIntPtr PeakWorkingSetSize, WorkingSetSize, QuotaPeakPagedPoolUsage, QuotaPagedPoolUsage;
        public UIntPtr QuotaPeakNonPagedPoolUsage, QuotaNonPagedPoolUsage, PagefileUsage, PeakPagefileUsage;
        public UIntPtr PrivateUsage, PrivateWorkingSetSize, SharedCommitUsage;
    }
    [DllImport("psapi.dll")] static extern bool GetProcessMemoryInfo(IntPtr process, out MemoryCountersEx2 counters, uint size);

    // Private working set in bytes (Task Manager's "Memory" column), or 0 if the process is gone.
    public static long PrivateWorkingSet(int pid)
    {
        try
        {
            using (var process = Process.GetProcessById(pid))
            {
                MemoryCountersEx2 counters;
                uint size = (uint)Marshal.SizeOf(typeof(MemoryCountersEx2));
                if (!GetProcessMemoryInfo(process.Handle, out counters, size)) return 0;
                return (long)counters.PrivateWorkingSetSize.ToUInt64();
            }
        }
        catch (Exception) { return 0; }
    }

    static bool HasVisibleMainWindow(uint pid, string title)
    {
        bool found = false;
        EnumWindows(delegate (IntPtr hWnd, IntPtr lParam)
        {
            uint owner;
            GetWindowThreadProcessId(hWnd, out owner);
            // GW_OWNER = 4: only unowned top-level windows count as the app window.
            if (owner == pid && IsWindowVisible(hWnd) && GetWindow(hWnd, 4) == IntPtr.Zero && TitleOf(hWnd) == title)
            {
                found = true;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    static string TitleOf(IntPtr hWnd)
    {
        var text = new System.Text.StringBuilder(256);
        GetWindowText(hWnd, text, text.Capacity);
        return text.ToString();
    }

    // Starts the exe and returns { pid, wall ms until its main window is visible, or -1 on timeout }.
    public static double[] LaunchAndWait(string exe, string title, int timeoutMs)
    {
        var info = new ProcessStartInfo(exe);
        info.UseShellExecute = false;
        var clock = Stopwatch.StartNew();
        var process = Process.Start(info);
        uint pid = (uint)process.Id;
        while (clock.ElapsedMilliseconds < timeoutMs)
        {
            if (HasVisibleMainWindow(pid, title))
            {
                return new double[] { pid, clock.Elapsed.TotalMilliseconds };
            }
            System.Threading.Thread.Sleep(1);
        }
        return new double[] { pid, -1 };
    }

    // Fills free + standby memory with touched pages so the file cache is repurposed, then frees it.
    public static long EvictFileCache(long keepFreeMb)
    {
        const long chunk = 64L * 1024 * 1024;
        var blocks = new System.Collections.Generic.List<IntPtr>();
        long total = 0;
        try
        {
            while (true)
            {
                var status = new MemoryStatusEx();
                GlobalMemoryStatusEx(status);
                if ((long)(status.ullAvailPhys / (1024 * 1024)) < keepFreeMb) break;
                IntPtr block = Marshal.AllocHGlobal(new IntPtr(chunk));
                blocks.Add(block);
                for (long offset = 0; offset < chunk; offset += 4096)
                {
                    Marshal.WriteByte(new IntPtr(block.ToInt64() + offset), 1);
                }
                total += chunk;
            }
        }
        catch (OutOfMemoryException) { }
        finally
        {
            foreach (var block in blocks) Marshal.FreeHGlobal(block);
        }
        return total / (1024 * 1024);
    }
}
'@

function Get-Descendants([int]$rootPid) {
    $all = Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId, Name
    $result = @()
    $frontier = @($rootPid)
    while ($frontier.Count -gt 0) {
        $children = @($all | Where-Object { $frontier -contains $_.ParentProcessId -and $_.ProcessId -ne $rootPid })
        $result += $children
        $frontier = @($children | ForEach-Object { $_.ProcessId })
    }
    return $result
}

function Get-Percentile([double[]]$values, [double]$p) {
    $sorted = @($values | Sort-Object)
    if ($sorted.Count -eq 0) { return $null }
    $rank = [math]::Ceiling($p / 100 * $sorted.Count)
    return $sorted[[math]::Max(0, $rank - 1)]
}

function Get-Median([double[]]$values) {
    $sorted = @($values | Sort-Object)
    $n = $sorted.Count
    if ($n -eq 0) { return $null }
    if ($n % 2 -eq 1) { return $sorted[($n - 1) / 2] }
    return ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2
}

function Stop-AppTree([int]$appPid, [int[]]$childPids) {
    $process = Get-Process -Id $appPid -ErrorAction SilentlyContinue
    if ($process) {
        $null = $process.CloseMainWindow()
        if (-not $process.WaitForExit(3000)) { & taskkill.exe /PID $appPid /T /F | Out-Null }
    }
    # WebView2 children linger briefly after the host exits; the next run must not reuse them.
    $deadline = (Get-Date).AddSeconds(5)
    foreach ($childPid in $childPids) {
        $child = Get-Process -Id $childPid -ErrorAction SilentlyContinue
        while ($child -and -not $child.HasExited -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 50 }
        if ($child -and -not $child.HasExited) { Stop-Process -Id $childPid -Force -ErrorAction SilentlyContinue }
    }
}

if (Get-Process swiftpdf -ErrorAction SilentlyContinue) { throw 'Close all running swiftpdf.exe instances first.' }

$logFile = Join-Path ([IO.Path]::GetTempPath()) "swiftpdf-startup-$PID.log"
$env:SWIFTPDF_STARTUP_LOG = $logFile
if ($BrowserArgs) { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $BrowserArgs }
$epoch = [DateTime]::new(1970, 1, 1, 0, 0, 0, [DateTimeKind]::Utc)
$samples = @()

for ($run = 1; $run -le $Runs; $run++) {
    Remove-Item $logFile -ErrorAction SilentlyContinue
    $evictedMb = 0
    if ($Mode -eq 'fresh-profile' -and (Test-Path $profileDir)) { Remove-Item $profileDir -Recurse -Force }
    if ($Mode -eq 'evict') { $evictedMb = [StartBench]::EvictFileCache(300); Start-Sleep -Milliseconds $EvictSettleMs }

    $result = [StartBench]::LaunchAndWait($Exe, $WindowTitle, $TimeoutMs)
    $appPid = [int]$result[0]
    $wallMs = [math]::Round($result[1], 1)
    $process = Get-Process -Id $appPid
    $startUs = ($process.StartTime.ToUniversalTime() - $epoch).Ticks / 10

    Start-Sleep -Milliseconds $SettleMs

    $marks = @{}
    if (Test-Path $logFile) {
        foreach ($line in Get-Content $logFile) {
            $parts = $line.Split(' ')
            if (-not $marks.ContainsKey($parts[0])) { $marks[$parts[0]] = [double]$parts[1] }
        }
    }
    # content_ready: T-111 and later. shown_by_*: older builds that showed the window on first paint.
    $shownBy = if ($marks.ContainsKey('content_ready') -or $marks.ContainsKey('shown_by_ui')) { 'ui' } elseif ($marks.ContainsKey('shown_by_fallback')) { 'fallback' } else { 'none' }
    $shownUs = if ($marks.ContainsKey('content_ready')) { $marks['content_ready'] } elseif ($marks.ContainsKey('shown_by_ui')) { $marks['shown_by_ui'] } elseif ($marks.ContainsKey('shown_by_fallback')) { $marks['shown_by_fallback'] } else { $null }
    $internalMs = if ($shownUs) { [math]::Round(($shownUs - $startUs) / 1000, 1) } else { $null }
    $mainMs = if ($marks.ContainsKey('main')) { [math]::Round(($marks['main'] - $startUs) / 1000, 1) } else { $null }
    $webviewMs = if ($marks.ContainsKey('webview_built')) { [math]::Round(($marks['webview_built'] - $startUs) / 1000, 1) } else { $null }

    $children = @(Get-Descendants $appPid)
    $webviewPids = @($children | Where-Object { $_.Name -eq 'msedgewebview2.exe' } | ForEach-Object { [int]$_.ProcessId })
    $allProcs = @(Get-Process -Id (@($appPid) + @($children | ForEach-Object { [int]$_.ProcessId })) -ErrorAction SilentlyContinue)
    $appProc = $allProcs | Where-Object { $_.Id -eq $appPid }
    # Private working set is what Task Manager's "Memory" column shows; plain working sets double-count shared DLL pages.
    $treePids = @($allProcs | ForEach-Object { $_.Id })
    $privateWs = ($treePids | ForEach-Object { [StartBench]::PrivateWorkingSet($_) } | Measure-Object -Sum).Sum
    $sample = [pscustomobject]@{
        run              = $run
        mode             = $Mode
        wall_ms          = $wallMs
        internal_ms      = $internalMs
        main_ms          = $mainMs
        webview_built_ms = $webviewMs
        shown_by         = $shownBy
        webview2_procs   = $webviewPids.Count
        total_ws_mb      = [math]::Round((($allProcs | Measure-Object WorkingSet64 -Sum).Sum) / 1MB, 1)
        total_private_mb = [math]::Round((($allProcs | Measure-Object PrivateMemorySize64 -Sum).Sum) / 1MB, 1)
        total_private_ws_mb = [math]::Round($privateWs / 1MB, 1)
        app_ws_mb        = [math]::Round($appProc.WorkingSet64 / 1MB, 1)
        evicted_mb       = $evictedMb
    }
    $samples += $sample
    Write-Host ("run {0,2}: visible {1,7} ms  content {2,7} ms  main {3,5} ms  webview_built {4,7} ms  shown_by {5}  webview2 {6}  ws {7} MB  private {8} MB  private_ws {9} MB" -f `
            $run, $wallMs, $internalMs, $mainMs, $webviewMs, $shownBy, $webviewPids.Count, $sample.total_ws_mb, $sample.total_private_mb, $sample.total_private_ws_mb)

    Stop-AppTree $appPid $webviewPids
    Start-Sleep -Milliseconds 1000
}

Remove-Item Env:\SWIFTPDF_STARTUP_LOG
if ($BrowserArgs) { Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS }
Remove-Item $logFile -ErrorAction SilentlyContinue

$valid = @($samples | Where-Object { $_.wall_ms -ge 0 })
$wall = [double[]]@($valid | ForEach-Object { $_.wall_ms })
$internal = [double[]]@($valid | Where-Object { $_.internal_ms } | ForEach-Object { $_.internal_ms })
$summary = [pscustomobject]@{
    date                  = (Get-Date).ToString('s')
    mode                  = $Mode
    browser_args          = $BrowserArgs
    exe                   = $Exe
    runs                  = $Runs
    timeouts              = $Runs - $valid.Count
    fallback_shows        = @($samples | Where-Object { $_.shown_by -ne 'ui' }).Count
    wall_median_ms        = Get-Median $wall
    wall_p95_ms           = Get-Percentile $wall 95
    wall_min_ms           = ($wall | Measure-Object -Minimum).Minimum
    wall_max_ms           = ($wall | Measure-Object -Maximum).Maximum
    internal_median_ms    = Get-Median $internal
    internal_p95_ms       = Get-Percentile $internal 95
    main_median_ms        = Get-Median ([double[]]@($valid | Where-Object { $_.main_ms } | ForEach-Object { $_.main_ms }))
    webview_built_median_ms = Get-Median ([double[]]@($valid | Where-Object { $_.webview_built_ms } | ForEach-Object { $_.webview_built_ms }))
    webview2_procs_median = Get-Median ([double[]]@($valid | ForEach-Object { $_.webview2_procs }))
    total_ws_median_mb    = Get-Median ([double[]]@($valid | ForEach-Object { $_.total_ws_mb }))
    total_private_median_mb = Get-Median ([double[]]@($valid | ForEach-Object { $_.total_private_mb }))
    total_private_ws_median_mb = Get-Median ([double[]]@($valid | ForEach-Object { $_.total_private_ws_mb }))
    samples               = $samples
}

Write-Host ''
Write-Host ("{0}: visible window (wall) median {1} ms, p95 {2} ms | content visible (internal) median {3} ms, p95 {4} ms | WebView2 procs {5} | WS {6} MB, private {7} MB, private WS {8} MB" -f `
        $Mode, $summary.wall_median_ms, $summary.wall_p95_ms, $summary.internal_median_ms, $summary.internal_p95_ms, `
        $summary.webview2_procs_median, $summary.total_ws_median_mb, $summary.total_private_median_mb, $summary.total_private_ws_median_mb)

if ($OutJson) {
    $dir = Split-Path $OutJson -Parent
    if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
    $summary | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 $OutJson
    Write-Host "Wrote $OutJson"
}
