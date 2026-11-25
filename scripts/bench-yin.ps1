#requires -Version 5.1
param([switch]$SkipWebView2)
. (Join-Path $PSScriptRoot 'common.ps1')

function Invoke-YinProcess {
    param([string]$Arguments, [string]$OutputPath)
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = Join-Path $script:ProjectRoot 'target/release/examples/yin_report.exe'
    $info.Arguments = $Arguments
    $info.WorkingDirectory = $script:ProjectRoot
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $watch = [Diagnostics.Stopwatch]::StartNew()
    try {
        if (-not $process.Start()) { throw 'Could not start YIN benchmark' }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.WaitForExit()
        $watch.Stop()
        $output = $stdout.GetAwaiter().GetResult()
        $errorOutput = $stderr.GetAwaiter().GetResult()
        [IO.File]::WriteAllText($OutputPath, $output, [Text.UTF8Encoding]::new($false))
        if ($process.ExitCode -ne 0) { throw "Benchmark exit $($process.ExitCode): $errorOutput" }
        $cpu = $process.TotalProcessorTime.TotalSeconds
        return [ordered]@{
            cpu_seconds = $cpu
            wall_seconds = $watch.Elapsed.TotalSeconds
            busy_one_core_percent = 100.0 * $cpu / $watch.Elapsed.TotalSeconds
            busy_machine_percent = 100.0 * $cpu / $watch.Elapsed.TotalSeconds / [Environment]::ProcessorCount
        }
    } finally { $process.Dispose() }
}

try {
    Invoke-ProjectTask -Name 'bench-yin' -SkipWebView2:$SkipWebView2 -Action {
        Invoke-Checked -Command $script:CargoCommand -Arguments @('build', '-p', 'tuner-core', '--example', 'yin_report', '--release', '--locked')
        $stamp = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH-mm-ss-fffZ')
        $runDirectory = Join-Path $script:ProjectRoot ".local/dsp-bench/$stamp"
        New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null
        $full = Invoke-YinProcess -Arguments '' -OutputPath (Join-Path $runDirectory 'full.csv')
        $engines = [ordered]@{}
        foreach ($engine in @('T06', 'T07')) {
            $engines[$engine] = Invoke-YinProcess -Arguments "--benchmark-only --engine $engine" -OutputPath (Join-Path $runDirectory "cpu-$engine.csv")
        }
        $metadata = [ordered]@{
            cpu = [Microsoft.Win32.Registry]::GetValue('HKEY_LOCAL_MACHINE\HARDWARE\DESCRIPTION\System\CentralProcessor\0', 'ProcessorNameString', 'unknown')
            logical_processors = [Environment]::ProcessorCount
            os = [Environment]::OSVersion.VersionString
            rust = $script:RustVersion
            profile = 'release; default Windows x64 target features'
            full_report_process = $full
            cpu_benchmark_processes = $engines
            measured_calls_per_engine = 640
            warmup_calls_per_engine = 30
            note = 'Batch process CPU includes generation, warmups and output; frame quantiles exclude warmups. Desktop remains in use. Do not run beside builds.'
            run_directory = $runDirectory
        }
        $json = $metadata | ConvertTo-Json -Depth 6
        [IO.File]::WriteAllText((Join-Path $runDirectory 'machine.json'), $json, [Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $script:ProjectRoot '.local/dsp-bench/latest.json'), $json, [Text.UTF8Encoding]::new($false))
        Write-Host "[PASS] YIN comparison and CPU benchmark: $runDirectory"
    }
} catch { Write-Error -Message $_.Exception.Message -ErrorAction Continue; exit 1 }
exit 0
