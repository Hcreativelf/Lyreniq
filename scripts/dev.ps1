#requires -Version 5.1
param([string]$BackgroundTestConfig)
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    $taskName = 'dev'
    if ($BackgroundTestConfig) { $taskName = 'dev-ui-smoke' }
    Invoke-ProjectTask -Name $taskName -Action {
        Assert-ProjectDependencies
        $arguments = @('tauri', 'dev')
        if ($BackgroundTestConfig) {
            $configPath = (Resolve-Path -LiteralPath $BackgroundTestConfig).Path
            $config = Get-Content -LiteralPath $configPath -Raw -Encoding UTF8 | ConvertFrom-Json
            if (@($config.app.windows).Count -ne 1) { throw 'Background test requires one explicitly hidden window.' }
            foreach ($window in $config.app.windows) {
                if ($window.visible -ne $false -or $window.focus -ne $false) {
                    throw 'Background test windows must be hidden and unfocused.'
                }
            }
            $arguments += @('--config', $configPath, '--no-watch')
        }
        $arguments += @('--', '--locked')
        Invoke-Checked -Command $script:PnpmCommand -Arguments $arguments
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
