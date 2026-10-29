#requires -Version 7.0
# RE-BUFF-DIRECT-TEST01 Stage 2 confined runner.
# Builds the prepared project offline and executes only the authored DLL Main.
# No dotnet run/test/clean, no package sources, no network, no repo edits,
# no cleanup and no overwrite. All writable environment paths are under scratch.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-z0-9][a-z0-9-]{0,31}$')]
    [string] $RunId,
    [string] $RepositoryRoot = (Join-Path $PSScriptRoot '../../..')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)

$WorkspaceRoot = [IO.Path]::GetFullPath($RepositoryRoot)
$GithubRoot    = Join-Path $WorkspaceRoot 'github_src'
$HarnessDir    = Join-Path $GithubRoot 'tests\managed-recovered\buff-attribute'
$TaskRoot      = Join-Path $GithubRoot 'tools\re\local-recovery\buff-direct-test01'
$Scratch       = Join-Path $TaskRoot 'scratch'
$ProjectDir    = Join-Path $Scratch 'project'
$Csproj        = Join-Path $ProjectDir 'BuffDirectTest01.csproj'
$Fixture       = Join-Path $ProjectDir 'RecoveredFixture.cs'
$PrepareJson   = Join-Path $Scratch 'provenance\prepare-complete.json'
$RefPackDest   = Join-Path $Scratch 'nuget\packages\microsoft.netcore.app.ref\8.0.25'
$RefPackManifest = Join-Path $Scratch 'provenance\refpack-manifest.tsv'
$NuGetConfig   = Join-Path $Scratch 'config\NuGet.Config'
$Nohooks       = Join-Path $Scratch 'nohooks'
$Packages      = Join-Path $Scratch 'nuget\packages'
$ObjDir        = Join-Path $ProjectDir 'obj'
$BinDir        = Join-Path $ProjectDir 'bin'
$RunDir        = Join-Path $Scratch 'run'
$DllPath       = Join-Path $BinDir 'BuffDirectTest01.dll'
$ReceiptDir    = Join-Path $Scratch "receipts\$RunId"
$Verifier      = Join-Path $GithubRoot 'tools\re\verify-preservation.ps1'
$PwshExe       = Join-Path $PSHOME 'pwsh.exe'
$Dotnet        = 'C:\Program Files\dotnet\dotnet.exe'
$TimeoutMs     = 900000

function Assert-NoReparseChain([string] $Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while ($true) {
        if (-not [IO.File]::Exists($current) -and -not [IO.Directory]::Exists($current)) {
            throw "path component does not exist: $current"
        }
        $attributes = [IO.File]::GetAttributes($current)
        if (($attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "reparse point rejected: $current"
        }
        $parent = [IO.Path]::GetDirectoryName($current)
        if ([string]::IsNullOrEmpty($parent) -or $parent -eq $current) { break }
        $current = $parent
    }
}

function Write-NewFile([string] $Path, [byte[]] $Bytes) {
    if (Test-Path -LiteralPath $Path) { throw "REFUSING OVERWRITE: $Path" }
    $fs = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $fs.Write($Bytes, 0, $Bytes.Length)
        $fs.Flush($true)
    }
    finally {
        $fs.Dispose()
    }
}

function Write-NewText([string] $Path, [string[]] $Lines) {
    $text = ($Lines -join "`n") + "`n"
    Write-NewFile $Path ([Text.Encoding]::UTF8.GetBytes($text))
}

function Write-NewJson([string] $Path, [object] $Value) {
    $text = (ConvertTo-Json -InputObject $Value -Depth 20) + "`n"
    Write-NewFile $Path ([Text.Encoding]::UTF8.GetBytes($text))
}

function Get-ShaBytes([byte[]] $Bytes) {
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Test-ByteRangeEquals([byte[]] $Haystack, [int] $Offset, [byte[]] $Needle) {
    if ($Offset -lt 0 -or ($Offset + $Needle.Length) -gt $Haystack.Length) { return $false }
    for ($i = 0; $i -lt $Needle.Length; $i++) {
        if ($Haystack[$Offset + $i] -ne $Needle[$i]) { return $false }
    }
    return $true
}

function Get-TextLines([byte[]] $Bytes) {
    return ([Text.Encoding]::UTF8.GetString($Bytes) -split "`r?`n")
}

function Invoke-CapturedProcess([string] $Exe, [string[]] $Arguments, [string] $WorkingDirectory, [object] $EnvironmentMap) {
    $psi = [Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Exe
    foreach ($arg in $Arguments) { $psi.ArgumentList.Add($arg) }
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    if ($null -ne $EnvironmentMap) {
        $psi.Environment.Clear()
        foreach ($key in $EnvironmentMap.Keys) { $psi.Environment[$key] = $EnvironmentMap[$key] }
    }

    $stdout = [IO.MemoryStream]::new()
    $stderr = [IO.MemoryStream]::new()
    $proc = [Diagnostics.Process]::new()
    $proc.StartInfo = $psi
    try {
        [void]$proc.Start()
        $tout = $proc.StandardOutput.BaseStream.CopyToAsync($stdout)
        $terr = $proc.StandardError.BaseStream.CopyToAsync($stderr)
        $timedOut = -not $proc.WaitForExit($TimeoutMs)
        if ($timedOut) {
            try { $proc.Kill($true) } catch { }
            [void]$proc.WaitForExit()
        }
        [void]$tout.GetAwaiter().GetResult()
        [void]$terr.GetAwaiter().GetResult()
        $exitCode = if ($timedOut) { -999 } else { $proc.ExitCode }
        return [pscustomobject]@{
            ExitCode = $exitCode
            TimedOut = $timedOut
            Stdout   = $stdout.ToArray()
            Stderr   = $stderr.ToArray()
        }
    }
    finally {
        $proc.Dispose()
        $stdout.Dispose()
        $stderr.Dispose()
    }
}

function Invoke-Verifier([string] $Name) {
    $result = Invoke-CapturedProcess $PwshExe @(
        '-NoProfile',
        '-File', $Verifier,
        '-RepositoryRoot', $GithubRoot
    ) $GithubRoot $null
    Write-NewFile (Join-Path $ReceiptDir "$Name.stdout.bin") $result.Stdout
    Write-NewFile (Join-Path $ReceiptDir "$Name.stderr.bin") $result.Stderr
    Write-NewText (Join-Path $ReceiptDir "$Name.meta.txt") @(
        "name=$Name",
        "exit_code=$($result.ExitCode)",
        "timed_out=$($result.TimedOut)",
        "stdout_bytes=$($result.Stdout.Length)",
        "stdout_sha256=$(Get-ShaBytes $result.Stdout)",
        "stderr_bytes=$($result.Stderr.Length)",
        "stderr_sha256=$(Get-ShaBytes $result.Stderr)"
    )
    if ($result.TimedOut) { throw "verifier $Name timed out after receipts were written" }
    $text = [Text.Encoding]::UTF8.GetString($result.Stdout)
    if ([string]::IsNullOrWhiteSpace($text)) { throw "verifier $Name produced empty stdout" }
    $json = ConvertFrom-Json -InputObject $text -ErrorAction Stop
    if ($result.ExitCode -ne 0) { throw "verifier $Name exited $($result.ExitCode)" }
    if ($json.status -ne 'PASS') { throw "verifier $Name status is $($json.status)" }
    return [pscustomobject]@{ Result = $result; Json = $json }
}

function Assert-FixtureIntegrity([object] $Prepare) {
    if (-not (Test-Path -LiteralPath $Fixture)) { throw "fixture missing: $Fixture" }
    $fixtureBytes = [IO.File]::ReadAllBytes($Fixture)
    $fixtureSha = Get-ShaBytes $fixtureBytes
    if ($fixtureBytes.Length -ne [int64]$Prepare.fixture.bytes) { throw 'fixture size drift' }
    if ($fixtureSha -ne [string]$Prepare.fixture.sha256) { throw 'fixture sha drift' }

    $sourceMap = @{}
    foreach ($source in $Prepare.sources) {
        $full = Join-Path $WorkspaceRoot ($source.path -replace '/', '\')
        if (-not (Test-Path -LiteralPath $full)) { throw "source missing: $($source.path)" }
        Assert-NoReparseChain $full
        $bytes = [IO.File]::ReadAllBytes($full)
        if ($bytes.Length -ne [int64]$source.bytes) { throw "source size drift: $($source.role)" }
        if ((Get-ShaBytes $bytes) -ne [string]$source.sha256) { throw "source sha drift: $($source.role)" }
        $sourceMap[[string]$source.role] = $bytes
    }

    foreach ($method in $Prepare.methods) {
        $sourceBytes = $sourceMap[[string]$method.role]
        $offset = [int]$method.source_offset
        $length = [int]$method.source_bytes
        if ($offset -lt 0 -or ($offset + $length) -gt $sourceBytes.Length) { throw "source range out of bounds: $($method.key)" }
        $slice = [byte[]]::new($length)
        [Array]::Copy($sourceBytes, $offset, $slice, 0, $length)
        if ((Get-ShaBytes $slice) -ne [string]$method.source_sha256) { throw "source slice sha drift: $($method.key)" }
        if ([int]$method.fixture_length -ne $length) { throw "fixture length drift: $($method.key)" }
        if (-not (Test-ByteRangeEquals $fixtureBytes ([int]$method.fixture_offset) $slice)) {
            throw "fixture subsequence drift: $($method.key)"
        }
        $fixtureSlice = [byte[]]::new($length)
        [Array]::Copy($fixtureBytes, [int]$method.fixture_offset, $fixtureSlice, 0, $length)
        if ((Get-ShaBytes $fixtureSlice) -ne [string]$method.fixture_sha256) { throw "fixture slice sha drift: $($method.key)" }
    }
    return $fixtureSha
}

function Assert-RefPackManifest([object] $Prepare) {
    if (-not (Test-Path -LiteralPath $RefPackManifest)) { throw "ref pack manifest missing: $RefPackManifest" }
    $manifestBytes = [IO.File]::ReadAllBytes($RefPackManifest)
    $manifestSha = Get-ShaBytes $manifestBytes
    if ($manifestSha -ne [string]$Prepare.refpack.manifest_sha256) { throw 'ref pack manifest sha drift' }
    $lines = Get-TextLines $manifestBytes | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    $count = 0
    $total = [int64]0
    foreach ($line in $lines) {
        $parts = $line -split "`t"
        if ($parts.Count -ne 3) { throw "malformed ref pack manifest row: $line" }
        $relative = $parts[0]
        $expectedBytes = [int64]$parts[1]
        $expectedSha = $parts[2]
        $full = Join-Path $RefPackDest ($relative -replace '/', '\')
        if (-not (Test-Path -LiteralPath $full)) { throw "ref pack file missing: $relative" }
        Assert-NoReparseChain $full
        $bytes = [IO.File]::ReadAllBytes($full)
        if ($bytes.Length -ne $expectedBytes) { throw "ref pack size drift: $relative" }
        if ((Get-ShaBytes $bytes) -ne $expectedSha) { throw "ref pack sha drift: $relative" }
        $count++
        $total += $bytes.Length
    }
    if ($count -ne [int]$Prepare.refpack.files) { throw "ref pack file count drift: $count" }
    if ($total -ne [int64]$Prepare.refpack.total_bytes) { throw "ref pack byte total drift: $total" }
    return [pscustomobject]@{ Count = $count; Bytes = $total; Sha256 = $manifestSha }
}

function Assert-HarnessCopyIntegrity {
    $pairs = @(
        @{ name = 'TestDoubles.cs'; persistent = (Join-Path $HarnessDir 'TestDoubles.cs'); scratch = (Join-Path $ProjectDir 'TestDoubles.cs') },
        @{ name = 'HarnessTests.cs'; persistent = (Join-Path $HarnessDir 'HarnessTests.cs'); scratch = (Join-Path $ProjectDir 'HarnessTests.cs') }
    )
    $rows = [Collections.Generic.List[object]]::new()
    foreach ($pair in $pairs) {
        foreach ($path in @($pair.persistent, $pair.scratch)) {
            if (-not (Test-Path -LiteralPath $path)) { throw "harness copy missing: $path" }
            Assert-NoReparseChain $path
        }
        $persistentBytes = [IO.File]::ReadAllBytes($pair.persistent)
        $scratchBytes = [IO.File]::ReadAllBytes($pair.scratch)
        $persistentSha = Get-ShaBytes $persistentBytes
        $scratchSha = Get-ShaBytes $scratchBytes
        if ($persistentBytes.Length -ne $scratchBytes.Length -or $persistentSha -ne $scratchSha) {
            throw "scratch harness copy drift: $($pair.name) persistent=$persistentSha scratch=$scratchSha"
        }
        [void]$rows.Add([ordered]@{
            name = $pair.name
            persistent_path = $pair.persistent
            scratch_path = $pair.scratch
            bytes = [int64]$persistentBytes.Length
            sha256 = $persistentSha
            match = $true
        })
    }
    return @($rows)
}

function Get-CompiledAssemblyEntryPoint([string] $Path) {
    if (-not $Path.StartsWith($Scratch + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw "assembly inspection outside scratch rejected: $Path"
    }
    if (-not (Test-Path -LiteralPath $Path)) { throw "compiled assembly missing: $Path" }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        $pe = [System.Reflection.PortableExecutable.PEReader]::new($stream)
        try {
            if (-not $pe.HasMetadata) { throw 'compiled assembly has no CLI metadata' }
            $reader = [System.Reflection.Metadata.PEReaderExtensions]::GetMetadataReader($pe)
            $assemblyDefinition = $reader.GetAssemblyDefinition()
            $assemblyName = $reader.GetString($assemblyDefinition.Name)
            $corHeader = $pe.PEHeaders.CorHeader
            if ($null -eq $corHeader) { throw 'compiled assembly has no CLI header' }
            $token = $corHeader.EntryPointTokenOrRelativeVirtualAddress
            if ($token -eq 0) { throw 'compiled assembly has no CLI entrypoint' }
            if (($token -band 0xFF000000) -ne 0x06000000) {
                throw ("CLI entrypoint is not a MethodDef token: 0x{0:x8}" -f $token)
            }
            $row = $token -band 0x00FFFFFF
            $handle = [System.Reflection.Metadata.Ecma335.MetadataTokens]::MethodDefinitionHandle($row)
            $method = $reader.GetMethodDefinition($handle)
            $methodName = $reader.GetString($method.Name)
            $declaringType = $reader.GetTypeDefinition($method.GetDeclaringType())
            $typeName = $reader.GetString($declaringType.Name)
            $typeNamespace = $reader.GetString($declaringType.Namespace)
            $displayType = if ([string]::IsNullOrEmpty($typeNamespace)) { $typeName } else { "$typeNamespace.$typeName" }
            $isStatic = (($method.Attributes -band [System.Reflection.MethodAttributes]::Static) -ne 0)
            return [pscustomobject]@{
                AssemblyName = $assemblyName
                EntryPointToken = ('0x{0:x8}' -f $token)
                EntryPointMethod = $methodName
                EntryPointType = $displayType
                EntryPointDisplay = "$displayType.$methodName"
                IsStatic = $isStatic
            }
        }
        finally {
            $pe.Dispose()
        }
    }
    finally {
        $stream.Dispose()
    }
}

# ---------------------------------------------------------------------------
# Preconditions (no writes until the receipt directory is created).
# ---------------------------------------------------------------------------
if (-not (Test-Path -LiteralPath $WorkspaceRoot)) { throw "workspace missing: $WorkspaceRoot" }
Assert-NoReparseChain $WorkspaceRoot
foreach ($p in @($GithubRoot, $HarnessDir, $Scratch, $ProjectDir, $Csproj, $Fixture, $PrepareJson,
                 $RefPackDest, $RefPackManifest, $NuGetConfig, $Nohooks, $Packages, $RunDir,
                 $Verifier, $PwshExe, $Dotnet)) {
    if (-not (Test-Path -LiteralPath $p)) { throw "precondition missing: $p" }
    Assert-NoReparseChain $p
}
if (Test-Path -LiteralPath $ReceiptDir) { throw "refusing overwrite: receipt dir exists: $ReceiptDir" }
if (-not $ReceiptDir.StartsWith($Scratch + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw "receipt dir outside scratch: $ReceiptDir"
}
New-Item -ItemType Directory -Path $ReceiptDir -ErrorAction Stop | Out-Null

$prepare = ConvertFrom-Json -InputObject ([Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($PrepareJson))) -ErrorAction Stop
if ($prepare.status -ne 'PREPARED_NOT_COMPILED') { throw "prepare status is $($prepare.status)" }
if ($prepare.dotnet_invoked -ne $false) { throw 'prepare manifest reports dotnet invocation' }
if ($prepare.compiler_invoked -ne $false) { throw 'prepare manifest reports compiler invocation' }
if ($prepare.methods.Count -ne 10) { throw "prepare method count is $($prepare.methods.Count)" }

$fixtureShaBefore = Assert-FixtureIntegrity $prepare
$refpack = Assert-RefPackManifest $prepare

$preservationPreparePre = Join-Path $Scratch 'logs\preservation-pre.stdout.bin'
if (-not (Test-Path -LiteralPath $preservationPreparePre)) { throw "prepare pre-verifier log missing: $preservationPreparePre" }
$preparePreBytes = [IO.File]::ReadAllBytes($preservationPreparePre)

$preservationPre = Invoke-Verifier 'preservation-pre'
if ($preservationPre.Result.Stdout.Length -ne $preparePreBytes.Length -or
    -not (Test-ByteRangeEquals $preparePreBytes 0 $preservationPre.Result.Stdout)) {
    throw 'preservation stdout differs from prepare pre-verifier log'
}

$harnessCopies = Assert-HarnessCopyIntegrity
$harnessCopyLines = @(
    'status=MATCH',
    'note=scratch TestDoubles.cs/HarnessTests.cs are byte-equal to persistent audited copies; neither is recovered core'
)
foreach ($row in $harnessCopies) {
    $harnessCopyLines += "$($row.name)`tbytes=$($row.bytes)`tsha256=$($row.sha256)`tpersistent=$($row.persistent_path)`tscratch=$($row.scratch_path)"
}
Write-NewText (Join-Path $ReceiptDir 'prebuild-harness-copy.txt') $harnessCopyLines

# ---------------------------------------------------------------------------
# Construct child environment. All writable paths are under scratch.
# ---------------------------------------------------------------------------
$envMap = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($key in @('SystemRoot', 'SystemDrive', 'windir', 'ComSpec', 'PATHEXT', 'NUMBER_OF_PROCESSORS',
                   'PROCESSOR_ARCHITECTURE', 'PROCESSOR_IDENTIFIER', 'OS')) {
    $value = [Environment]::GetEnvironmentVariable($key, 'Process')
    if ($value) { $envMap[$key] = $value }
}
foreach ($key in @('ProgramFiles', 'ProgramFiles(x86)', 'ProgramW6432', 'ProgramData', 'ALLUSERSPROFILE',
                   'CommonProgramFiles', 'CommonProgramFiles(x86)', 'PUBLIC')) {
    $value = [Environment]::GetEnvironmentVariable($key, 'Process')
    if ($value) { $envMap[$key] = $value }
}
$envMap['PATH'] = 'C:\Program Files\dotnet;C:\Windows\System32;C:\Windows'
$envMap['TEMP'] = Join-Path $Scratch 'env\temp'
$envMap['TMP'] = Join-Path $Scratch 'env\tmp'
$envMap['HOME'] = Join-Path $Scratch 'env\home'
$envMap['USERPROFILE'] = Join-Path $Scratch 'env\userprofile'
$envMap['APPDATA'] = Join-Path $Scratch 'env\appdata'
$envMap['LOCALAPPDATA'] = Join-Path $Scratch 'env\localappdata'
$envMap['DOTNET_CLI_HOME'] = Join-Path $Scratch 'env\dotnet-cli-home'
$envMap['DOTNET_ROOT'] = 'C:\Program Files\dotnet'
$envMap['NUGET_PACKAGES'] = $Packages
$envMap['NUGET_HTTP_CACHE_PATH'] = Join-Path $Scratch 'env\nuget-http-cache'
$envMap['NUGET_PLUGINS_CACHE_PATH'] = Join-Path $Scratch 'env\nuget-plugins-cache'
$envMap['NUGET_SCRATCH'] = Join-Path $Scratch 'env\nuget-scratch'
$envMap['NUGET_XMLDOC_MODE'] = 'skip'
$envMap['MSBuildUserExtensionsPath'] = (Join-Path $Scratch 'env\msbuild-user-extensions') + '\'
$envMap['MSBuildProjectExtensionsPath'] = $ObjDir + '\'
$envMap['DOTNET_BUNDLE_EXTRACT_BASE_DIR'] = Join-Path $Scratch 'env\bundle'
$envMap['DOTNET_CLI_TELEMETRY_OPTOUT'] = '1'
$envMap['DOTNET_SKIP_FIRST_TIME_EXPERIENCE'] = '1'
$envMap['DOTNET_NOLOGO'] = '1'
$envMap['DOTNET_ADD_GLOBAL_TOOLS_TO_PATH'] = '0'
$envMap['DOTNET_GENERATE_ASPNET_CERTIFICATE'] = 'false'
$envMap['DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE'] = 'true'
$envMap['MSBuildEnableWorkloadResolver'] = 'false'
$envMap['DOTNET_EnableDiagnostics'] = '0'
$envMap['COMPlus_EnableDiagnostics'] = '0'
$envMap['DOTNET_MULTILEVEL_LOOKUP'] = '0'
$envMap['MSBUILDDISABLENODEREUSE'] = '1'
$envMap['DOTNET_CLI_DO_NOT_USE_MSBUILD_SERVER'] = '1'
$envMap['DOTNET_CLI_UI_LANGUAGE'] = 'en'

$envLines = @(
    "run_id=$RunId",
    "constructed_utc=$([DateTime]::UtcNow.ToString('o'))",
    "child_env_var_count=$($envMap.Count)"
)
foreach ($key in ($envMap.Keys | Sort-Object)) { $envLines += "$key=$($envMap[$key])" }
$envText = ($envLines -join "`n") + "`n"
Write-NewFile (Join-Path $ReceiptDir 'environment.txt') ([Text.Encoding]::UTF8.GetBytes($envText))
Write-NewText (Join-Path $ReceiptDir 'environment.sha256.txt') @(
    "environment_sha256=$(Get-ShaBytes ([Text.Encoding]::UTF8.GetBytes($envText)))",
    "note=child environment constructed from scratch; inherited DOTNET/MSBUILD/NUGET/COR/COMPLUS variables are not passed"
)

# ---------------------------------------------------------------------------
# Stage runner.
# ---------------------------------------------------------------------------
$common = @(
    '-p:ImportDirectoryBuildProps=false',
    '-p:ImportDirectoryBuildTargets=false',
    '-p:ImportDirectoryPackagesProps=false',
    "-p:DirectoryBuildPropsPath=$(Join-Path $Nohooks 'Directory.Build.props')",
    "-p:DirectoryBuildTargetsPath=$(Join-Path $Nohooks 'Directory.Build.targets')",
    "-p:CustomBeforeMicrosoftCommonProps=$(Join-Path $Nohooks 'no-before.props')",
    "-p:CustomAfterMicrosoftCommonProps=$(Join-Path $Nohooks 'no-after.props')",
    "-p:CustomBeforeMicrosoftCommonTargets=$(Join-Path $Nohooks 'no-before.targets')",
    "-p:CustomAfterMicrosoftCommonTargets=$(Join-Path $Nohooks 'no-after.targets')",
    '-p:ImportUserLocationsByWildcardBeforeMicrosoftCommonProps=false',
    '-p:ImportUserLocationsByWildcardAfterMicrosoftCommonProps=false',
    '-p:ImportUserLocationsByWildcardBeforeMicrosoftCommonTargets=false',
    '-p:ImportUserLocationsByWildcardAfterMicrosoftCommonTargets=false',
    '-p:UseSharedCompilation=false',
    '-p:BuildInParallel=false',
    '-p:UseAppHost=false',
    '-p:SelfContained=false',
    '-p:DisableTransitiveFrameworkReferenceDownloads=true',
    '-p:UseArtifactsOutput=false',
    '-p:RunAnalyzers=false',
    '-p:EnableNETAnalyzers=false',
    '-p:RestoreDisableParallel=true',
    '-p:NuGetAudit=false',
    '-p:RestoreNoHttpCache=true',
    "-p:RestoreConfigFile=$NuGetConfig",
    "-p:RestorePackagesPath=$Packages",
    "-p:BaseIntermediateOutputPath=$ObjDir\",
    "-p:MSBuildProjectExtensionsPath=$ObjDir\",
    "-p:OutputPath=$BinDir\",
    '-p:AppendTargetFrameworkToOutputPath=false',
    '-p:AppendRuntimeIdentifierToOutputPath=false',
    '-p:Deterministic=true'
)

function Invoke-Stage([string] $Name, [string[]] $Arguments, [string] $WorkingDirectory, [object] $EnvironmentMap) {
    $result = Invoke-CapturedProcess $Dotnet $Arguments $WorkingDirectory $EnvironmentMap
    Write-NewFile (Join-Path $ReceiptDir "$Name.stdout.bin") $result.Stdout
    Write-NewFile (Join-Path $ReceiptDir "$Name.stderr.bin") $result.Stderr
    Write-NewText (Join-Path $ReceiptDir "$Name.meta.txt") @(
        "stage=$Name",
        "exe=$Dotnet",
        "cwd=$WorkingDirectory",
        "argv=$(ConvertTo-Json -InputObject $Arguments -Compress)",
        "exit_code=$($result.ExitCode)",
        "timed_out=$($result.TimedOut)",
        "stdout_bytes=$($result.Stdout.Length)",
        "stdout_sha256=$(Get-ShaBytes $result.Stdout)",
        "stderr_bytes=$($result.Stderr.Length)",
        "stderr_sha256=$(Get-ShaBytes $result.Stderr)"
    )
    Write-Host ("STAGE {0} exit={1} stdout={2}B stderr={3}B" -f $Name, $result.ExitCode, $result.Stdout.Length, $result.Stderr.Length)
    return $result
}

try {
    $version = Invoke-Stage '01-dotnet-version' @('--version') $ProjectDir $envMap
    if ($version.ExitCode -ne 0) { throw 'dotnet --version failed' }
    $versionText = ([Text.Encoding]::UTF8.GetString($version.Stdout)).Trim()
    if ($versionText -ne '9.0.312') { throw "unexpected SDK version: $versionText" }

    $sdks = Invoke-Stage '02-dotnet-list-sdks' @('--list-sdks') $ProjectDir $envMap
    if ($sdks.ExitCode -ne 0) { throw 'dotnet --list-sdks failed' }
    $sdksText = [Text.Encoding]::UTF8.GetString($sdks.Stdout)
    if ($sdksText -notmatch '9\.0\.312') { throw 'SDK 9.0.312 not listed' }

    $runtimes = Invoke-Stage '03-dotnet-list-runtimes' @('--list-runtimes') $ProjectDir $envMap
    if ($runtimes.ExitCode -ne 0) { throw 'dotnet --list-runtimes failed' }
    $runtimesText = [Text.Encoding]::UTF8.GetString($runtimes.Stdout)
    if ($runtimesText -notmatch 'Microsoft\.NETCore\.App 8\.0\.11 ') { throw 'runtime 8.0.11 not listed' }

    $restore = Invoke-Stage '04-restore' (@('restore', $Csproj, '--configfile', $NuGetConfig, '--packages', $Packages, '--disable-parallel', '-v:minimal') + $common) $ProjectDir $envMap
    if ($restore.ExitCode -ne 0) { throw "restore failed: $($restore.ExitCode)" }

    $build = Invoke-Stage '05-build' (@('build', $Csproj, '--no-restore', '--nologo', '-v:minimal', '-m:1', '-nodeReuse:false') + $common) $ProjectDir $envMap
    if ($build.ExitCode -ne 0) { throw "build failed: $($build.ExitCode)" }

    if (-not (Test-Path -LiteralPath $DllPath)) { throw "compiled DLL missing: $DllPath" }
    $dllBytes = [IO.File]::ReadAllBytes($DllPath)
    Write-NewText (Join-Path $ReceiptDir '06-target-identity.txt') @(
        "dll=$DllPath",
        "bytes=$($dllBytes.Length)",
        "sha256=$(Get-ShaBytes $dllBytes)"
    )

    try {
        $assemblyInfo = Get-CompiledAssemblyEntryPoint $DllPath
        Write-NewText (Join-Path $ReceiptDir '06-assembly-inspect.txt') @(
            "status=PASS",
            "assembly_name=$($assemblyInfo.AssemblyName)",
            "entry_point=$($assemblyInfo.EntryPointDisplay)",
            "entry_point_method=$($assemblyInfo.EntryPointMethod)",
            "entry_point_type=$($assemblyInfo.EntryPointType)",
            "entry_point_static=$($assemblyInfo.IsStatic)",
            "entry_point_token=$($assemblyInfo.EntryPointToken)"
        )
    }
    catch {
        Write-NewText (Join-Path $ReceiptDir '06-assembly-inspect.blocker.txt') @(
            "status=BLOCKED",
            "reason=$($_.Exception.Message)"
        )
        throw "compiled assembly inspection blocked: $($_.Exception.Message)"
    }

    if ($assemblyInfo.AssemblyName -ne 'BuffDirectTest01') { throw "unexpected assembly name: $($assemblyInfo.AssemblyName)" }
    if ($assemblyInfo.EntryPointDisplay -ne 'Program.Main') { throw "unexpected entrypoint: $($assemblyInfo.EntryPointDisplay)" }
    if (-not $assemblyInfo.IsStatic) { throw 'entrypoint is not static' }
    if ($assemblyInfo.EntryPointToken -notmatch '^0x060[0-9a-f]{5}$') { throw "unexpected entrypoint token: $($assemblyInfo.EntryPointToken)" }

    $execute = Invoke-Stage '07-execute' @($DllPath) $RunDir $envMap
    $testText = [Text.Encoding]::UTF8.GetString($execute.Stdout)
    Write-NewFile (Join-Path $ReceiptDir '07-execute.test-stdout.txt') $execute.Stdout

    $testsTotal = [regex]::Match($testText, '(?m)^TESTS_TOTAL=([0-9]+)\s*$')
    $testsPassed = [regex]::Match($testText, '(?m)^TESTS_PASSED=([0-9]+)\s*$')
    $testsFailed = [regex]::Match($testText, '(?m)^TESTS_FAILED=([0-9]+)\s*$')
    $assertions = [regex]::Match($testText, '(?m)^ASSERTIONS_TOTAL=([0-9]+)\s*$')
    $synthetic = [regex]::Match($testText, '(?m)^SYNTHETIC_MODEL_TESTS=([0-9]+)\s*$')
    $resultLine = [regex]::Match($testText, '(?m)^RESULT=(PASS|FAIL)\s*$')

    if ($execute.ExitCode -ne 0) { throw "test DLL exited $($execute.ExitCode)" }
    if (-not $resultLine.Success -or $resultLine.Groups[1].Value -ne 'PASS') { throw 'test RESULT is not PASS' }
    if (-not $testsTotal.Success -or [int]$testsTotal.Groups[1].Value -ne 28) { throw 'TESTS_TOTAL is not 28' }
    if (-not $testsPassed.Success -or [int]$testsPassed.Groups[1].Value -ne 28) { throw 'TESTS_PASSED is not 28' }
    if (-not $testsFailed.Success -or [int]$testsFailed.Groups[1].Value -ne 0) { throw 'TESTS_FAILED is not 0' }
    if (-not $assertions.Success -or [int]$assertions.Groups[1].Value -le 0) { throw 'ASSERTIONS_TOTAL is not positive' }
    if (-not $synthetic.Success -or [int]$synthetic.Groups[1].Value -ne 0) { throw 'SYNTHETIC_MODEL_TESTS is not 0' }

    Write-NewText (Join-Path $ReceiptDir '07-execute.test-summary.txt') @(
        "tests_total=$($testsTotal.Groups[1].Value)",
        "tests_passed=$($testsPassed.Groups[1].Value)",
        "tests_failed=$($testsFailed.Groups[1].Value)",
        "assertions_total=$($assertions.Groups[1].Value)",
        "synthetic_model_tests=$($synthetic.Groups[1].Value)",
        "result=$($resultLine.Groups[1].Value)"
    )

    $fixtureShaAfter = Assert-FixtureIntegrity $prepare
    if ($fixtureShaAfter -ne $fixtureShaBefore) { throw 'fixture changed during build/execute' }

    $preservationPost = Invoke-Verifier 'preservation-post'
    if ($preservationPost.Result.Stdout.Length -ne $preservationPre.Result.Stdout.Length -or
        -not (Test-ByteRangeEquals $preservationPre.Result.Stdout 0 $preservationPost.Result.Stdout)) {
        throw 'preservation stdout changed during build/execute'
    }

    $complete = [ordered]@{
        schema = 're-buff-direct-test01-run-v1'
        task = 'RE-BUFF-DIRECT-TEST01'
        stage = 2
        status = 'PASS'
        run_id = $RunId
        workspace_root = $WorkspaceRoot
        scratch = $Scratch
        prepare_manifest = $PrepareJson
        fixture_sha256_before = $fixtureShaBefore
        fixture_sha256_after = $fixtureShaAfter
        refpack_files = $refpack.Count
        refpack_bytes = $refpack.Bytes
        refpack_manifest_sha256 = $refpack.Sha256
        sdk_version = $versionText
        dll = $DllPath
        dll_bytes = [int64]$dllBytes.Length
        dll_sha256 = Get-ShaBytes $dllBytes
        assembly_inspect_status = 'PASS'
        assembly_name = $assemblyInfo.AssemblyName
        entry_point = $assemblyInfo.EntryPointDisplay
        entry_point_method = $assemblyInfo.EntryPointMethod
        entry_point_token = $assemblyInfo.EntryPointToken
        entry_point_static = $assemblyInfo.IsStatic
        execute_exit_code = $execute.ExitCode
        tests_total = [int]$testsTotal.Groups[1].Value
        tests_passed = [int]$testsPassed.Groups[1].Value
        tests_failed = [int]$testsFailed.Groups[1].Value
        assertions_total = [int]$assertions.Groups[1].Value
        synthetic_model_tests = [int]$synthetic.Groups[1].Value
        result = $resultLine.Groups[1].Value
        preservation_pre_status = $preservationPre.Json.status
        preservation_post_status = $preservationPost.Json.status
        preservation_stdout_equal = $true
        harness_copy_status = 'MATCH'
        harness_copies = @($harnessCopies)
        stages = @(
            '01-dotnet-version', '02-dotnet-list-sdks', '03-dotnet-list-runtimes',
            '04-restore', '05-build', '06-target-identity', '06-assembly-inspect', '07-execute'
        )
        limitations = @(
            'Recovered core under stubbed TEST_DOUBLE dependencies; not a runtime game claim.',
            'No dotnet run/test/clean, no package sources, no network, no repo edits, no cleanup.',
            'Preservation verifier is pinned-byte integrity only, not an OS sandbox.',
            'Compiled-assembly inspection is read-only PEReader metadata on the scratch test DLL only; no Add-Type and no game binary inspection.',
            'Timeout returns -999 after killing only the task-spawned child and preserving stdout/stderr receipts.'
        )
    }
    Write-NewJson (Join-Path $ReceiptDir 'run.completed.json') $complete
    Write-Host "RUN PASS run_id=$RunId tests=$($testsTotal.Groups[1].Value) assertions=$($assertions.Groups[1].Value)"
    Write-Host "receipt=$ReceiptDir"
    exit 0
}
catch {
    $message = $_.Exception.Message
    Write-NewText (Join-Path $ReceiptDir 'run.aborted.txt') @(
        "run_id=$RunId",
        "status=ABORTED",
        "reason=$message",
        "utc=$([DateTime]::UtcNow.ToString('o'))"
    )
    Write-Host "RUN ABORTED: $message"
    exit 2
}
