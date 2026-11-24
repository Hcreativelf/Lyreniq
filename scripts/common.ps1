Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$script:ProjectRoot = Split-Path -Parent $PSScriptRoot
$script:ProjectPackage = Get-Content -LiteralPath (Join-Path $script:ProjectRoot 'package.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$script:RustVersion = [regex]::Match(
    (Get-Content -LiteralPath (Join-Path $script:ProjectRoot 'rust-toolchain.toml') -Raw),
    '(?m)^channel\s*=\s*"([0-9.]+)"'
).Groups[1].Value

function Get-ApplicationPath {
    param([string]$Name)
    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($command) { return $command.Source }
    return $null
}

function Initialize-ProjectTools {
    $paths = @()
    $script:NodeCommand = Get-ApplicationPath 'node'
    if ($script:NodeCommand) { $paths += Split-Path -Parent $script:NodeCommand }
    $sharedPnpm = Join-Path $env:APPDATA 'npm\pnpm.cmd'
    $script:PnpmCommand = Get-ApplicationPath 'pnpm'
    if (Test-Path -LiteralPath $sharedPnpm) { $script:PnpmCommand = $sharedPnpm }
    if ($script:PnpmCommand) { $paths += Split-Path -Parent $script:PnpmCommand }
    $cargoRoot = $env:CARGO_HOME
    if (-not $cargoRoot) { $cargoRoot = Join-Path $env:USERPROFILE '.cargo' }
    $paths += Join-Path $cargoRoot 'bin'
    $env:PATH = ($paths -join ';') + ';' + $env:PATH
    # Checking must never download a different package manager implicitly.
    $env:pnpm_config_pm_on_fail = 'error'
    $script:RustupCommand = Get-ApplicationPath 'rustup'
    $script:CargoCommand = Get-ApplicationPath 'cargo'
}

function Get-NativeOutput {
    param([string]$Command, [string[]]$Arguments = @())
    $result = (& $Command @Arguments | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "$Command failed (exit $LASTEXITCODE)" }
    return $result
}

function Invoke-Checked {
    param([string]$Command, [string[]]$Arguments = @())
    Write-Host ('> ' + [IO.Path]::GetFileName($Command) + ' ' + ($Arguments -join ' '))
    & $Command @Arguments | Out-Host
    $commandExit = $LASTEXITCODE
    if ($commandExit -ne 0) {
        Write-Host "[FAIL] $([IO.Path]::GetFileName($Command)) (exit $commandExit)"
        throw "$Command failed (exit $commandExit)"
    }
    Write-Host "[PASS] $([IO.Path]::GetFileName($Command)) (exit 0)"
}

function Assert-ProjectEnvironment {
    param([switch]$SkipWebView2)
    if ($env:OS -ne 'Windows_NT' -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
        throw 'Only Windows x64 is validated by this project.'
    }
    $failures = [System.Collections.Generic.List[string]]::new()
    foreach ($tool in @(
        @{ Name = 'Node'; Command = $script:NodeCommand; Version = 'v' + $script:ProjectPackage.engines.node },
        @{ Name = 'pnpm'; Command = $script:PnpmCommand; Version = $script:ProjectPackage.engines.pnpm }
    )) {
        try {
            if (-not $tool.Command) { throw 'not found on PATH or in the shared user installation' }
            $actual = Get-NativeOutput -Command $tool.Command -Arguments @('--version')
            if ($actual -ne $tool.Version) { throw "expected $($tool.Version), found $actual" }
            Write-Host "[PASS] $($tool.Name) $actual ($($tool.Command))"
        } catch { $failures.Add("$($tool.Name): $($_.Exception.Message)") }
    }
    try {
        if (-not $script:RustupCommand -or -not $script:CargoCommand) { throw 'rustup/cargo not installed' }
        $installed = Get-NativeOutput -Command $script:RustupCommand -Arguments @('toolchain', 'list')
        if ($installed -notmatch ([regex]::Escape($script:RustVersion) + '-x86_64-pc-windows-msvc\b')) {
            throw "install Rust $script:RustVersion for x86_64-pc-windows-msvc; check-env does not install it"
        }
        $actual = Get-NativeOutput -Command $script:RustupCommand -Arguments @('run', $script:RustVersion, 'rustc', '--version')
        $components = Get-NativeOutput -Command $script:RustupCommand -Arguments @('component', 'list', '--installed', '--toolchain', $script:RustVersion)
        foreach ($component in @('rustfmt', 'clippy')) {
            if ($components -notmatch ('(?m)^' + $component + '-')) { throw "missing $component" }
        }
        Write-Host "[PASS] $actual; rustfmt/clippy installed"
    } catch { $failures.Add("Rust: $($_.Exception.Message)") }
    try {
        $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
        if (-not (Test-Path -LiteralPath $vswhere)) { throw 'Visual Studio Installer / vswhere not found' }
        $msvc = Get-NativeOutput -Command $vswhere -Arguments @('-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath')
        if (-not $msvc) { throw 'install the MSVC x64/x86 component listed in .vsconfig' }
        Write-Host "[PASS] MSVC $msvc"
    } catch { $failures.Add("MSVC: $($_.Exception.Message)") }
    try {
        $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10'
        $sdk = @(Get-ChildItem -LiteralPath (Join-Path $kits 'Include') -Directory -ErrorAction SilentlyContinue | Where-Object {
            (Test-Path -LiteralPath (Join-Path $_.FullName 'um\Windows.h')) -and
            (Test-Path -LiteralPath (Join-Path $kits "Lib\$($_.Name)\um\x64\kernel32.lib")) -and
            (Test-Path -LiteralPath (Join-Path $kits "Lib\$($_.Name)\ucrt\x64\ucrt.lib"))
        } | Sort-Object { [version]$_.Name } -Descending)
        if ($sdk.Count -eq 0) { throw 'Windows SDK headers and x64 libraries not found; see .vsconfig' }
        Write-Host "[PASS] Windows SDK $($sdk[0].Name)"
    } catch { $failures.Add("Windows SDK: $($_.Exception.Message)") }
    if ($SkipWebView2) {
        Write-Host '[SKIP] WebView2 runtime check: headless build/CI requested; GUI is not verified.'
    } else {
        $runtimes = @()
        foreach ($base in @(${env:ProgramFiles(x86)}, $env:ProgramFiles, $env:LOCALAPPDATA)) {
            $runtimeRoot = Join-Path $base 'Microsoft\EdgeWebView\Application'
            if (Test-Path -LiteralPath $runtimeRoot) {
                $runtimes += @(Get-ChildItem -LiteralPath $runtimeRoot -Filter msedgewebview2.exe -File -Recurse -ErrorAction SilentlyContinue)
            }
        }
        if ($runtimes.Count -eq 0) { $failures.Add('WebView2 runtime not found in standard system/user locations') }
        else { Write-Host "[PASS] WebView2 $($runtimes[0].VersionInfo.ProductVersion)" }
    }
    if ($failures.Count -gt 0) { throw ("Environment check failed:`n - " + ($failures -join "`n - ")) }
}

function Assert-ProjectDependencies {
    if (-not (Test-Path -LiteralPath (Join-Path $script:ProjectRoot 'apps\desktop\node_modules\.bin\tauri.cmd'))) {
        throw 'Project dependencies are missing. Run scripts/bootstrap.ps1 first.'
    }
}

function Invoke-ProjectTask {
    param([string]$Name, [scriptblock]$Action, [switch]$SkipWebView2)
    $previousPath = $env:PATH
    $previousPmOnFail = $env:pnpm_config_pm_on_fail
    $transcript = $false
    Push-Location -LiteralPath $script:ProjectRoot
    try {
        New-Item -ItemType Directory -Path '.local/logs' -Force | Out-Null
        Start-Transcript -Path ".local/logs/$Name.log" -Force | Out-Null
        $transcript = $true
        Initialize-ProjectTools
        Assert-ProjectEnvironment -SkipWebView2:$SkipWebView2
        & $Action
    } finally {
        if ($transcript) { Stop-Transcript | Out-Null }
        Pop-Location
        $env:PATH = $previousPath
        $env:pnpm_config_pm_on_fail = $previousPmOnFail
    }
}
