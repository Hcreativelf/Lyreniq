#requires -Version 5.1
param([switch]$SkipWebView2)
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    Invoke-ProjectTask -Name 'build' -SkipWebView2:$SkipWebView2 -Action {
        Assert-ProjectDependencies
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('tauri', 'build', '--no-bundle', '--ci', '--', '--locked')
        Write-Host '[SKIP] Installer bundling is deferred to T24. The release executable is in target/release.'
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
