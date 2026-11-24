#requires -Version 5.1
param([switch]$SkipWebView2)
. (Join-Path $PSScriptRoot 'common.ps1')
$previousPath = $env:PATH
$previousPmOnFail = $env:pnpm_config_pm_on_fail
Push-Location -LiteralPath $script:ProjectRoot
try {
    Initialize-ProjectTools
    Assert-ProjectEnvironment -SkipWebView2:$SkipWebView2
    Write-Host '[PASS] Environment ready. No tools or dependencies were installed.'
} catch {
    Write-Error -Message $_.Exception.Message -ErrorAction Continue
    exit 1
} finally {
    Pop-Location
    $env:PATH = $previousPath
    $env:pnpm_config_pm_on_fail = $previousPmOnFail
}
exit 0
