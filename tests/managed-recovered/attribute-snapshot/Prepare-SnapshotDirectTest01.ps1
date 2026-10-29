#requires -Version 7.0
# RE-SNAPSHOT-DIRECT-TEST01 Stage 1 preparation.
# Scope: create only the new private scratch tree, byte-slice exactly three
# recovered methods into a private fixture, copy the audited 8.0.25 ref pack,
# run the approved read-only preservation verifier pre/post, and record
# baselines/manifests. This script does NOT invoke dotnet, a compiler, tests,
# MCP, Git, or the network, and it does not delete or overwrite anything.
[CmdletBinding()]
param(
    [string] $RepositoryRoot = (Join-Path $PSScriptRoot '../../..')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)

$WorkspaceRoot  = [IO.Path]::GetFullPath($RepositoryRoot)
$GithubRoot     = Join-Path $WorkspaceRoot 'github_src'
$HarnessDir     = Join-Path $GithubRoot 'tests\managed-recovered\attribute-snapshot'
$TaskRoot       = Join-Path $GithubRoot 'tools\re\local-recovery\snapshot-direct-test01'
$Scratch        = Join-Path $TaskRoot 'scratch'
$ProjectDir     = Join-Path $Scratch 'project'
$Packages       = Join-Path $Scratch 'nuget\packages'
$RefPackSource  = 'REQUIRED_NET8_REFERENCE_PACK_DIRECTORY'
$RefPackDest    = Join-Path $Packages 'microsoft.netcore.app.ref\8.0.25'
$Verifier       = Join-Path $GithubRoot 'tools\re\verify-preservation.ps1'
$PriorRunner    = Join-Path $GithubRoot 'tools\re\local-recovery\combat-reference-verify01\scratch\runner\Run-Event101.ps1'
$PwshExe        = Join-Path $PSHOME 'pwsh.exe'
$DotnetExe      = 'C:\Program Files\dotnet\dotnet.exe'
$SdkVersionFile = 'C:\Program Files\dotnet\sdk\9.0.312\.version'
$RuntimeDir     = 'C:\Program Files\dotnet\shared\Microsoft.NETCore.App\8.0.11'
$TimeoutMs      = 900000

$ExpectedPersistentNames = @(
    'TestScaffolding.cs',
    'HarnessTests.cs',
    'Prepare-SnapshotDirectTest01.ps1',
    'Run-SnapshotDirectTest01.ps1',
    'README.md'
)

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

function Assert-Absent([string] $Path) {
    if ([IO.File]::Exists($Path) -or [IO.Directory]::Exists($Path)) {
        throw "REFUSING: path already exists: $Path"
    }
}

function New-ExclusiveDirectory([string] $Path) {
    Assert-Absent $Path
    $parent = [IO.Path]::GetDirectoryName($Path)
    if (-not [IO.Directory]::Exists($parent)) { throw "parent missing: $parent" }
    New-Item -ItemType Directory -Path $Path -ErrorAction Stop | Out-Null
    if (-not [IO.Directory]::Exists($Path)) { throw "directory creation failed: $Path" }
}

function Ensure-Directory([string] $Path) {
    if ([IO.Directory]::Exists($Path)) { return }
    if ([IO.File]::Exists($Path)) { throw "not a directory: $Path" }
    [void][IO.Directory]::CreateDirectory($Path)
}

function Write-NewFile([string] $Path, [byte[]] $Bytes) {
    if ([IO.File]::Exists($Path) -or [IO.Directory]::Exists($Path)) {
        throw "REFUSING OVERWRITE: path exists: $Path"
    }
    $parent = [IO.Path]::GetDirectoryName($Path)
    if (-not [IO.Directory]::Exists($parent)) { throw "parent missing: $parent" }
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
    $text = (($Lines -join "`n") -replace "`r`n", "`n") + "`n"
    Write-NewFile $Path ([Text.Encoding]::UTF8.GetBytes($text))
}

function Write-NewJson([string] $Path, [object] $Value) {
    $text = ((ConvertTo-Json -InputObject $Value -Depth 30) -replace "`r`n", "`n") + "`n"
    Write-NewFile $Path ([Text.Encoding]::UTF8.GetBytes($text))
}

function Get-ShaBytes([byte[]] $Bytes) {
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Get-Digest([string] $Path) {
    Assert-NoReparseChain $Path
    $bytes = [IO.File]::ReadAllBytes($Path)
    return [pscustomobject]@{
        bytes  = [int64]$bytes.Length
        sha256 = Get-ShaBytes $bytes
    }
}

function Get-RelativePath([string] $Path) {
    return ([IO.Path]::GetRelativePath($WorkspaceRoot, $Path)).Replace('\', '/')
}

function Test-ByteRangeEquals([byte[]] $Haystack, [int] $Offset, [byte[]] $Needle) {
    if ($Offset -lt 0 -or ($Offset + $Needle.Length) -gt $Haystack.Length) { return $false }
    for ($i = 0; $i -lt $Needle.Length; $i++) {
        if ($Haystack[$Offset + $i] -ne $Needle[$i]) { return $false }
    }
    return $true
}

function Test-BytesContain([byte[]] $Haystack, [byte[]] $Needle) {
    if ($Needle.Length -eq 0) { return $true }
    if ($Haystack.Length -lt $Needle.Length) { return $false }
    # Latin1 maps every byte 1:1 to a char, so this is an exact byte-substring
    # search without UTF-8 normalization or generated-code rewriting.
    $haystackText = [Text.Encoding]::Latin1.GetString($Haystack)
    $needleText = [Text.Encoding]::Latin1.GetString($Needle)
    return $haystackText.Contains($needleText, [StringComparison]::Ordinal)
}

function Invoke-CapturedProcess([string] $Exe, [string[]] $Arguments, [string] $WorkingDirectory) {
    $psi = [Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Exe
    foreach ($arg in $Arguments) { $psi.ArgumentList.Add($arg) }
    $psi.WorkingDirectory = $WorkingDirectory
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true

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

function Invoke-Preservation([string] $Suffix, [string] $LogDir) {
    $startedUtc = [DateTime]::UtcNow
    $result = Invoke-CapturedProcess $PwshExe @(
        '-NoProfile',
        '-File', $Verifier,
        '-RepositoryRoot', $GithubRoot
    ) $GithubRoot
    $endedUtc = [DateTime]::UtcNow

    Write-NewFile (Join-Path $LogDir "preservation-$Suffix.stdout.bin") $result.Stdout
    Write-NewFile (Join-Path $LogDir "preservation-$Suffix.stderr.bin") $result.Stderr
    Write-NewText (Join-Path $LogDir "preservation-$Suffix.meta.txt") @(
        "suffix=$Suffix",
        "started_utc=$($startedUtc.ToString('o'))",
        "ended_utc=$($endedUtc.ToString('o'))",
        "exe=$PwshExe",
        "argv=$(ConvertTo-Json -InputObject @('-NoProfile','-File',$Verifier,'-RepositoryRoot',$GithubRoot) -Compress)",
        "cwd=$GithubRoot",
        "exit_code=$($result.ExitCode)",
        "timed_out=$($result.TimedOut)",
        "stdout_bytes=$($result.Stdout.Length)",
        "stdout_sha256=$(Get-ShaBytes $result.Stdout)",
        "stderr_bytes=$($result.Stderr.Length)",
        "stderr_sha256=$(Get-ShaBytes $result.Stderr)"
    )

    if ($result.TimedOut) { throw "preservation verifier $Suffix timed out after receipts were written" }
    $text = [Text.Encoding]::UTF8.GetString($result.Stdout)
    if ([string]::IsNullOrWhiteSpace($text)) { throw "preservation verifier $Suffix produced empty stdout" }
    try {
        $json = ConvertFrom-Json -InputObject $text -ErrorAction Stop
    }
    catch {
        throw "preservation verifier $Suffix stdout is not valid JSON: $($_.Exception.Message)"
    }
    if ($result.ExitCode -ne 0) { throw "preservation verifier $Suffix exited $($result.ExitCode)" }
    if ($json.status -ne 'PASS') { throw "preservation verifier $Suffix status is $($json.status)" }
    return [pscustomobject]@{
        Result = $result
        Json   = $json
    }
}

# ---------------------------------------------------------------------------
# Preconditions
# ---------------------------------------------------------------------------
if (-not [IO.Directory]::Exists($WorkspaceRoot)) { throw "workspace missing: $WorkspaceRoot" }
Assert-NoReparseChain $WorkspaceRoot

$requiredExisting = @(
    $GithubRoot,
    (Join-Path $GithubRoot 'tests'),
    (Join-Path $GithubRoot 'tests\managed-recovered'),
    $HarnessDir,
    (Join-Path $GithubRoot 'tools'),
    (Join-Path $GithubRoot 'tools\re'),
    (Join-Path $GithubRoot 'tools\re\local-recovery'),
    (Join-Path $WorkspaceRoot 'AGENTS.md'),
    $Verifier,
    $PriorRunner,
    $PwshExe,
    $DotnetExe,
    $SdkVersionFile,
    $RuntimeDir,
    $RefPackSource
)
foreach ($p in $requiredExisting) {
    if (-not [IO.File]::Exists($p) -and -not [IO.Directory]::Exists($p)) { throw "precondition missing: $p" }
    Assert-NoReparseChain $p
}

Assert-Absent $TaskRoot
Assert-Absent $Scratch
Assert-Absent $RefPackDest

$persistentItems = @(Get-ChildItem -LiteralPath $HarnessDir -Force)
$ActualPersistentFiles = @($persistentItems | Where-Object { -not $_.PSIsContainer })
$persistentDirs = @($persistentItems | Where-Object { $_.PSIsContainer })
if ($persistentDirs.Count -ne 0) { throw "persistent root contains subdirectories: $($persistentDirs.Name -join ', ')" }
if ($ActualPersistentFiles.Count -ne $ExpectedPersistentNames.Count) {
    throw "persistent root file count is $($ActualPersistentFiles.Count), expected $($ExpectedPersistentNames.Count)"
}
foreach ($name in $ExpectedPersistentNames) {
    if (-not [IO.File]::Exists((Join-Path $HarnessDir $name))) { throw "persistent file missing: $name" }
}
foreach ($file in $ActualPersistentFiles) {
    if ($ExpectedPersistentNames -notcontains $file.Name) { throw "unexpected persistent file: $($file.Name)" }
    Assert-NoReparseChain $file.FullName
}

$sdkVersionLines = @(Get-Content -LiteralPath $SdkVersionFile)
if ($sdkVersionLines -notcontains '9.0.312') {
    throw "installed SDK 9.0.312 not verified from $SdkVersionFile"
}
$runtimeVersion = Split-Path -Leaf $RuntimeDir
if ($runtimeVersion -ne '8.0.11') { throw "unexpected runtime directory: $RuntimeDir" }

$refPackSourceFiles = @(Get-ChildItem -LiteralPath $RefPackSource -Recurse -File -Force)
if ($refPackSourceFiles.Count -ne 351) {
    throw "ref pack source file count is $($refPackSourceFiles.Count), expected 351"
}
$refPackSourceBytes = [int64]0
foreach ($file in $refPackSourceFiles) {
    Assert-NoReparseChain $file.FullName
    $refPackSourceBytes += $file.Length
}

# ---------------------------------------------------------------------------
# Exact source ledger and byte slicing
# ---------------------------------------------------------------------------
$sources = @{
    BAS = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8\source\_global\BaseAttributeComponent.cs'
        bytes = 35847
        sha256 = 'c2b14c11808966b4ee81301b631f52f9e7c07777bdd9e72758bfc81f686fb6b9'
    }
    AS = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\85c3ebfaabe8232a30fd17706a7bd5eaa8be2796a105dec883760ada6908624d\source\_global\AttributeSnapshot.cs'
        bytes = 372
        sha256 = '3e2cd1b7799363521318c126faaf27e4ff2e24278e72761f7243b6158b7dd5da'
    }
    CALC = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0\source\_global\Calculation.cs'
        bytes = 24688
        sha256 = 'daa9fff61d83566475f474ab84e2feea1b1b7de987a08a678cc0c87a32ea6e34'
    }
}

$methodSpecs = @(
    [pscustomobject]@{
        Key='TakeSnapshot'; Class='BaseAttributeComponent'; Role='BAS'; StartLine=412; EndLine=418;
        Bytes=268; Sha256='6d84953e8e51125da0a08610d1d59c8219068ee653fe06f07bdbd26df531f201';
        Td='0x02002e53'; Md='0x060184c5'; Rva=7113276; CodeSize=52;
        Status='FULL_SELECTED_CIL'; Evidence='evidence/re-gameplay-snapshot01/TakeSnapshot.il:21-49';
        Signature="`tpublic AttributeSnapshot TakeSnapshot()"
    },
    [pscustomobject]@{
        Key='GetCurrentValue'; Class='AttributeSnapshot'; Role='AS'; StartLine=14; EndLine=17;
        Bytes=96; Sha256='2e467b842558ac501d1d3f02ec879a969906a09fefe724fc24f991f57aed41f6';
        Td='0x02002e52'; Md='0x060184b2'; Rva=7111924; CodeSize=9;
        Status='HISTORICAL_SOURCE_CONTEXT_METADATA_BOUND'; Evidence='src/reconstructed/managed/semantic/gameplay/R38_METHOD_EVIDENCE_ADDENDUM.md:161-167; raw methods row 221 in src/reconstructed/managed/full/CSharpScript/shards/85c3ebfaabe8232a30fd17706a7bd5eaa8be2796a105dec883760ada6908624d/evidence/raw/methods.jsonl; raw type row 74';
        Signature="`tpublic float GetCurrentValue(EAttributeType attrId)"
    },
    [pscustomobject]@{
        Key='GetAttrFromSnapshots'; Class='Calculation'; Role='CALC'; StartLine=20; EndLine=28;
        Bytes=360; Sha256='28fb4ef1a2962b49c17b987a90b3990d62cbda76729772c53ad80faabbaf5abf';
        Td='0x02002eb2'; Md='0x06018865'; Rva=7210036; CodeSize=56;
        Status='FULL_CIL_SOURCE_FIDELITY_PASS'; Evidence='src/reconstructed/managed/semantic/gameplay/R38_METHOD_EVIDENCE_ADDENDUM.md:116,124-140; raw methods row 642; src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md row 289';
        Signature="`tprivate static double GetAttrFromSnapshots(SnapshotPayload snap, EAttrOwnerType attrOwnerType, EAttributeType attrId)"
    }
)

$sourceCache = @{}
foreach ($role in $sources.Keys) {
    $spec = $sources[$role]
    if (-not [IO.File]::Exists($spec.path)) { throw "source missing: $($spec.path)" }
    Assert-NoReparseChain $spec.path
    $bytes = [IO.File]::ReadAllBytes($spec.path)
    if ($bytes.Length -ne $spec.bytes) { throw "source size drift for $role" }
    if ((Get-ShaBytes $bytes) -ne $spec.sha256) { throw "source sha drift for $role" }
    $sourceCache[$role] = $bytes
}

$lineOffsetsCache = @{}
foreach ($role in $sources.Keys) {
    $bytes = $sourceCache[$role]
    $offsets = [Collections.Generic.List[int]]::new()
    $offsets.Add(0)
    for ($i = 0; $i -lt $bytes.Length; $i++) {
        if ($bytes[$i] -eq 10) { $offsets.Add($i + 1) }
    }
    $lineOffsetsCache[$role] = $offsets
}

$methodLedger = foreach ($m in $methodSpecs) {
    $sourceBytes = $sourceCache[$m.Role]
    $sourceSpec = $sources[$m.Role]
    $offsets = $lineOffsetsCache[$m.Role]
    if ($offsets.Count -lt $m.EndLine) { throw "not enough lines for $($m.Key)" }
    $start = $offsets[$m.StartLine - 1]
    $end = if ($m.EndLine -lt $offsets.Count) { $offsets[$m.EndLine] } else { $sourceBytes.Length }
    $length = $end - $start
    $slice = [byte[]]::new($length)
    [Array]::Copy($sourceBytes, $start, $slice, 0, $length)
    if ($length -ne $m.Bytes) { throw "slice size drift for $($m.Key): $length" }
    if ((Get-ShaBytes $slice) -ne $m.Sha256) { throw "slice sha drift for $($m.Key)" }

    $text = [Text.Encoding]::UTF8.GetString($slice)
    $lines = $text -split "`n"
    $firstLine = $lines[0].TrimEnd("`r")
    if ($firstLine -cne $m.Signature) { throw "start signature mismatch for $($m.Key): [$firstLine]" }
    $lastIndex = $lines.Count - 2
    if ($lastIndex -lt 0) { throw "slice has no closing line for $($m.Key)" }
    $lastLine = $lines[$lastIndex].TrimEnd("`r")
    if ($lastLine -cne "`t}") { throw "end brace mismatch for $($m.Key)" }

    $openBraces = 0
    $closeBraces = 0
    $crBytes = 0
    foreach ($b in $slice) {
        if ($b -eq 123) { $openBraces++ }
        elseif ($b -eq 125) { $closeBraces++ }
        elseif ($b -eq 13) { $crBytes++ }
    }
    if ($openBraces -ne $closeBraces) { throw "brace imbalance for $($m.Key): $openBraces/$closeBraces" }
    if ($slice[$length - 1] -ne 10) { throw "slice does not end with LF for $($m.Key)" }

    [pscustomobject]@{
        Key = $m.Key
        Class = $m.Class
        Role = $m.Role
        SourcePath = $sourceSpec.path
        SourceWholeBytes = [int64]$sourceSpec.bytes
        SourceWholeSha256 = $sourceSpec.sha256
        StartLine = $m.StartLine
        EndLine = $m.EndLine
        SourceOffset = $start
        SourceLength = $length
        SourceSha256 = $m.Sha256
        Td = $m.Td
        Md = $m.Md
        Rva = $m.Rva
        CodeSize = $m.CodeSize
        Status = $m.Status
        Evidence = $m.Evidence
        CrBytes = $crBytes
        Slice = $slice
    }
}

# ---------------------------------------------------------------------------
# Create the exclusive private scratch tree
# ---------------------------------------------------------------------------
New-ExclusiveDirectory $TaskRoot
New-ExclusiveDirectory $Scratch
foreach ($rel in @(
    'config',
    'project',
    'nuget',
    'nuget\packages',
    'nuget\packages\microsoft.netcore.app.ref',
    'nohooks',
    'env',
    'env\temp',
    'env\tmp',
    'env\home',
    'env\userprofile',
    'env\appdata',
    'env\localappdata',
    'env\dotnet-cli-home',
    'env\nuget-http-cache',
    'env\nuget-plugins-cache',
    'env\nuget-scratch',
    'env\msbuild-user-extensions',
    'env\bundle',
    'logs',
    'provenance',
    'receipts',
    'run'
)) {
    New-ExclusiveDirectory (Join-Path $Scratch $rel)
}

$logDir = Join-Path $Scratch 'logs'
$provenanceDir = Join-Path $Scratch 'provenance'

# ---------------------------------------------------------------------------
# Baseline anchors: source originals, map/evidence, old harness scripts and
# representative old receipts. Only selected anchors are hashed; this is not a
# whole-corpus index.
# ---------------------------------------------------------------------------
$anchorSpecs = @(
    @{ role = 'instructions.AGENTS'; path = 'AGENTS.md' },
    @{ role = 'source.TakeSnapshot'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/BaseAttributeComponent.cs' },
    @{ role = 'source.GetCurrentValue'; path = 'src/reconstructed/managed/full/CSharpScript/shards/85c3ebfaabe8232a30fd17706a7bd5eaa8be2796a105dec883760ada6908624d/source/_global/AttributeSnapshot.cs' },
    @{ role = 'source.GetAttrFromSnapshots'; path = 'src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/source/_global/Calculation.cs' },
    @{ role = 'context.SnapshotPayload'; path = 'src/reconstructed/managed/full/CSharpScript/shards/1bb927286cf97bdacde702c35cdca5d539da7c601d79a0ac3effd44db222faab/source/_global/SnapshotPayload.cs' },
    @{ role = 'context.EAttrOwnerType'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/EAttrOwnerType.cs' },
    @{ role = 'map.METHOD_EVIDENCE_MAP'; path = 'src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md' },
    @{ role = 'evidence.R38'; path = 'src/reconstructed/managed/semantic/gameplay/R38_METHOD_EVIDENCE_ADDENDUM.md' },
    @{ role = 'evidence.R53'; path = 'src/reconstructed/managed/semantic/gameplay/R53_R46_RETAINED_EVIDENCE.txt' },
    @{ role = 'evidence.TakeSnapshot.il'; path = 'evidence/re-gameplay-snapshot01/TakeSnapshot.il' },
    @{ role = 'raw.Calculation.methods'; path = 'src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/evidence/raw/methods.jsonl' },
    @{ role = 'raw.Calculation.types'; path = 'src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/evidence/raw/types.jsonl' },
    @{ role = 'raw.AttributeSnapshot.methods'; path = 'src/reconstructed/managed/full/CSharpScript/shards/85c3ebfaabe8232a30fd17706a7bd5eaa8be2796a105dec883760ada6908624d/evidence/raw/methods.jsonl' },
    @{ role = 'raw.AttributeSnapshot.types'; path = 'src/reconstructed/managed/full/CSharpScript/shards/85c3ebfaabe8232a30fd17706a7bd5eaa8be2796a105dec883760ada6908624d/evidence/raw/types.jsonl' },
    @{ role = 'old.harness.prepare'; path = 'github_src/tests/managed-recovered/buff-attribute/Prepare-BuffDirectTest01.ps1' },
    @{ role = 'old.harness.run'; path = 'github_src/tests/managed-recovered/buff-attribute/Run-BuffDirectTest01.ps1' },
    @{ role = 'old.harness.tests'; path = 'github_src/tests/managed-recovered/buff-attribute/HarnessTests.cs' },
    @{ role = 'old.harness.doubles'; path = 'github_src/tests/managed-recovered/buff-attribute/TestDoubles.cs' },
    @{ role = 'old.harness.readme'; path = 'github_src/tests/managed-recovered/buff-attribute/README.md' },
    @{ role = 'old.receipt.prepare'; path = 'github_src/tools/re/local-recovery/buff-direct-test01/scratch/provenance/prepare-complete.json' },
    @{ role = 'old.receipt.completed'; path = 'github_src/tools/re/local-recovery/buff-direct-test01/scratch/receipts/a1/run.completed.json' },
    @{ role = 'old.receipt.summary'; path = 'github_src/tools/re/local-recovery/buff-direct-test01/scratch/receipts/a1/07-execute.test-summary.txt' },
    @{ role = 'old.receipt.revision'; path = 'github_src/tools/re/local-recovery/buff-direct-test01/scratch/revisions/stage1-precompile-corrections01/REVISION.json' },
    @{ role = 'verifier'; path = 'github_src/tools/re/verify-preservation.ps1' },
    @{ role = 'prior.runner'; path = 'github_src/tools/re/local-recovery/combat-reference-verify01/scratch/runner/Run-Event101.ps1' }
)

function Get-Baseline {
    $rows = foreach ($spec in $anchorSpecs) {
        $full = Join-Path $WorkspaceRoot ($spec.path -replace '/', '\')
        if (-not [IO.File]::Exists($full)) { throw "anchor missing: $($spec.path)" }
        $digest = Get-Digest $full
        [ordered]@{
            role = $spec.role
            path = $spec.path
            bytes = $digest.bytes
            sha256 = $digest.sha256
        }
    }
    $sorted = @($rows | Sort-Object { $_.path })
    $anchorsText = ($sorted | ConvertTo-Json -Depth 8 -Compress)
    return [ordered]@{
        schema = 're-snapshot-direct-test01-baseline-v1'
        task = 'RE-SNAPSHOT-DIRECT-TEST01'
        stage = 1
        anchors = $sorted
        anchor_count = $sorted.Count
        anchors_sha256 = Get-ShaBytes ([Text.Encoding]::UTF8.GetBytes($anchorsText))
    }
}

$baselinePre = Get-Baseline
Write-NewJson (Join-Path $provenanceDir 'baseline-pre.json') $baselinePre

# Read-only preservation verifier, before fixture/project writes.
$preservationPre = Invoke-Preservation 'pre' $logDir

# ---------------------------------------------------------------------------
# Private fixture: authored class wrappers + exact byte slices only.
# ---------------------------------------------------------------------------
$fixtureHeaderLines = @(
    '// PRIVATE SCRATCH FIXTURE - RE-SNAPSHOT-DIRECT-TEST01.',
    '// Generated by Prepare-SnapshotDirectTest01.ps1 from exact saved-source byte slices.',
    '// Not persistent, not public. Do not edit.',
    '// Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db',
    '// MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3',
    'using System;',
    'using Aki.Protocol;',
    ''
)
$fixtureHeaderText = ($fixtureHeaderLines -join "`n") + "`n"

$fixtureStream = [IO.MemoryStream]::new()
try {
    $headerBytes = [Text.Encoding]::UTF8.GetBytes($fixtureHeaderText)
    $fixtureStream.Write($headerBytes, 0, $headerBytes.Length)
    $currentClass = $null
    foreach ($m in $methodLedger) {
        if ($m.Class -ne $currentClass) {
            if ($null -ne $currentClass) {
                $closeBytes = [Text.Encoding]::UTF8.GetBytes("}`n`n")
                $fixtureStream.Write($closeBytes, 0, $closeBytes.Length)
            }
            $classBytes = [Text.Encoding]::UTF8.GetBytes("public partial class $($m.Class)`n{`n")
            $fixtureStream.Write($classBytes, 0, $classBytes.Length)
            $currentClass = $m.Class
        }
        $m | Add-Member -NotePropertyName FixtureOffset -NotePropertyValue ([int]$fixtureStream.Length) -Force
        $m | Add-Member -NotePropertyName FixtureLength -NotePropertyValue ([int]$m.Slice.Length) -Force
        $fixtureStream.Write($m.Slice, 0, $m.Slice.Length)
    }
    if ($null -ne $currentClass) {
        $closeBytes = [Text.Encoding]::UTF8.GetBytes("}`n")
        $fixtureStream.Write($closeBytes, 0, $closeBytes.Length)
    }
    $fixtureBytes = $fixtureStream.ToArray()
}
finally {
    $fixtureStream.Dispose()
}

$fixturePath = Join-Path $ProjectDir 'RecoveredFixture.cs'
Write-NewFile $fixturePath $fixtureBytes
$fixtureSha = Get-ShaBytes $fixtureBytes
foreach ($m in $methodLedger) {
    if (-not (Test-ByteRangeEquals $fixtureBytes $m.FixtureOffset $m.Slice)) {
        throw "fixture subsequence mismatch for $($m.Key)"
    }
    $actualSlice = [byte[]]::new($m.FixtureLength)
    [Array]::Copy($fixtureBytes, $m.FixtureOffset, $actualSlice, 0, $m.FixtureLength)
    if ((Get-ShaBytes $actualSlice) -ne $m.SourceSha256) {
        throw "fixture subsequence sha mismatch for $($m.Key)"
    }
}

# ---------------------------------------------------------------------------
# Copy authored harness files byte-exactly into the private project.
# ---------------------------------------------------------------------------
function Copy-ExactFile([string] $Source, [string] $Destination) {
    if ([IO.File]::Exists($Destination) -or [IO.Directory]::Exists($Destination)) {
        throw "REFUSING OVERWRITE: $Destination"
    }
    $bytes = [IO.File]::ReadAllBytes($Source)
    Write-NewFile $Destination $bytes
    $sourceSha = Get-ShaBytes $bytes
    $destBytes = [IO.File]::ReadAllBytes($Destination)
    if ($destBytes.Length -ne $bytes.Length -or (Get-ShaBytes $destBytes) -ne $sourceSha) {
        throw "copy mismatch: $Source -> $Destination"
    }
    return [pscustomobject]@{
        source = Get-RelativePath $Source
        destination = Get-RelativePath $Destination
        bytes = [int64]$bytes.Length
        sha256 = $sourceSha
    }
}

$copiedHarness = @(
    (Copy-ExactFile (Join-Path $HarnessDir 'TestScaffolding.cs') (Join-Path $ProjectDir 'TestScaffolding.cs')),
    (Copy-ExactFile (Join-Path $HarnessDir 'HarnessTests.cs') (Join-Path $ProjectDir 'HarnessTests.cs'))
)

# ---------------------------------------------------------------------------
# Project, NuGet config, SDK pin and inert nohook files
# ---------------------------------------------------------------------------
$csprojText = @'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>annotations</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
    <LangVersion>12.0</LangVersion>
    <AssemblyName>SnapshotDirectTest01</AssemblyName>
    <RootNamespace>SnapshotDirectTest01</RootNamespace>
    <UseAppHost>false</UseAppHost>
    <SelfContained>false</SelfContained>
    <DisableTransitiveFrameworkReferenceDownloads>true</DisableTransitiveFrameworkReferenceDownloads>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
    <GenerateAssemblyInfo>false</GenerateAssemblyInfo>
    <Deterministic>true</Deterministic>
    <RunAnalyzers>false</RunAnalyzers>
    <EnableNETAnalyzers>false</EnableNETAnalyzers>
    <RestoreDisableParallel>true</RestoreDisableParallel>
    <NuGetAudit>false</NuGetAudit>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="TestScaffolding.cs" />
    <Compile Include="HarnessTests.cs" />
    <Compile Include="RecoveredFixture.cs" />
  </ItemGroup>
</Project>
'@
Write-NewText (Join-Path $ProjectDir 'SnapshotDirectTest01.csproj') ($csprojText -split "`n")

$nugetText = @"
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <clear />
  </packageSources>
  <auditSources>
    <clear />
  </auditSources>
  <fallbackPackageFolders>
    <clear />
  </fallbackPackageFolders>
  <disabledPackageSources>
    <clear />
  </disabledPackageSources>
  <config>
    <add key="globalPackagesFolder" value="$Packages" />
  </config>
</configuration>
"@
Write-NewText (Join-Path $Scratch 'config\NuGet.Config') ($nugetText -split "`n")

$globalJsonText = @'
{
  "sdk": {
    "version": "9.0.312",
    "rollForward": "disable",
    "allowPrerelease": false
  }
}
'@
Write-NewText (Join-Path $Scratch 'global.json') ($globalJsonText -split "`n")

$emptyProjectText = @'
<Project>
</Project>
'@
foreach ($name in @('Directory.Build.props', 'Directory.Build.targets', 'no-before.props', 'no-after.props', 'no-before.targets', 'no-after.targets')) {
    Write-NewText (Join-Path $Scratch "nohooks\$name") ($emptyProjectText -split "`n")
}

# ---------------------------------------------------------------------------
# Copy the audited net8.0 targeting pack byte-exactly.
# ---------------------------------------------------------------------------
New-ExclusiveDirectory $RefPackDest
$packRows = [Collections.Generic.List[object]]::new()
$sourceRootFull = [IO.Path]::GetFullPath($RefPackSource)
foreach ($file in (Get-ChildItem -LiteralPath $RefPackSource -Recurse -File -Force | Sort-Object FullName)) {
    Assert-NoReparseChain $file.FullName
    $relative = [IO.Path]::GetRelativePath($sourceRootFull, $file.FullName)
    $destination = Join-Path $RefPackDest $relative
    Ensure-Directory ([IO.Path]::GetDirectoryName($destination))
    $bytes = [IO.File]::ReadAllBytes($file.FullName)
    Write-NewFile $destination $bytes
    $sha = Get-ShaBytes $bytes
    $copied = [IO.File]::ReadAllBytes($destination)
    if ($copied.Length -ne $bytes.Length -or (Get-ShaBytes $copied) -ne $sha) {
        throw "ref pack copy mismatch: $relative"
    }
    [void]$packRows.Add([pscustomobject]@{
        path = $relative.Replace('\', '/')
        bytes = [int64]$bytes.Length
        sha256 = $sha
    })
}

if ($packRows.Count -ne 351) { throw "ref pack destination file count is $($packRows.Count), expected 351" }

$packBytes = [int64]0
foreach ($row in $packRows) { $packBytes += $row.bytes }
$manifestLines = foreach ($row in $packRows) { "$($row.path)`t$($row.bytes)`t$($row.sha256)" }
$manifestText = ($manifestLines -join "`n") + "`n"
$manifestBytes = [Text.Encoding]::UTF8.GetBytes($manifestText)
$refPackManifestPath = Join-Path $provenanceDir 'refpack-manifest.tsv'
Write-NewFile $refPackManifestPath $manifestBytes
$refPackManifestSha = Get-ShaBytes $manifestBytes
Write-NewJson (Join-Path $provenanceDir 'refpack-manifest.json') ([ordered]@{
    schema = 're-snapshot-direct-test01-refpack-v1'
    source = $RefPackSource
    destination = Get-RelativePath $RefPackDest
    files = @($packRows)
    file_count = $packRows.Count
    total_bytes = $packBytes
    manifest_path = Get-RelativePath $refPackManifestPath
    manifest_bytes = [int64]$manifestBytes.Length
    manifest_sha256 = $refPackManifestSha
})

# ---------------------------------------------------------------------------
# Read-only environment, instructions and ancestor-hook audit
# ---------------------------------------------------------------------------
$dotnetDigest = Get-Digest $DotnetExe
$pwshDigest = Get-Digest $PwshExe
$sdkVersionDigest = Get-Digest $SdkVersionFile
$runtimeDigest = [pscustomobject]@{ path = $RuntimeDir; exists = [IO.Directory]::Exists($RuntimeDir); version = $runtimeVersion; reparse = $false }

$hookNames = @(
    'Directory.Build.props', 'Directory.Build.targets', 'Directory.Packages.props',
    'NuGet.Config', 'NuGet.config', 'global.json', 'MSBuild.rsp',
    'Directory.Build.rsp', 'Directory.Solution.props', 'Directory.Solution.targets'
)
$ancestorDirs = @(
    'C:\', 'C:\Users', $HOME, $WorkspaceRoot, $GithubRoot,
    (Join-Path $GithubRoot 'tests'),
    (Join-Path $GithubRoot 'tests\managed-recovered'),
    $HarnessDir,
    (Join-Path $GithubRoot 'tools'),
    (Join-Path $GithubRoot 'tools\re'),
    (Join-Path $GithubRoot 'tools\re\local-recovery')
)
$hookFound = [Collections.Generic.List[object]]::new()
foreach ($dir in $ancestorDirs) {
    if (-not [IO.Directory]::Exists($dir)) { continue }
    Assert-NoReparseChain $dir
    foreach ($name in $hookNames) {
        $p = Join-Path $dir $name
        if ([IO.File]::Exists($p)) {
            $d = Get-Digest $p
            [void]$hookFound.Add([ordered]@{
                path = Get-RelativePath $p
                bytes = $d.bytes
                sha256 = $d.sha256
            })
        }
    }
}
if ($hookFound.Count -gt 0) {
    throw "ancestor hook file(s) present and must be reviewed: $(($hookFound | ForEach-Object { $_.path }) -join ', ')"
}

$processHookKeys = @(
    'MSBuildUserExtensionsPath', 'MSBuildProjectExtensionsPath', 'DirectoryBuildPropsPath',
    'DirectoryBuildTargetsPath', 'CustomBeforeMicrosoftCommonProps', 'CustomAfterMicrosoftCommonProps',
    'CustomBeforeMicrosoftCommonTargets', 'CustomAfterMicrosoftCommonTargets',
    'ImportDirectoryBuildProps', 'ImportDirectoryBuildTargets', 'ImportDirectoryPackagesProps',
    'ImportUserLocationsByWildcardBeforeMicrosoftCommonProps',
    'ImportUserLocationsByWildcardAfterMicrosoftCommonProps',
    'ImportUserLocationsByWildcardBeforeMicrosoftCommonTargets',
    'ImportUserLocationsByWildcardAfterMicrosoftCommonTargets',
    'DOTNET_ROOT', 'DOTNET_CLI_HOME', 'NUGET_PACKAGES',
    'MSBuildEnableWorkloadResolver', 'DOTNET_CLI_DO_NOT_USE_MSBUILD_SERVER',
    'MSBUILDDISABLENODEREUSE', 'UseSharedCompilation'
)
$processHookValues = [ordered]@{}
foreach ($key in $processHookKeys) {
    $value = [Environment]::GetEnvironmentVariable($key, 'Process')
    $processHookValues[$key] = if ($null -eq $value) { '<unset>' } else { $value }
}

$nestedAgentPaths = @(
    (Join-Path $GithubRoot 'AGENTS.md'),
    (Join-Path $GithubRoot 'tests\AGENTS.md'),
    (Join-Path $GithubRoot 'tests\managed-recovered\AGENTS.md'),
    (Join-Path $HarnessDir 'AGENTS.md'),
    (Join-Path $GithubRoot 'tools\re\local-recovery\AGENTS.md')
)
$nestedAgents = @(
    foreach ($p in $nestedAgentPaths) {
        if ([IO.File]::Exists($p)) {
            $d = Get-Digest $p
            [ordered]@{ path = Get-RelativePath $p; bytes = $d.bytes; sha256 = $d.sha256 }
        }
    }
)
$rootAgentsDigest = Get-Digest (Join-Path $WorkspaceRoot 'AGENTS.md')

$runScriptPath = Join-Path $HarnessDir 'Run-SnapshotDirectTest01.ps1'
$runText = [IO.File]::ReadAllText($runScriptPath)
$requiredRunnerTokens = @(
    '-p:ImportDirectoryBuildProps=false',
    '-p:ImportDirectoryBuildTargets=false',
    '-p:ImportDirectoryPackagesProps=false',
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
    '-p:Deterministic=true',
    '-nodeReuse:false',
    'DOTNET_CLI_TELEMETRY_OPTOUT',
    'MSBuildEnableWorkloadResolver',
    'MSBUILDDISABLENODEREUSE',
    'DOTNET_CLI_DO_NOT_USE_MSBUILD_SERVER',
    'DOTNET_EnableDiagnostics',
    'COMPlus_EnableDiagnostics',
    'NUGET_PACKAGES',
    'MSBuildUserExtensionsPath',
    'DOTNET_BUNDLE_EXTRACT_BASE_DIR'
)
$missingRunnerTokens = @($requiredRunnerTokens | Where-Object { -not $runText.Contains($_, [StringComparison]::Ordinal) })
if ($missingRunnerTokens.Count -gt 0) {
    throw "Stage 2 runner is missing containment token(s): $($missingRunnerTokens -join ', ')"
}
$forbiddenRunnerTokens = @('dotnet run', 'dotnet test', 'dotnet clean', 'Remove-Item', 'Invoke-WebRequest', 'curl ')
$foundForbiddenRunnerTokens = @($forbiddenRunnerTokens | Where-Object { $runText.Contains($_, [StringComparison]::Ordinal) })
if ($foundForbiddenRunnerTokens.Count -gt 0) {
    throw "Stage 2 runner contains forbidden token(s): $($foundForbiddenRunnerTokens -join ', ')"
}

$environment = [ordered]@{
    schema = 're-snapshot-direct-test01-environment-v1'
    task = 'RE-SNAPSHOT-DIRECT-TEST01'
    stage = 1
    captured_utc = [DateTime]::UtcNow.ToString('o')
    powershell_version = $PSVersionTable.PSVersion.ToString()
    pwsh_exe = [ordered]@{ path = $PwshExe; bytes = $pwshDigest.bytes; sha256 = $pwshDigest.sha256 }
    dotnet_exe = [ordered]@{ path = $DotnetExe; bytes = $dotnetDigest.bytes; sha256 = $dotnetDigest.sha256 }
    sdk = [ordered]@{
        version = '9.0.312'
        version_file = $SdkVersionFile
        version_file_bytes = $sdkVersionDigest.bytes
        version_file_sha256 = $sdkVersionDigest.sha256
        version_lines = @($sdkVersionLines)
        verified = $true
    }
    runtime = $runtimeDigest
    refpack_source = [ordered]@{
        path = $RefPackSource
        files = $refPackSourceFiles.Count
        total_bytes = $refPackSourceBytes
        verified_file_count = 351
    }
    dotnet_invoked = $false
    compiler_invoked = $false
}
Write-NewJson (Join-Path $provenanceDir 'environment.json') $environment

$hookAudit = [ordered]@{
    schema = 're-snapshot-direct-test01-hook-audit-v1'
    task = 'RE-SNAPSHOT-DIRECT-TEST01'
    stage = 1
    root_agents_md = [ordered]@{
        path = 'AGENTS.md'
        bytes = $rootAgentsDigest.bytes
        sha256 = $rootAgentsDigest.sha256
    }
    nested_agents_md = @($nestedAgents)
    ancestor_directories_checked = @($ancestorDirs)
    hook_names_checked = @($hookNames)
    ancestor_hook_files_present = @($hookFound)
    ancestor_hook_files_present_count = $hookFound.Count
    unknown_hooks_executed = $false
    process_hook_environment = $processHookValues
    stage2_runner = [ordered]@{
        path = Get-RelativePath $runScriptPath
        required_tokens_checked = $requiredRunnerTokens.Count
        required_tokens_missing = @($missingRunnerTokens)
        forbidden_tokens_checked = @($forbiddenRunnerTokens)
        forbidden_tokens_found = @($foundForbiddenRunnerTokens)
        no_execute_audit_only = $true
    }
    sdk_refs = [ordered]@{
        sdk_version_file = $SdkVersionFile
        sdk_version_file_sha256 = $sdkVersionDigest.sha256
        dotnet_exe_sha256 = $dotnetDigest.sha256
        runtime_dir = $RuntimeDir
        runtime_version = $runtimeVersion
        refpack_source = $RefPackSource
        refpack_files = $refPackSourceFiles.Count
        refpack_bytes = $refPackSourceBytes
    }
    nohook_directory = Get-RelativePath (Join-Path $Scratch 'nohooks')
    nohook_files = @('Directory.Build.props','Directory.Build.targets','no-before.props','no-after.props','no-before.targets','no-after.targets')
}
Write-NewJson (Join-Path $provenanceDir 'hook-audit.json') $hookAudit

# ---------------------------------------------------------------------------
# Static containment checks on authored C# and exact persistent-slice absence.
# ---------------------------------------------------------------------------
$testScaffoldingPath = Join-Path $HarnessDir 'TestScaffolding.cs'
$harnessTestsPath = Join-Path $HarnessDir 'HarnessTests.cs'
$persistentCodePaths = @($testScaffoldingPath, $harnessTestsPath)
$forbiddenCodeTokens = @(
    'System.IO', 'System.Diagnostics', 'System.Net', 'System.Reflection',
    'System.Threading', 'System.Runtime.InteropServices', 'DllImport',
    'Marshal.', 'Activator.', 'Assembly.Load', 'Environment.', 'File.',
    'Directory.', 'Process', 'Thread', 'Task', 'DateTime', 'Stopwatch',
    'HttpClient', 'Socket', 'Array.Copy', 'CurrentValues[(int)attrId]',
    'BaseValues[(int)attrId]'
)
$forbiddenCodeFindings = [Collections.Generic.List[object]]::new()
foreach ($p in $persistentCodePaths) {
    $text = [IO.File]::ReadAllText($p)
    foreach ($token in $forbiddenCodeTokens) {
        if ($text.Contains($token, [StringComparison]::Ordinal)) {
            [void]$forbiddenCodeFindings.Add([ordered]@{ path = Get-RelativePath $p; token = $token })
        }
    }
}
if ($forbiddenCodeFindings.Count -gt 0) {
    throw "authored C# contains forbidden API or recovered-body token(s): $(($forbiddenCodeFindings | ForEach-Object { $_.path + ':' + $_.token }) -join ', ')"
}

$harnessText = [IO.File]::ReadAllText($harnessTestsPath)
$sidMatches = [regex]::Matches($harnessText, '\("S0[1-8]",')
if ($sidMatches.Count -ne 8) { throw "HarnessTests.cs declares $($sidMatches.Count) SIDs, expected 8" }
if ($harnessText.Contains('CalculateHurt', [StringComparison]::Ordinal)) { throw "HarnessTests.cs contains CalculateHurt" }
if ($harnessText.Contains('BaseDamageComponent', [StringComparison]::Ordinal)) { throw "HarnessTests.cs contains BaseDamageComponent" }
if (-not $harnessText.Contains('TakeSnapshot', [StringComparison]::Ordinal)) { throw "HarnessTests.cs does not call TakeSnapshot" }
if (-not $harnessText.Contains('GetCurrentValue', [StringComparison]::Ordinal)) { throw "HarnessTests.cs does not call GetCurrentValue" }
if (-not $harnessText.Contains('GetAttrFromSnapshotsForTest', [StringComparison]::Ordinal)) { throw "HarnessTests.cs does not call the wrapper" }

$scaffoldingText = [IO.File]::ReadAllText($testScaffoldingPath)
if (-not $scaffoldingText.Contains('return GetAttrFromSnapshots(', [StringComparison]::Ordinal)) {
    throw "TestScaffolding.cs wrapper does not call the private recovered method"
}
if ($scaffoldingText.Contains('BaseDamageComponent', [StringComparison]::Ordinal)) {
    throw "TestScaffolding.cs contains BaseDamageComponent"
}

$sliceAbsence = [Collections.Generic.List[object]]::new()
foreach ($m in $methodLedger) {
    foreach ($name in $ExpectedPersistentNames) {
        $p = Join-Path $HarnessDir $name
        $bytes = [IO.File]::ReadAllBytes($p)
        if (Test-BytesContain $bytes $m.Slice) {
            throw "persistent file contains exact recovered slice: $name ($($m.Key))"
        }
    }
    [void]$sliceAbsence.Add([ordered]@{
        method = $m.Key
        persistent_contains_exact_slice = $false
        persistent_files_checked = $ExpectedPersistentNames.Count
    })
}

# ---------------------------------------------------------------------------
# Post-write baseline and preservation verifier
# ---------------------------------------------------------------------------
$baselinePost = Get-Baseline
Write-NewJson (Join-Path $provenanceDir 'baseline-post.json') $baselinePost
$preAnchorJson = ($baselinePre.anchors | ConvertTo-Json -Depth 8 -Compress)
$postAnchorJson = ($baselinePost.anchors | ConvertTo-Json -Depth 8 -Compress)
if ($preAnchorJson -cne $postAnchorJson) { throw 'baseline anchor drift detected' }
if ($baselinePre.anchors_sha256 -ne $baselinePost.anchors_sha256) { throw 'baseline aggregate hash drift detected' }

$preservationPost = Invoke-Preservation 'post' $logDir
if (-not (Test-ByteRangeEquals $preservationPre.Result.Stdout 0 $preservationPost.Result.Stdout) -or
    $preservationPre.Result.Stdout.Length -ne $preservationPost.Result.Stdout.Length) {
    throw 'preservation verifier stdout changed between pre and post runs'
}

# ---------------------------------------------------------------------------
# Inventory (explicit self-exclusion) and completion manifest
# ---------------------------------------------------------------------------
function Get-Inventory([string[]] $Roots) {
    $rows = [Collections.Generic.List[object]]::new()
    foreach ($root in $Roots) {
        foreach ($file in (Get-ChildItem -LiteralPath $root -Recurse -File -Force | Sort-Object FullName)) {
            Assert-NoReparseChain $file.FullName
            $bytes = [IO.File]::ReadAllBytes($file.FullName)
            [void]$rows.Add([ordered]@{
                path = Get-RelativePath $file.FullName
                bytes = [int64]$bytes.Length
                sha256 = Get-ShaBytes $bytes
            })
        }
    }
    return @($rows | Sort-Object { $_.path })
}

$scratchInventory = Get-Inventory @($Scratch)
$harnessInventory = Get-Inventory @($HarnessDir)
if ($harnessInventory.Count -ne 5) { throw "persistent inventory count is $($harnessInventory.Count), expected 5" }

$methodLedgerForJson = foreach ($m in $methodLedger) {
    [ordered]@{
        key = $m.Key
        class = $m.Class
        source_path = Get-RelativePath $m.SourcePath
        source_range = "$($m.StartLine)-$($m.EndLine)"
        source_whole_bytes = $m.SourceWholeBytes
        source_whole_sha256 = $m.SourceWholeSha256
        source_offset = $m.SourceOffset
        source_length = $m.SourceLength
        source_sha256 = $m.SourceSha256
        fixture_offset = $m.FixtureOffset
        fixture_length = $m.FixtureLength
        fixture_sha256 = $m.SourceSha256
        equality = $true
        td = $m.Td
        md = $m.Md
        rva = $m.Rva
        code_size = $m.CodeSize
        status = $m.Status
        evidence = $m.Evidence
        cr_bytes = $m.CrBytes
    }
}

$completePath = Join-Path $provenanceDir 'prepare-complete.json'
$selfRelativePath = Get-RelativePath $completePath
$complete = [ordered]@{
    schema = 're-snapshot-direct-test01-prepare-v1'
    task = 'RE-SNAPSHOT-DIRECT-TEST01'
    stage = 1
    status = 'PREPARED_NOT_COMPILED'
    compiler_invoked = $false
    dotnet_invoked = $false
    network_used = $false
    git_used = $false
    mcp_used = $false
    repository_root = $WorkspaceRoot
    github_root = $GithubRoot
    persistent_root = Get-RelativePath $HarnessDir
    scratch = Get-RelativePath $Scratch
    assembly_sha256 = 'e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db'
    mvid = '2d6d37cb-76e5-4075-8220-3eeca41471e3'
    version = 'UNKNOWN'
    declaring_type_fix = [ordered]@{
        method = 'GetAttrFromSnapshots'
        md = '0x06018865'
        actual_declaring_type = 'Calculation'
        actual_declaring_type_token = '0x02002eb2'
        not_declaring_type = 'BaseDamageComponent'
        raw_methods_row = 642
        evidence = 'src/reconstructed/managed/semantic/gameplay/R38_METHOD_EVIDENCE_ADDENDUM.md:116,124-140; src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md row 289'
    }
    sources = @(
        foreach ($role in @('BAS', 'AS', 'CALC')) {
            [ordered]@{
                role = $role
                path = Get-RelativePath $sources[$role].path
                bytes = $sources[$role].bytes
                sha256 = $sources[$role].sha256
            }
        }
    )
    methods = @($methodLedgerForJson)
    fixture = [ordered]@{
        path = Get-RelativePath $fixturePath
        bytes = [int64]$fixtureBytes.Length
        sha256 = $fixtureSha
        method_count = $methodLedger.Count
        subsequence_verified = $true
    }
    refpack = [ordered]@{
        source = $RefPackSource
        destination = Get-RelativePath $RefPackDest
        files = $packRows.Count
        total_bytes = $packBytes
        manifest_path = Get-RelativePath $refPackManifestPath
        manifest_bytes = [int64]$manifestBytes.Length
        manifest_sha256 = $refPackManifestSha
    }
    baselines = [ordered]@{
        pre_path = 'provenance/baseline-pre.json'
        post_path = 'provenance/baseline-post.json'
        anchors_sha256 = $baselinePre.anchors_sha256
        anchors_equal = $true
        anchor_count = $baselinePre.anchor_count
    }
    preservation = [ordered]@{
        pre_exit_code = $preservationPre.Result.ExitCode
        pre_status = $preservationPre.Json.status
        pre_stdout_bytes = $preservationPre.Result.Stdout.Length
        pre_stdout_sha256 = Get-ShaBytes $preservationPre.Result.Stdout
        post_exit_code = $preservationPost.Result.ExitCode
        post_status = $preservationPost.Json.status
        post_stdout_bytes = $preservationPost.Result.Stdout.Length
        post_stdout_sha256 = Get-ShaBytes $preservationPost.Result.Stdout
        stdout_equal = $true
    }
    tests = [ordered]@{
        planned = 8
        sids = @('S01','S02','S03','S04','S05','S06','S07','S08')
        run = $false
        assertions = 'runtime-counted; not run in Stage 1'
        synthetic_model = 0
        recovered_methods = 3
    }
    persistent = [ordered]@{
        files = @($harnessInventory)
        file_count = $harnessInventory.Count
        expected_file_count = 5
        exact_slice_absence = @($sliceAbsence)
        forbidden_code_tokens = @()
        sids_declared = 8
        calculate_hurt_absent = $true
        base_damage_component_absent = $true
    }
    hook_audit = [ordered]@{
        path = 'provenance/hook-audit.json'
        ancestor_hook_files_present = $hookFound.Count
        unknown_hooks_executed = $false
        stage2_runner_required_tokens_missing = $missingRunnerTokens.Count
        stage2_runner_forbidden_tokens_found = $foundForbiddenRunnerTokens.Count
    }
    environment = [ordered]@{
        path = 'provenance/environment.json'
        sdk_version = '9.0.312'
        runtime_version = $runtimeVersion
        refpack_source_files = $refPackSourceFiles.Count
        refpack_source_bytes = $refPackSourceBytes
    }
    inventory = [ordered]@{
        scratch = @($scratchInventory)
        persistent = @($harnessInventory)
        self_excluded_paths = @($selfRelativePath)
        manual_or_outside_deletes_observed = 0
        automatic_compiler_transient_count = 'UNKNOWN_NOT_ZERO'
    }
    full_research_backup = 'NOT_ESTABLISHED'
    incident_383 = 'OPEN'
    limitations = @(
        'Private scratch fixture only; persistent harness contains no recovered method body.',
        'Authored hosts/test doubles are not recovered implementations; PASS later would mean exact core under authored hosts.',
        'AttributeSnapshot and BaseAttributeComponent field/allocator shape is authored TEST_DOUBLE scaffolding for the original whole-class context.',
        'EAttributeType is a minimal authored adapter subset; no whole-build, enum-compatibility or game-configuration claim is made.',
        'AttributeSnapshot.GetCurrentValue is historical source context with metadata binding, not a newly selected CIL body.',
        'GetAttrFromSnapshots declares on Calculation TD 0x02002eb2, not BaseDamageComponent; raw methods row 642 is the identity check.',
        'No dotnet/compiler/test/MCP/Git/network/game/Cargo/delete operation was performed in Stage 1.',
        'The preservation verifier is pinned-byte integrity only, not an OS sandbox.',
        'Full research backup is NOT ESTABLISHED; incident 383 remains OPEN.',
        'Automatic compiler transient count is UNKNOWN, not claimed as zero.'
    )
}
Write-NewJson $completePath $complete

Write-Host 'PREPARE PASS'
Write-Host "scratch=$Scratch"
Write-Host "persistent=$HarnessDir"
Write-Host "fixture=$fixturePath bytes=$($fixtureBytes.Length) sha256=$fixtureSha"
Write-Host "methods=$($methodLedger.Count)"
Write-Host "refpack_files=$($packRows.Count) bytes=$packBytes manifest_sha256=$refPackManifestSha"
Write-Host "preservation_pre=$($preservationPre.Json.status) preservation_post=$($preservationPost.Json.status)"
Write-Host "manifest=$completePath"
