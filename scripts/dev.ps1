#requires -Version 5.1
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    Invoke-ProjectTask -Name 'dev' -Action {
        Assert-ProjectDependencies
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('tauri', 'dev', '--', '--locked')
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
