<#
.SYNOPSIS
  Checks a bench/run.ps1 results file against the §6.1 budgets and the recorded baseline.

.DESCRIPTION
  Fails (exit code 1) when any metric in the results
    - breaks its budget in bench/budgets.json (kind 'max': value must be < limit;
      kind 'min': value must be >= limit), or
    - regresses by more than the tolerance (default regression_tolerance in budgets.json, 10%)
      against bench/baseline.json. "Regresses" follows the metric's 'better' field: for 'lower'
      a value above baseline * 1.10 fails, for 'higher' a value below baseline * 0.90 fails.
  Metrics in the baseline but missing from the results are reported as warnings, or as failures
  with -Strict. Budgets that nothing measures yet are listed for information.
  Exit codes: 0 pass, 1 budget or regression failure, 2 bad input.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File bench\compare.ps1 -Results D:\dev\swiftpdf-bench\results-20261023-120000.json
#>
param(
    [Parameter(Mandatory = $true)][string]$Results,
    # Defaults: benchaseline.json and benchudgets.json next to this script.
    [string]$Baseline = '',
    [string]$Budgets = '',
    [double]$Tolerance = -1,
    [switch]$Strict
)

$ErrorActionPreference = 'Stop'

# Windows PowerShell 5.1 does not set $PSScriptRoot while binding parameter defaults.
if (-not $Baseline) { $Baseline = Join-Path $PSScriptRoot 'baseline.json' }
if (-not $Budgets) { $Budgets = Join-Path $PSScriptRoot 'budgets.json' }

function Read-Json([string]$path, [string]$what) {
    if (-not (Test-Path $path)) {
        Write-Host "$what not found: $path"
        exit 2
    }
    return Get-Content $path -Raw | ConvertFrom-Json
}

# ConvertFrom-Json gives objects, not hashtables: list a metrics object as name/value pairs.
function Get-Entries($object) {
    if ($null -eq $object) { return @() }
    return @($object.PSObject.Properties | ForEach-Object { [pscustomobject]@{ key = $_.Name; metric = $_.Value } })
}

function Test-Budget($budget, [double]$value) {
    if ($budget.kind -eq 'min') { return $value -ge $budget.limit }
    return $value -lt $budget.limit
}

$current = Read-Json $Results 'Results'
$base = Read-Json $Baseline 'Baseline'
$budgetFile = Read-Json $Budgets 'Budgets'
if ($Tolerance -lt 0) { $Tolerance = [double]$budgetFile.regression_tolerance }

$budgetsByMetric = @{}
foreach ($budget in $budgetFile.budgets) {
    foreach ($key in $budget.metrics) { $budgetsByMetric[$key] = $budget }
}

$failures = 0
$warnings = 0
$rows = @()
$currentEntries = Get-Entries $current.metrics
$baseEntries = Get-Entries $base.metrics

if ($base.machine.cpu -and $current.machine.cpu -and $base.machine.cpu -ne $current.machine.cpu) {
    Write-Host "WARNING: baseline CPU '$($base.machine.cpu)' differs from this run's '$($current.machine.cpu)'."
    $warnings++
}
if ($base.affinity -ne $current.affinity) {
    Write-Host "WARNING: baseline affinity '$($base.affinity)' differs from this run's '$($current.affinity)'."
    $warnings++
}

foreach ($entry in $currentEntries) {
    $key = $entry.key
    $metric = $entry.metric
    $value = [double]$metric.value
    $better = if ($metric.better) { $metric.better } else { 'lower' }
    $verdicts = @()

    $budgetText = '-'
    if ($budgetsByMetric.ContainsKey($key)) {
        $budget = $budgetsByMetric[$key]
        $sign = if ($budget.kind -eq 'min') { '>=' } else { '<' }
        $budgetText = "$sign $($budget.limit)"
        if (-not (Test-Budget $budget $value)) { $verdicts += 'OVER BUDGET' }
    }

    $baseText = '-'
    $changeText = '-'
    $baseMetric = $base.metrics.$key
    if ($null -ne $baseMetric -and [double]$baseMetric.value -gt 0) {
        $baseValue = [double]$baseMetric.value
        $change = ($value - $baseValue) / $baseValue
        $baseText = $baseValue
        $changeText = '{0:+0.0;-0.0;0.0}%' -f ($change * 100)
        $regressed = if ($better -eq 'higher') { $change -lt (-1 * $Tolerance) } else { $change -gt $Tolerance }
        if ($regressed) { $verdicts += 'REGRESSION' }
    }
    elseif ($null -eq $baseMetric) {
        $baseText = 'new'
    }

    $verdict = if ($verdicts.Count -gt 0) { $verdicts -join ', ' } else { 'pass' }
    if ($verdicts.Count -gt 0) { $failures++ }
    $rows += [pscustomobject]@{
        Metric   = $key
        Value    = $value
        Unit     = $metric.unit
        Baseline = $baseText
        Change   = $changeText
        Budget   = $budgetText
        Result   = $verdict
    }
}

foreach ($entry in $baseEntries) {
    if ($null -ne $current.metrics.($entry.key)) { continue }
    $result = if ($Strict) { 'MISSING' } else { 'missing (warning)' }
    if ($Strict) { $failures++ } else { $warnings++ }
    $rows += [pscustomobject]@{
        Metric = $entry.key; Value = '-'; Unit = $entry.metric.unit; Baseline = $entry.metric.value
        Change = '-'; Budget = '-'; Result = $result
    }
}

$rows | Format-Table -AutoSize | Out-String -Width 200 | Write-Host

$unmeasured = @($budgetFile.budgets | Where-Object { @($_.metrics).Count -eq 0 })
if ($unmeasured.Count -gt 0) {
    Write-Host 'Budgets not measured yet:'
    foreach ($budget in $unmeasured) { Write-Host ("  {0}: {1} {2} (by {3})" -f $budget.id, $budget.limit, $budget.unit, $budget.measured_by) }
    Write-Host ''
}

Write-Host ("Tolerance {0:P0}. {1} failure(s), {2} warning(s)." -f $Tolerance, $failures, $warnings)
if ($failures -gt 0) { exit 1 }
exit 0
