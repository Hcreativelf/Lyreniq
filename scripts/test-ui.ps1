#requires -Version 5.1
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    Invoke-ProjectTask -Name 'ui-smoke' -Action {
        Assert-ProjectDependencies
        Invoke-Checked -Command $script:NodeCommand -Arguments @((Join-Path $PSScriptRoot 'ui-smoke.mjs'))
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
