#requires -Version 7.0
# RE-BUFF-DIRECT-TEST01 Stage 1 preparation.
# Scope: create only the new task scratch tree, byte-slice ten exact recovered
# methods into a private fixture, copy the audited 8.0.25 ref pack, run the
# read-only preservation verifier, and record baselines/manifests.
# This script does NOT invoke dotnet, a compiler, tests, MCP, Git, or the network.
[CmdletBinding()]
param(
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
$Packages      = Join-Path $Scratch 'nuget\packages'
$RefPackSource = 'REQUIRED_NET8_REFERENCE_PACK_DIRECTORY'
$RefPackDest   = Join-Path $Packages 'microsoft.netcore.app.ref\8.0.25'
$Verifier      = Join-Path $GithubRoot 'tools\re\verify-preservation.ps1'
$PriorRunner   = Join-Path $GithubRoot 'tools\re\local-recovery\combat-reference-verify01\scratch\runner\Run-Event101.ps1'
$PwshExe       = Join-Path $PSHOME 'pwsh.exe'

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

function New-Directory([string] $Path) {
    if (Test-Path -LiteralPath $Path) { throw "REFUSING OVERWRITE: directory exists: $Path" }
    New-Item -ItemType Directory -Path $Path -ErrorAction Stop | Out-Null
}

function Ensure-Directory([string] $Path) {
    if (Test-Path -LiteralPath $Path) {
        if (-not (Get-Item -LiteralPath $Path -Force).PSIsContainer) {
            throw "not a directory: $Path"
        }
        return
    }
    [void][IO.Directory]::CreateDirectory($Path)
}

function Write-NewFile([string] $Path, [byte[]] $Bytes) {
    if (Test-Path -LiteralPath $Path) { throw "REFUSING OVERWRITE: file exists: $Path" }
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
        $finished = $proc.WaitForExit(900000)
        if (-not $finished) {
            try { $proc.Kill($true) } catch { }
            [void]$proc.WaitForExit()
            throw "process timed out: $Exe"
        }
        [void]$tout.GetAwaiter().GetResult()
        [void]$terr.GetAwaiter().GetResult()
        return [pscustomobject]@{
            ExitCode = $proc.ExitCode
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
    $result = Invoke-CapturedProcess $PwshExe @(
        '-NoProfile',
        '-File', $Verifier,
        '-RepositoryRoot', $GithubRoot
    ) $GithubRoot

    Write-NewFile (Join-Path $LogDir "preservation-$Suffix.stdout.bin") $result.Stdout
    Write-NewFile (Join-Path $LogDir "preservation-$Suffix.stderr.bin") $result.Stderr
    Write-NewText (Join-Path $LogDir "preservation-$Suffix.meta.txt") @(
        "suffix=$Suffix",
        "exit_code=$($result.ExitCode)",
        "stdout_bytes=$($result.Stdout.Length)",
        "stdout_sha256=$(Get-ShaBytes $result.Stdout)",
        "stderr_bytes=$($result.Stderr.Length)",
        "stderr_sha256=$(Get-ShaBytes $result.Stderr)"
    )

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
foreach ($p in @(
    $GithubRoot,
    (Join-Path $GithubRoot 'tests'),
    (Join-Path $GithubRoot 'tests\managed-recovered'),
    $HarnessDir,
    (Join-Path $GithubRoot 'tools'),
    (Join-Path $GithubRoot 'tools\re'),
    (Join-Path $GithubRoot 'tools\re\local-recovery'),
    $Verifier,
    $PriorRunner,
    $PwshExe
)) {
    if (-not (Test-Path -LiteralPath $p)) { throw "precondition missing: $p" }
    Assert-NoReparseChain $p
}

if (Test-Path -LiteralPath $TaskRoot) { throw "task parent already exists: $TaskRoot" }
if (Test-Path -LiteralPath $Scratch) { throw "scratch already exists: $Scratch" }
if (Test-Path -LiteralPath $RefPackDest) { throw "ref pack destination already exists: $RefPackDest" }

foreach ($name in @('TestDoubles.cs', 'HarnessTests.cs', 'Prepare-BuffDirectTest01.ps1', 'Run-BuffDirectTest01.ps1', 'README.md')) {
    $p = Join-Path $HarnessDir $name
    if (-not (Test-Path -LiteralPath $p)) { throw "harness file missing: $p" }
}

$sdkVersionFile = 'C:\Program Files\dotnet\sdk\9.0.312\.version'
$runtimeDir = 'C:\Program Files\dotnet\shared\Microsoft.NETCore.App\8.0.11'
$dotnetExe = 'C:\Program Files\dotnet\dotnet.exe'
foreach ($p in @($sdkVersionFile, $runtimeDir, $dotnetExe, $RefPackSource)) {
    if (-not (Test-Path -LiteralPath $p)) { throw "read-only tool input missing: $p" }
    Assert-NoReparseChain $p
}

# ---------------------------------------------------------------------------
# Create the exclusive scratch tree (no existing file/directory is replaced).
# ---------------------------------------------------------------------------
New-Directory $TaskRoot
New-Directory $Scratch
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
    'run',
    'logs',
    'provenance',
    'receipts'
)) {
    New-Directory (Join-Path $Scratch $rel)
}

$logDir = Join-Path $Scratch 'logs'
$provenanceDir = Join-Path $Scratch 'provenance'

# ---------------------------------------------------------------------------
# Baseline anchors: must be byte-identical before and after the new writes.
# ---------------------------------------------------------------------------
$anchorSpecs = @(
    @{ role = 'source.X'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/BaseAttributeComponent.cs' },
    @{ role = 'source.A'; path = 'src/reconstructed/managed/full/CSharpScript/shards/acb55285e74c71a95fb06fb83c0fcc2bde6108a38a8d8e74f23c6f2a46112d0d/source/_global/ActiveBuffInternal.cs' },
    @{ role = 'source.U'; path = 'src/reconstructed/managed/full/CSharpScript/shards/915ffeea66739cd2f9098f048312b97f6dc3410709b2d8622aca7e9dd040ef07/source/_global/AbilityUtils.cs' },
    @{ role = 'map'; path = 'src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md' },
    @{ role = 'semantic.storage'; path = 'src/reconstructed/managed/semantic/gameplay/BaseAttributeStorage.cs' },
    @{ role = 'curated.BaseAttributeComponent'; path = 'github_src/src/managed/cs-e3fc67839851/attributes/BaseAttributeComponent.cs' },
    @{ role = 'curated.ActiveBuffInternal'; path = 'github_src/src/managed/cs-e3fc67839851/attributes/ActiveBuffInternal.cs' },
    @{ role = 'enum.ECalculationPolicyType'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/ECalculationPolicyType.cs' },
    @{ role = 'enum.EAttributeBasedFloatCalculationType'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/EAttributeBasedFloatCalculationType.cs' },
    @{ role = 'contract.IAttributeSet'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/IAttributeSet.cs' },
    @{ role = 'contract.IBoundsLocker'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/IBoundsLocker.cs' },
    @{ role = 'contract.IBuffModifierData'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/source/_global/IBuffModifierData.cs' },
    @{ role = 'context.CharacterAttributeTypes'; path = 'src/reconstructed/managed/full/CSharpScript/shards/d3a27139dfcea4922c249db273352613fdfb87a270b7bea7eda8923b697af86e/source/_global/CharacterAttributeTypes.cs' },
    @{ role = 'verifier'; path = 'github_src/tools/re/verify-preservation.ps1' },
    @{ role = 'prior.runner'; path = 'github_src/tools/re/local-recovery/combat-reference-verify01/scratch/runner/Run-Event101.ps1' },
    @{ role = 'evidence.X.methods'; path = 'src/reconstructed/managed/full/CSharpScript/shards/32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8/evidence/raw/methods.jsonl' },
    @{ role = 'evidence.A.methods'; path = 'src/reconstructed/managed/full/CSharpScript/shards/acb55285e74c71a95fb06fb83c0fcc2bde6108a38a8d8e74f23c6f2a46112d0d/evidence/raw/methods.jsonl' },
    @{ role = 'evidence.U.methods'; path = 'src/reconstructed/managed/full/CSharpScript/shards/915ffeea66739cd2f9098f048312b97f6dc3410709b2d8622aca7e9dd040ef07/evidence/raw/methods.jsonl' }
)

function Get-Baseline {
    $rows = foreach ($spec in $anchorSpecs) {
        $full = Join-Path $WorkspaceRoot $spec.path
        if (-not (Test-Path -LiteralPath $full)) { throw "anchor missing: $($spec.path)" }
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
        schema = 're-buff-direct-test01-baseline-v1'
        anchors = $sorted
        anchors_sha256 = Get-ShaBytes ([Text.Encoding]::UTF8.GetBytes($anchorsText))
    }
}

$baselinePre = Get-Baseline
Write-NewJson (Join-Path $provenanceDir 'baseline-pre.json') $baselinePre

# Read-only preservation verifier, before fixture/project writes.
$preservationPre = Invoke-Preservation 'pre' $logDir

# ---------------------------------------------------------------------------
# Exact source ledger and byte slicing.
# ---------------------------------------------------------------------------
$sources = @{
    X = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\32152cd0de30a64721c6311b291be1a7a6ab2e16c018715de6315b3fc39d29d8\source\_global\BaseAttributeComponent.cs'
        bytes = 35847
        sha256 = 'c2b14c11808966b4ee81301b631f52f9e7c07777bdd9e72758bfc81f686fb6b9'
    }
    A = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\acb55285e74c71a95fb06fb83c0fcc2bde6108a38a8d8e74f23c6f2a46112d0d\source\_global\ActiveBuffInternal.cs'
        bytes = 36427
        sha256 = '38bf6220daa1cef5f4a71e7e39c6a3376eddfb06f133c140bd72dc509667b74d'
    }
    U = @{
        path = Join-Path $WorkspaceRoot 'src\reconstructed\managed\full\CSharpScript\shards\915ffeea66739cd2f9098f048312b97f6dc3410709b2d8622aca7e9dd040ef07\source\_global\AbilityUtils.cs'
        bytes = 2442
        sha256 = '3df28587ba45fbf96167ed2f4a941e8be81c17dfb443f8a13207c39bfff6ac0b'
    }
}

$methodSpecs = @(
    [pscustomobject]@{ Key='SetBaseValue'; Class='BaseAttributeComponent'; Role='X'; StartLine=299; EndLine=323; Bytes=678; Sha256='59ee5dc0102152176ffea92c874ce8dcd93fb65e0410261e4e2aee4ffc4e3076'; Signature="`tpublic void SetBaseValue(EAttributeType attrId, float value)"; Td='0x02002e53'; Md='0x060184bc'; Rva=7112520; CodeSize=139; Status='ILSPY_FULL_CIL'; Evidence='METHOD_EVIDENCE_MAP.md:3831' },
    [pscustomobject]@{ Key='AddBaseValue'; Class='BaseAttributeComponent'; Role='X'; StartLine=325; EndLine=328; Bytes=125; Sha256='4d27cfb2ebf8412db1236961df007c792b99af15947c7ce36a4cfc527f82371a'; Signature="`tpublic void AddBaseValue(EAttributeType attrId, float value)"; Td='0x02002e53'; Md='0x060184bd'; Rva=7112671; CodeSize=18; Status='ILSPY_FULL_CIL'; Evidence='METHOD_EVIDENCE_MAP.md:3832' },
    [pscustomobject]@{ Key='GetBaseValue'; Class='BaseAttributeComponent'; Role='X'; StartLine=330; EndLine=333; Bytes=90; Sha256='1a96350e7e86891bbe509879bec7bcce1d7d5c94312b38c5fa2f54bc81130d03'; Signature="`tpublic float GetBaseValue(EAttributeType attrId)"; Td='0x02002e53'; Md='0x060184be'; Rva=7112690; CodeSize=9; Status='UNSELECTED_SOURCE_CONTEXT_ONLY'; Evidence='raw methods token/RVA; no map row' },
    [pscustomobject]@{ Key='GetCurrentValue'; Class='BaseAttributeComponent'; Role='X'; StartLine=335; EndLine=338; Bytes=96; Sha256='2e467b842558ac501d1d3f02ec879a969906a09fefe724fc24f991f57aed41f6'; Signature="`tpublic float GetCurrentValue(EAttributeType attrId)"; Td='0x02002e53'; Md='0x060184bf'; Rva=7112700; CodeSize=9; Status='SOURCE_RAW_ONLY'; Evidence='METHOD_EVIDENCE_MAP.md:3833' },
    [pscustomobject]@{ Key='GetMaxValue'; Class='BaseAttributeComponent'; Role='X'; StartLine=348; EndLine=355; Bytes=200; Sha256='5d307fb86bf8c0da3e3b9ec03b459d12c7cfe9024e293cc5e44c27aa32f4c410'; Signature="`tprivate float? GetMaxValue(EAttributeType attrId)"; Td='0x02002e53'; Md='0x060184c2'; Rva=7112716; CodeSize=38; Status='UNSELECTED_SOURCE_CONTEXT_ONLY'; Evidence='METHOD_EVIDENCE_MAP.md:3987' },
    [pscustomobject]@{ Key='RefreshBounds'; Class='BaseAttributeComponent'; Role='X'; StartLine=673; EndLine=699; Bytes=712; Sha256='fb4b2b93a7585df4b9c6ee6acc626b404f2d8659b1b4dc6bdec34cf8e9b3c51a'; Signature="`tprivate float RefreshBounds(EAttributeType attrId, float baseValue, float maxValue)"; Td='0x02002e53'; Md='0x060184d0'; Rva=7115160; CodeSize=204; Status='UNSELECTED_SOURCE_CONTEXT_ONLY'; Evidence='METHOD_EVIDENCE_MAP.md:3987' },
    [pscustomobject]@{ Key='DispatchCurrentValueEvent'; Class='BaseAttributeComponent'; Role='X'; StartLine=926; EndLine=932; Bytes=203; Sha256='cffd0f4d5f2353e14b2c2e33786b519ed91c2dbc6a4fc8fd6216477fd956a230'; Signature="`tpublic void DispatchCurrentValueEvent(EAttributeType attrId, float newValue, float oldValue)"; Td='0x02002e53'; Md='0x060184e1'; Rva=7116736; CodeSize=15; Status='FULL_SELECTED_BODY/SOURCE_ONLY_PRIMARY_ILSPY'; Evidence='METHOD_EVIDENCE_MAP.md:513' },
    [pscustomobject]@{ Key='ModifyStateAttribute'; Class='ActiveBuffInternal'; Role='A'; StartLine=933; EndLine=1040; Bytes=3663; Sha256='412828bef096ae8288e712e553623be340e23045d2cd60b0b26576d97d2ec52c'; Signature="`tpublic static void ModifyStateAttribute(IAttributeSet? instigator, BaseAttributeComponent owner, IBuffModifierData modifier, int level, float timeDilation, int stackCount, float? snapshot = null)"; Td='0x02002e71'; Md='0x06018716'; Rva=7181812; CodeSize=735; Status='UNSELECTED_SAVED_SOURCE'; Evidence='scope directive; METHOD_EVIDENCE_MAP.md:4011 context' },
    [pscustomobject]@{ Key='GetLevelValue'; Class='AbilityUtils'; Role='U'; StartLine=5; EndLine=20; Bytes=321; Sha256='3b3c49092b7d0934aae98ebf6bb50f1d6f740e9d3b9c8b96021b6a5cc4cada51'; Signature="`tpublic static T GetLevelValue<T>(T[]? array, int level, T defaultValueIfNotFound)"; Td='0x02002e4e'; Md='0x0601848d'; Rva=7109925; CodeSize=54; Status='UNSELECTED_SAVED_SOURCE'; Evidence='scope directive' },
    [pscustomobject]@{ Key='GetAttrValue'; Class='AbilityUtils'; Role='U'; StartLine=35; EndLine=47; Bytes=470; Sha256='2e0078033de6bc04c116318aa912afc28b834ac3c4798830bce2a00c9436843b'; Signature="`tpublic static float GetAttrValue(IAttributeSet? attrSet, EAttributeType attrId, EAttributeBasedFloatCalculationType attrCalcPolicy)"; Td='0x02002e4e'; Md='0x0601848f'; Rva=7110012; CodeSize=61; Status='UNSELECTED_SAVED_SOURCE'; Evidence='scope directive' }
)

$sourceCache = @{}
foreach ($role in $sources.Keys) {
    $spec = $sources[$role]
    if (-not (Test-Path -LiteralPath $spec.path)) { throw "source missing: $($spec.path)" }
    $bytes = [IO.File]::ReadAllBytes($spec.path)
    if ($bytes.Length -ne $spec.bytes) { throw "source size drift for $role" }
    if ((Get-ShaBytes $bytes) -ne $spec.sha256) { throw "source sha drift for $role" }
    $sourceCache[$role] = $bytes
}

$methodLedger = foreach ($m in $methodSpecs) {
    $bytes = $sourceCache[$m.Role]
    $offsets = [Collections.Generic.List[int]]::new()
    $offsets.Add(0)
    for ($i = 0; $i -lt $bytes.Length; $i++) {
        if ($bytes[$i] -eq 10) { $offsets.Add($i + 1) }
    }
    if ($offsets.Count -lt $m.EndLine) { throw "not enough lines for $($m.Key)" }
    $start = $offsets[$m.StartLine - 1]
    $end = if ($m.EndLine -lt $offsets.Count) { $offsets[$m.EndLine] } else { $bytes.Length }
    $length = $end - $start
    $slice = [byte[]]::new($length)
    [Array]::Copy($bytes, $start, $slice, 0, $length)
    if ($length -ne $m.Bytes) { throw "slice size drift for $($m.Key): $length" }
    if ((Get-ShaBytes $slice) -ne $m.Sha256) { throw "slice sha drift for $($m.Key)" }

    $text = [Text.Encoding]::UTF8.GetString($slice)
    $lines = $text -split "`n"
    $firstLine = $lines[0].TrimEnd("`r")
    if ($firstLine -ne $m.Signature) { throw "start signature mismatch for $($m.Key): [$firstLine]" }
    $lastLine = $lines[$lines.Count - 2].TrimEnd("`r")
    if ($lastLine -ne "`t}") { throw "end brace mismatch for $($m.Key)" }
    $open = 0; $close = 0
    foreach ($b in $slice) {
        if ($b -eq 123) { $open++ }
        elseif ($b -eq 125) { $close++ }
    }
    if ($open -ne $close) { throw "brace imbalance for $($m.Key): $open/$close" }

    [pscustomobject]@{
        Key = $m.Key
        Class = $m.Class
        Role = $m.Role
        StartLine = $m.StartLine
        EndLine = $m.EndLine
        SourceBytes = $m.Bytes
        SourceSha256 = $m.Sha256
        SourceOffset = $start
        Td = $m.Td
        Md = $m.Md
        Rva = $m.Rva
        CodeSize = $m.CodeSize
        Status = $m.Status
        Evidence = $m.Evidence
        Slice = $slice
    }
}

# ---------------------------------------------------------------------------
# Private fixture: authored host declarations + exact byte slices only.
# ---------------------------------------------------------------------------
$fixtureHeader = @"
// PRIVATE SCRATCH FIXTURE - RE-BUFF-DIRECT-TEST01.
// Generated by Prepare-BuffDirectTest01.ps1 from exact saved-source byte slices.
// Not persistent, not public. Do not edit.
// Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
using System;
using System.Collections.Generic;
using System.Linq;

"@

$fixtureStream = [IO.MemoryStream]::new()
try {
    $headerBytes = [Text.Encoding]::UTF8.GetBytes($fixtureHeader)
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
# Copy authored harness files byte-exactly into the scratch project.
# ---------------------------------------------------------------------------
function Copy-ExactFile([string] $Source, [string] $Destination) {
    if (Test-Path -LiteralPath $Destination) { throw "REFUSING OVERWRITE: $Destination" }
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
    (Copy-ExactFile (Join-Path $HarnessDir 'TestDoubles.cs') (Join-Path $ProjectDir 'TestDoubles.cs')),
    (Copy-ExactFile (Join-Path $HarnessDir 'HarnessTests.cs') (Join-Path $ProjectDir 'HarnessTests.cs'))
)

$csprojText = @'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>annotations</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
    <LangVersion>12.0</LangVersion>
    <AssemblyName>BuffDirectTest01</AssemblyName>
    <RootNamespace>BuffDirectTest01</RootNamespace>
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
    <Compile Include="TestDoubles.cs" />
    <Compile Include="HarnessTests.cs" />
    <Compile Include="RecoveredFixture.cs" />
  </ItemGroup>
</Project>
'@
Write-NewText (Join-Path $ProjectDir 'BuffDirectTest01.csproj') ($csprojText -split "`n")

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

$emptyProject = @'
<Project>
</Project>
'@
foreach ($name in @('Directory.Build.props', 'Directory.Build.targets', 'no-before.props', 'no-after.props', 'no-before.targets', 'no-after.targets')) {
    Write-NewText (Join-Path $Scratch "nohooks\$name") ($emptyProject -split "`n")
}

# ---------------------------------------------------------------------------
# Copy the audited net8.0 targeting pack byte-exactly.
# ---------------------------------------------------------------------------
$packRows = [Collections.Generic.List[object]]::new()
$sourceRootFull = [IO.Path]::GetFullPath($RefPackSource)
foreach ($file in (Get-ChildItem -LiteralPath $RefPackSource -Recurse -File -Force | Sort-Object FullName)) {
    Assert-NoReparseChain $file.FullName
    $relative = [IO.Path]::GetRelativePath($sourceRootFull, $file.FullName)
    $destination = Join-Path $RefPackDest $relative
    Ensure-Directory ([IO.Path]::GetDirectoryName($destination))
    if (Test-Path -LiteralPath $destination) { throw "REFUSING OVERWRITE: $destination" }
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

$packBytes = [int64]0
foreach ($row in $packRows) { $packBytes += $row.bytes }
$manifestLines = foreach ($row in $packRows) { "$($row.path)`t$($row.bytes)`t$($row.sha256)" }
$manifestText = ($manifestLines -join "`n") + "`n"
$manifestBytes = [Text.Encoding]::UTF8.GetBytes($manifestText)
$manifestPath = Join-Path $provenanceDir 'refpack-manifest.tsv'
Write-NewFile $manifestPath $manifestBytes
$manifestSha = Get-ShaBytes $manifestBytes
Write-NewJson (Join-Path $provenanceDir 'refpack-manifest.json') ([ordered]@{
    schema = 're-buff-direct-test01-refpack-v1'
    source = $RefPackSource
    destination = Get-RelativePath $RefPackDest
    files = @($packRows)
    total_bytes = $packBytes
    manifest_path = Get-RelativePath $manifestPath
    manifest_bytes = [int64]$manifestBytes.Length
    manifest_sha256 = $manifestSha
})

# ---------------------------------------------------------------------------
# Post-write baseline and preservation verifier.
# ---------------------------------------------------------------------------
$baselinePost = Get-Baseline
Write-NewJson (Join-Path $provenanceDir 'baseline-post.json') $baselinePost
$preAnchorJson = ($baselinePre.anchors | ConvertTo-Json -Depth 8 -Compress)
$postAnchorJson = ($baselinePost.anchors | ConvertTo-Json -Depth 8 -Compress)
if ($preAnchorJson -ne $postAnchorJson) { throw 'baseline anchor drift detected' }
if ($baselinePre.anchors_sha256 -ne $baselinePost.anchors_sha256) { throw 'baseline aggregate hash drift detected' }

$preservationPost = Invoke-Preservation 'post' $logDir
if (-not (Test-ByteRangeEquals $preservationPre.Result.Stdout 0 $preservationPost.Result.Stdout) -or
    $preservationPre.Result.Stdout.Length -ne $preservationPost.Result.Stdout.Length) {
    throw 'preservation verifier stdout changed between pre and post runs'
}

# ---------------------------------------------------------------------------
# Inventory and completion manifest.
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

$methodLedgerForJson = foreach ($m in $methodLedger) {
    [ordered]@{
        key = $m.Key
        class = $m.Class
        role = $m.Role
        source_range = "$($m.StartLine)-$($m.EndLine)"
        source_bytes = $m.SourceBytes
        source_sha256 = $m.SourceSha256
        source_offset = $m.SourceOffset
        fixture_offset = $m.FixtureOffset
        fixture_length = $m.FixtureLength
        fixture_sha256 = $m.SourceSha256
        td = $m.Td
        md = $m.Md
        rva = $m.Rva
        code_size = $m.CodeSize
        status = $m.Status
        evidence = $m.Evidence
    }
}

$complete = [ordered]@{
    schema = 're-buff-direct-test01-prepare-v1'
    task = 'RE-BUFF-DIRECT-TEST01'
    stage = 1
    status = 'PREPARED_NOT_COMPILED'
    compiler_invoked = $false
    dotnet_invoked = $false
    repository_root = $WorkspaceRoot
    github_root = $GithubRoot
    harness_dir = Get-RelativePath $HarnessDir
    scratch = Get-RelativePath $Scratch
    assembly_sha256 = 'e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db'
    mvid = '2d6d37cb-76e5-4075-8220-3eeca41471e3'
    version = 'UNKNOWN'
    sources = @(
        foreach ($role in @('X', 'A', 'U')) {
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
        manifest_path = Get-RelativePath $manifestPath
        manifest_bytes = [int64]$manifestBytes.Length
        manifest_sha256 = $manifestSha
    }
    baselines = [ordered]@{
        pre_path = 'provenance/baseline-pre.json'
        post_path = 'provenance/baseline-post.json'
        anchors_sha256 = $baselinePre.anchors_sha256
        equal = $true
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
        planned = 28
        run = $false
        assertions = 'runtime-counted; not run in Stage 1'
        synthetic_model = 0
    }
    copied_harness = @($copiedHarness)
    harness_inventory = @($harnessInventory)
    scratch_inventory = @($scratchInventory)
    manifest_self_excluded = $true
    limitations = @(
        'Private scratch fixture only; persistent harness contains no recovered method body.',
        'Test doubles are not recovered implementations; PASS will mean real core under stubs.',
        'No dotnet/compiler/test/MCP/Git/network/game/Cargo operation in Stage 1.',
        'Preservation verifier is pinned-byte integrity only, not an OS sandbox.'
    )
}

$completePath = Join-Path $provenanceDir 'prepare-complete.json'
Write-NewJson $completePath $complete

Write-Host 'PREPARE PASS'
Write-Host "scratch=$Scratch"
Write-Host "fixture=$fixturePath bytes=$($fixtureBytes.Length) sha256=$fixtureSha"
Write-Host "methods=$($methodLedger.Count)"
Write-Host "refpack_files=$($packRows.Count) bytes=$packBytes manifest_sha256=$manifestSha"
Write-Host "preservation_pre=$($preservationPre.Json.status) preservation_post=$($preservationPost.Json.status)"
Write-Host "manifest=$completePath"
