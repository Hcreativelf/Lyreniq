#requires -Version 5.1
param([switch]$SkipWebView2)
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    Invoke-ProjectTask -Name 'bootstrap' -SkipWebView2:$SkipWebView2 -Action {
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('install', '--frozen-lockfile')
        Invoke-Checked -Command $script:CargoCommand -Arguments @('fetch', '--locked', '--target', 'x86_64-pc-windows-msvc')
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
