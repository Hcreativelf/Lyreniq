#requires -Version 5.1
param([switch]$SkipWebView2)
. (Join-Path $PSScriptRoot 'common.ps1')
try {
    Invoke-ProjectTask -Name 'test' -SkipWebView2:$SkipWebView2 -Action {
        Assert-ProjectDependencies
        foreach ($file in Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.ps1') {
            $tokens = $null
            $parseErrors = $null
            [System.Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$tokens, [ref]$parseErrors) | Out-Null
            if ($parseErrors.Count -gt 0) { throw "PowerShell syntax errors in $($file.Name): $parseErrors" }
        }
        Invoke-Checked -Command $script:CargoCommand -Arguments @('fmt', '--all', '--', '--check')
        Invoke-Checked -Command $script:CargoCommand -Arguments @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
        Invoke-Checked -Command $script:CargoCommand -Arguments @('test', '--workspace', '--locked')
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('-r', 'lint')
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('-r', 'typecheck')
        Invoke-Checked -Command $script:PnpmCommand -Arguments @('-r', 'test')
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
