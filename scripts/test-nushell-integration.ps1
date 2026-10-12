#requires -Version 7.0
param(
    [string]$NuExe = 'D:\Program Files\nu\bin\nu.exe',
    [string]$BuildDirectory = 'target\release',
    [string]$OutputDirectory = '',
    [ValidateRange(1,300)][int]$TimeoutSeconds = 30
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$version = (& $NuExe -n --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0 -or $version -notmatch '^0\.116\.') { throw "Expected Nushell 0.116.x; found $version" }
if (-not [IO.Path]::IsPathRooted($BuildDirectory)) { $BuildDirectory = Join-Path $projectRoot $BuildDirectory }
if (-not $OutputDirectory) { $OutputDirectory = Join-Path $projectRoot ('.tmp\nu-integration-' + [guid]::NewGuid()) }
$OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a new evidence directory; never overwrite a run' }
foreach ($sub in @('bin','nu','tests')) { New-Item -ItemType Directory -Path (Join-Path $OutputDirectory $sub) -Force | Out-Null }
foreach ($name in @('r3viewer.exe','nu_plugin_r3viewer.exe')) { Copy-Item -LiteralPath (Join-Path $BuildDirectory $name) -Destination (Join-Path $OutputDirectory 'bin') }
Copy-Item -LiteralPath (Join-Path $projectRoot 'nu\r3viewer.nu') -Destination (Join-Path $OutputDirectory 'nu')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'test-nushell-integration.nu') -Destination (Join-Path $OutputDirectory 'tests')
$configPath = Join-Path $OutputDirectory 'empty.toml'
[IO.File]::WriteAllText($configPath, '')
[IO.File]::WriteAllText((Join-Path $OutputDirectory 'tests\source.txt'), 'stream α')
[IO.File]::WriteAllText((Join-Path $OutputDirectory 'tests\source.csv'), "name,n`nAna,42`n" )
$identities = foreach ($path in @($NuExe,(Join-Path $OutputDirectory 'bin\r3viewer.exe'),(Join-Path $OutputDirectory 'bin\nu_plugin_r3viewer.exe'),(Join-Path $OutputDirectory 'nu\r3viewer.nu'),(Join-Path $OutputDirectory 'tests\test-nushell-integration.nu'),$configPath,(Join-Path $OutputDirectory 'tests\source.txt'),(Join-Path $OutputDirectory 'tests\source.csv'))) {
    $item = Get-Item -LiteralPath $path
    @{path=$item.FullName;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash}
}
@{nushell=$version;inputs=$identities} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'inputs-before.json') -Encoding utf8
$startInfo = [Diagnostics.ProcessStartInfo]::new()
$startInfo.FileName = $NuExe
$startInfo.UseShellExecute = $false
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$startInfo.CreateNoWindow = $true
foreach ($arg in @('-n','--plugins',(Join-Path $OutputDirectory 'bin\nu_plugin_r3viewer.exe'),(Join-Path $OutputDirectory 'tests\test-nushell-integration.nu'),$configPath)) { $startInfo.ArgumentList.Add($arg) }
$process = [Diagnostics.Process]::new()
$process.StartInfo = $startInfo
$timedOut = $false
$code = $null
$failure = $null
try {
    if (-not $process.Start()) { throw 'Nu process did not start' }
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        $timedOut = $true
        $process.Kill($true) # Only this owned process and its descendants.
        if (-not $process.WaitForExit(5000)) { throw 'Timed-out Nu did not exit after tree cleanup' }
    }
    $code = $process.ExitCode
    if (-not $stdout.Wait(5000) -or -not $stderr.Wait(5000)) { throw 'Output did not reach EOF after process exit' }
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'integration.stdout.log'), $stdout.Result)
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'integration.stderr.log'), $stderr.Result)
    Write-Output $stdout.Result
    if ($stderr.Result) { Write-Output $stderr.Result }
} catch { $failure = $_.Exception.Message }
finally {
    @{exit_code=$code;timed_out=$timedOut;failure=$failure;nushell=$version} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json') -Encoding utf8
    $process.Dispose()
}
if ($timedOut -or $failure -or $code -ne 0) { throw "Integration failed; evidence: $OutputDirectory" }
Write-Output "Integration passed; evidence: $OutputDirectory"
