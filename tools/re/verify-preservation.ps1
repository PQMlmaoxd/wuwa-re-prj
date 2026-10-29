[CmdletBinding()]
param(
    [string] $RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)

$script:Schema = 'wuwa-reconstruction-preservation-verification-v1'
$script:Task = 'RE-HARDEN-SAFETY-CLOSE01'
$script:Root = ''
$script:ScriptSha256 = ''
$script:Checks = [Collections.Generic.List[object]]::new()
$script:ByPath = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::OrdinalIgnoreCase)
$script:Errors = [Collections.Generic.List[string]]::new()
$script:RootPins = [Collections.Generic.List[object]]::new()
$script:Historical = $null
$script:RootRecheckStable = $false
$script:Stats = [ordered]@{
    records_observed = 0
    duplicate_records_merged = 0
    conflicting_pins = 0
    malformed = 0
    reparse_rejected = 0
}
$script:ScopeCounts = [ordered]@{
    root_chain = 6
    harden_authority_pins = 0
    harden_authority_receipts = 0
    access_manifests = 0
    frozen_code_rows = 0
    source131_files = 0
}
$script:ReservedNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach($name in @('CON','PRN','AUX','NUL','COM1','COM2','COM3','COM4','COM5','COM6','COM7','COM8','COM9','LPT1','LPT2','LPT3','LPT4','LPT5','LPT6','LPT7','LPT8','LPT9','CONIN$','CONOUT$')){
    [void]$script:ReservedNames.Add($name)
}

function Inc-Stat([string]$Name) {
    $script:Stats[$Name] = [int]$script:Stats[$Name] + 1
}

function Add-Error([string]$Message) {
    [void]$script:Errors.Add($Message)
}

function ConvertTo-SafeRelativePath([string]$Path) {
    if([string]::IsNullOrEmpty($Path)) { throw 'path is empty' }
    if($Path.Length -gt 512) { throw 'path is longer than 512 characters' }
    if($Path -match '[\x00-\x1F\x7F]') { throw 'path contains a control character' }
    if($Path -match '[\\:*?"<>|]') { throw 'path contains a forbidden character' }
    if($Path.StartsWith('/')) { throw 'path is absolute' }
    if($Path.EndsWith('/')) { throw 'path ends with a slash' }
    if($Path.Contains('//')) { throw 'path contains an empty segment' }
    $segments = $Path.Split('/')
    foreach($segment in $segments) {
        if($segment -eq '.' -or $segment -eq '..') { throw 'path contains a dot segment' }
        if($segment.EndsWith('.') -or $segment.EndsWith(' ')) { throw 'path segment ends with a dot or space' }
        $stem = $segment.Split('.')[0]
        if($script:ReservedNames.Contains($stem)) { throw 'path contains a reserved device name' }
    }
    return $Path
}

function Assert-NoReparseChain([string]$Path) {
    $current = [IO.Path]::GetFullPath($Path)
    while($true) {
        if(-not [IO.File]::Exists($current) -and -not [IO.Directory]::Exists($current)) {
            throw "path component does not exist: $current"
        }
        $attributes = [IO.File]::GetAttributes($current)
        if(($attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
            throw "reparse point rejected: $current"
        }
        $parent = [IO.Path]::GetDirectoryName($current)
        if([string]::IsNullOrEmpty($parent)) { break }
        if($parent -eq $current) { break }
        $current = $parent
    }
}

function Resolve-InputPath([string]$RelativePath) {
    $safe = ConvertTo-SafeRelativePath $RelativePath
    $combined = [IO.Path]::Combine($script:Root, $safe.Replace('/','\'))
    $candidate = [IO.Path]::GetFullPath($combined)
    $trimmed = $script:Root.TrimEnd([char[]]@('\','/'))
    $prefix = $trimmed + [IO.Path]::DirectorySeparatorChar
    if(-not $candidate.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "path escapes the repository root: $RelativePath"
    }
    return $candidate
}

function Get-FileDigest([string]$FullPath) {
    $stream = [IO.FileStream]::new($FullPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        $length = $stream.Length
        $digest = [Security.Cryptography.SHA256]::HashData($stream)
    }
    finally {
        $stream.Dispose()
    }
    $record = [ordered]@{
        bytes = [int64]$length
        sha256 = [Convert]::ToHexString($digest).ToLowerInvariant()
    }
    return [pscustomobject]$record
}

function Read-VerifiedBytes([string]$RelativePath) {
    $safe = ConvertTo-SafeRelativePath $RelativePath
    if(-not $script:ByPath.ContainsKey($safe)) { throw "no pinned root record for '$safe'" }
    $pin = $script:ByPath[$safe]
    if($pin.category -ne 'root_chain') { throw "byte-buffer guard requires a root_chain pin for '$safe'" }
    $full = Resolve-InputPath $safe
    if(-not [IO.File]::Exists($full)) { throw "missing file: $safe" }
    Assert-NoReparseChain $full
    $stream = [IO.FileStream]::new($full, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        $memory = [IO.MemoryStream]::new()
        try {
            $stream.CopyTo($memory)
            $bytes = $memory.ToArray()
        }
        finally {
            $memory.Dispose()
        }
    }
    finally {
        $stream.Dispose()
    }
    $length = [int64]$bytes.Length
    if($null -ne $pin.expected_bytes -and $length -ne $pin.expected_bytes) {
        throw ("byte-buffer size mismatch for '{0}': expected {1}, actual {2}" -f $safe, $pin.expected_bytes, $length)
    }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        $digest = $hasher.ComputeHash($bytes)
    }
    finally {
        $hasher.Dispose()
    }
    $hex = [Convert]::ToHexString($digest).ToLowerInvariant()
    if($hex -ne $pin.expected_sha256) {
        throw ("byte-buffer SHA-256 mismatch for '{0}': expected {1}, actual {2}" -f $safe, $pin.expected_sha256, $hex)
    }
    return ,$bytes
}

function Get-JsonDocument([string]$RelativePath) {
    $bytes = Read-VerifiedBytes $RelativePath
    $utf8 = [Text.UTF8Encoding]::new($false, $true)
    $text = $utf8.GetString($bytes)
    try {
        return ConvertFrom-Json -InputObject $text -ErrorAction Stop
    }
    catch {
        throw "malformed JSON in '$RelativePath': $($_.Exception.Message)"
    }
}

function Get-TextDocument([string]$RelativePath) {
    $bytes = Read-VerifiedBytes $RelativePath
    $utf8 = [Text.UTF8Encoding]::new($false, $true)
    return $utf8.GetString($bytes)
}

function Add-Pin([string]$Id, [string]$Category, [string]$Source, [object]$Path, [object]$Bytes, [object]$Sha256) {
    $safe = ConvertTo-SafeRelativePath ([string]$Path)
    $numericBytes = $null
    if($null -ne $Bytes) {
        $numericBytes = [int64]$Bytes
        if($numericBytes -lt 0) { throw "negative byte count for '$safe'" }
    }
    $shaText = [string]$Sha256
    if($shaText -notmatch '^[0-9a-f]{64}$') { throw "invalid lowercase sha256 for '$safe'" }

    Inc-Stat 'records_observed'
    if($script:ByPath.ContainsKey($safe)) {
        $existing = $script:ByPath[$safe]
        $conflict = $false
        if($existing.expected_sha256 -ne $shaText) { $conflict = $true }
        if($null -ne $existing.expected_bytes -and $null -ne $numericBytes -and $existing.expected_bytes -ne $numericBytes) { $conflict = $true }
        if($null -eq $existing.expected_bytes -and $null -ne $numericBytes) { $conflict = $true }
        if($null -ne $existing.expected_bytes -and $null -eq $numericBytes) { $conflict = $true }
        if($conflict) {
            Inc-Stat 'conflicting_pins'
            Add-Error "conflicting pins for '$safe'"
            return
        }
        [void]$existing.sources.Add($Source)
        Inc-Stat 'duplicate_records_merged'
        return
    }
    $sources = [Collections.Generic.List[string]]::new()
    [void]$sources.Add($Source)
    $row = [ordered]@{
        id = $Id
        category = $Category
        sources = $sources
        path = $safe
        expected_bytes = $numericBytes
        actual_bytes = $null
        expected_sha256 = $shaText
        actual_sha256 = $null
        status = 'NOT_CHECKED'
    }
    $record = [pscustomobject]$row
    [void]$script:Checks.Add($record)
    $script:ByPath[$safe] = $record
}

function Add-PinChecked([string]$Id, [string]$Category, [string]$Source, [object]$Path, [object]$Bytes, [object]$Sha256) {
    try {
        Add-Pin -Id $Id -Category $Category -Source $Source -Path $Path -Bytes $Bytes -Sha256 $Sha256
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "malformed pin from '$Source': $($_.Exception.Message)"
    }
}

function Test-PinRecord([object]$Record) {
    try {
        $full = Resolve-InputPath $Record.path
    }
    catch {
        $Record.status = 'MALFORMED_PATH'
        Inc-Stat 'malformed'
        Add-Error "path resolution failed for '$($Record.path)': $($_.Exception.Message)"
        return
    }

    if(-not [IO.File]::Exists($full)) {
        try {
            $probe = [IO.Path]::GetDirectoryName($full)
            $previous = ''
            while(-not [string]::IsNullOrEmpty($probe) -and -not [IO.Directory]::Exists($probe) -and $probe -ne $previous) {
                $previous = $probe
                $probe = [IO.Path]::GetDirectoryName($probe)
            }
            if(-not [string]::IsNullOrEmpty($probe)) { Assert-NoReparseChain $probe }
        }
        catch {
            $Record.status = 'REPARSE_REJECTED'
            Inc-Stat 'reparse_rejected'
            Add-Error "reparse rejection near missing pin '$($Record.path)': $($_.Exception.Message)"
            return
        }
        $Record.status = 'MISSING'
        Add-Error "missing pin '$($Record.path)'"
        return
    }

    try {
        Assert-NoReparseChain $full
    }
    catch {
        $Record.status = 'REPARSE_REJECTED'
        Inc-Stat 'reparse_rejected'
        Add-Error "reparse rejection for '$($Record.path)': $($_.Exception.Message)"
        return
    }

    try {
        $digest = Get-FileDigest $full
        $Record.actual_bytes = $digest.bytes
        $Record.actual_sha256 = $digest.sha256
        if($null -ne $Record.expected_bytes -and $digest.bytes -ne $Record.expected_bytes) {
            $Record.status = 'SIZE_MISMATCH'
        }
        elseif($digest.sha256 -ne $Record.expected_sha256) {
            $Record.status = 'SHA_MISMATCH'
        }
        else {
            $Record.status = 'MATCH'
        }
        if($Record.status -ne 'MATCH') {
            Add-Error ("{0} for '{1}': expected {2}/{3}, actual {4}/{5}" -f $Record.status, $Record.path, $Record.expected_bytes, $Record.expected_sha256, $digest.bytes, $digest.sha256)
        }
    }
    catch {
        $Record.status = 'READ_ERROR'
        Add-Error "read error for '$($Record.path)': $($_.Exception.Message)"
    }
}

function Add-HardenAuthorityPins([object]$Freeze) {
    $pins = @($Freeze.authority.pins)
    $receipts = @($Freeze.authority.receipts)
    $script:ScopeCounts.harden_authority_pins = $pins.Count
    $script:ScopeCounts.harden_authority_receipts = $receipts.Count
    if($pins.Count -ne 5) {
        Inc-Stat 'malformed'
        Add-Error "harden FREEZE authority.pins count is $($pins.Count), expected 5"
    }
    if($receipts.Count -ne 4) {
        Inc-Stat 'malformed'
        Add-Error "harden FREEZE authority.receipts count is $($receipts.Count), expected 4"
    }
    for($i = 0; $i -lt $pins.Count; $i++) {
        try {
            $pin = $pins[$i]
            Add-PinChecked -Id ("harden.authority.pin[{0}]" -f $i) -Category 'harden_authority_pin' -Source ("harden_freeze.authority.pins[{0}]" -f $i) -Path $pin.path -Bytes $pin.bytes -Sha256 $pin.sha256
        }
        catch {
            Inc-Stat 'malformed'
            Add-Error "malformed harden authority pin $($i): $($_.Exception.Message)"
        }
    }
    for($i = 0; $i -lt $receipts.Count; $i++) {
        try {
            $receipt = $receipts[$i]
            Add-PinChecked -Id ("harden.authority.receipt[{0}]" -f $i) -Category 'harden_authority_receipt' -Source ("harden_freeze.authority.receipts[{0}]" -f $i) -Path $receipt.path -Bytes $receipt.bytes -Sha256 $receipt.sha256
        }
        catch {
            Inc-Stat 'malformed'
            Add-Error "malformed harden authority receipt $($i): $($_.Exception.Message)"
        }
    }
}

function Add-AccessManifests([object]$Freeze) {
    $properties = @($Freeze.manifests.PSObject.Properties)
    $script:ScopeCounts.access_manifests = $properties.Count
    if($properties.Count -ne 16) {
        Inc-Stat 'malformed'
        Add-Error "corpus-access FREEZE manifests count is $($properties.Count), expected 16"
    }
    for($i = 0; $i -lt $properties.Count; $i++) {
        try {
            $property = $properties[$i]
            $entry = $property.Value
            Add-PinChecked -Id ("access.manifest[{0}]" -f $i) -Category 'access_manifest' -Source ("access_freeze.manifests:{0}" -f $property.Name) -Path $property.Name -Bytes $entry.bytes -Sha256 $entry.sha256
        }
        catch {
            Inc-Stat 'malformed'
            Add-Error "malformed corpus-access manifest $($i): $($_.Exception.Message)"
        }
    }
}

function Add-FrozenCodePins([string]$RelativePath) {
    $text = Get-TextDocument $RelativePath
    $normalized = $text.Replace("`r`n", "`n")
    if($normalized.EndsWith("`n")) { $normalized = $normalized.Substring(0, $normalized.Length - 1) }
    $lines = @($normalized.Split([char]10))
    $script:ScopeCounts.frozen_code_rows = $lines.Count
    if($lines.Count -ne 32) {
        Inc-Stat 'malformed'
        Add-Error "frozen code manifest row count is $($lines.Count), expected 32"
    }
    for($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        if($line -notmatch '^(.+?)  ([0-9]+)  ([0-9a-f]{64})$') {
            Inc-Stat 'malformed'
            Add-Error "malformed frozen code manifest row $($i + 1)"
            continue
        }
        $rowPath = $Matches[1]
        $rowBytes = [int64]$Matches[2]
        $rowSha = $Matches[3]
        Add-PinChecked -Id ("frozen.code[{0}]" -f $i) -Category 'frozen_code' -Source ("frozen_code_manifest row {0}" -f ($i + 1)) -Path $rowPath -Bytes $rowBytes -Sha256 $rowSha
    }
}

function Add-Source131Pins([object]$Doc) {
    $files = @($Doc.files)
    $script:ScopeCounts.source131_files = $files.Count
    if($files.Count -ne 131) {
        Inc-Stat 'malformed'
        Add-Error "source131 file count is $($files.Count), expected 131"
    }
    if([int64]$Doc.count -ne 131) {
        Inc-Stat 'malformed'
        Add-Error "source131 declared count is $($Doc.count), expected 131"
    }
    if([int64]$Doc.mismatched -ne 0) {
        Inc-Stat 'malformed'
        Add-Error "source131 mismatched count is $($Doc.mismatched), expected 0"
    }
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $total = [int64]0
    for($i = 0; $i -lt $files.Count; $i++) {
        try {
            $file = $files[$i]
            $filePath = [string]$file.path
            if([string]::IsNullOrEmpty($filePath)) { throw 'empty source path' }
            if(-not $seen.Add($filePath)) {
                Inc-Stat 'malformed'
                Add-Error "duplicate source131 path '$filePath'"
                continue
            }
            $fileBytes = [int64]$file.bytes
            $fileSha = [string]$file.sha256
            $declaredSha = [string]$file.declared_sha256
            if($fileSha -ne $declaredSha) {
                Inc-Stat 'malformed'
                Add-Error "source131 declared_sha256 mismatch for '$filePath'"
            }
            if($file.matches -ne $true) {
                Inc-Stat 'malformed'
                Add-Error "source131 matches is not true for '$filePath'"
            }
            Add-PinChecked -Id ("source131[{0}]" -f $i) -Category 'source131' -Source 'source131_receipt.files' -Path $filePath -Bytes $fileBytes -Sha256 $fileSha
            $total += $fileBytes
        }
        catch {
            Inc-Stat 'malformed'
            Add-Error "malformed source131 row $($i): $($_.Exception.Message)"
        }
    }
    if($total -ne [int64]$Doc.total_bytes) {
        Inc-Stat 'malformed'
        Add-Error "source131 total bytes $total do not match declared total $($Doc.total_bytes)"
    }
}

function Add-HistoricalTestEvidence([string]$RelativePath) {
    $text = Get-TextDocument $RelativePath
    $normalized = $text.Replace("`r`n", "`n")
    $lines = @($normalized.Split([char]10))
    $passed = 0
    $failed = 0
    $ignored = 0
    $resultLines = 0
    $badVerdict = 0
    $shapeErrors = 0
    foreach($line in $lines) {
        if($line -match '^test result: ([A-Za-z]+)\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored; ') {
            $resultLines++
            if($Matches[1] -ne 'ok') { $badVerdict++ }
            $passed += [int]$Matches[2]
            $failed += [int]$Matches[3]
            $ignored += [int]$Matches[4]
        }
        elseif($line -match '^test result:') {
            $shapeErrors++
        }
    }
    $ok = ($resultLines -gt 0) -and ($passed -eq 214) -and ($failed -eq 0) -and ($ignored -eq 1) -and ($badVerdict -eq 0) -and ($shapeErrors -eq 0)
    $statusText = 'MALFORMED'
    if($ok) {
        $statusText = 'HISTORICAL_REGRESSIONS_REUSED'
    }
    else {
        Inc-Stat 'malformed'
        Add-Error ("historical test stdout parse: results={0} passed={1} failed={2} ignored={3} badVerdict={4} shapeErrors={5}" -f $resultLines, $passed, $failed, $ignored, $badVerdict, $shapeErrors)
    }
    $script:Historical = [ordered]@{
        source = $RelativePath
        status = $statusText
        newly_executed = $false
        parsed = [ordered]@{
            result_lines = $resultLines
            passed = $passed
            failed = $failed
            ignored = $ignored
        }
        expected = [ordered]@{
            passed = 214
            failed = 0
            ignored = 1
        }
        ok = $ok
    }
}

function Complete-Run([string]$Status, [int]$ExitCode) {
    $matched = 0
    $missing = 0
    $drift = 0
    $notChecked = 0
    foreach($record in $script:Checks) {
        switch($record.status) {
            'MATCH' { $matched++ }
            'MISSING' { $missing++ }
            'NOT_CHECKED' { $notChecked++ }
            'REPARSE_REJECTED' { }
            default { $drift++ }
        }
    }
    $counts = [ordered]@{
        records_observed = [int]$script:Stats.records_observed
        unique_paths = $script:Checks.Count
        duplicate_records_merged = [int]$script:Stats.duplicate_records_merged
        conflicting_pins = [int]$script:Stats.conflicting_pins
        checked = $script:Checks.Count - $notChecked
        matched = $matched
        missing = $missing
        drift = $drift
        reparse_rejected = [int]$script:Stats.reparse_rejected
        malformed = [int]$script:Stats.malformed
    }
    $checkRows = [Collections.Generic.List[object]]::new()
    foreach($record in $script:Checks) {
        $row = [ordered]@{
            id = $record.id
            category = $record.category
            sources = @($record.sources)
            path = $record.path
            expected_bytes = $record.expected_bytes
            actual_bytes = $record.actual_bytes
            expected_sha256 = $record.expected_sha256
            actual_sha256 = $record.actual_sha256
            status = $record.status
        }
        [void]$checkRows.Add([pscustomobject]$row)
    }
    $result = [ordered]@{
        schema = $script:Schema
        task = $script:Task
        status = $Status
        mode = 'READ_ONLY_NO_SUBPROCESS_PINNED_BYTE_INTEGRITY'
        repository_root = $script:Root
        script = 'tools/re/verify-preservation.ps1'
        script_sha256 = $script:ScriptSha256
        scope = [ordered]@{
            RUST_TESTS_NEWLY_EXECUTED = 0
            semantic_status = 'NOT_RECHECKED'
            full_research_status = 'NOT_RECHECKED'
            originals34_content_status = 'NOT_RECHECKED'
            graph_764mb_status = 'NOT_READ'
            index_rebuild_status = 'NOT_PERFORMED'
            full_corpus_rehash_status = 'NOT_PERFORMED'
        }
        scope_counts = $script:ScopeCounts
        counts = $counts
        root_chain_recheck_stable = $script:RootRecheckStable
        checks = @($checkRows)
        historical_regressions = $script:Historical
        errors = @($script:Errors)
        limitations = @(
            'Pinned-byte integrity only; no semantic, build, runtime, provenance-activation, or whole-corpus claim.'
            'Rust tests are not executed; the historical 214 passed / 0 failed / 1 ignored receipt is parsed, not rerun.'
            'The 34 original research files and the 764 MB graph are not rehashed; the existing index is not rebuilt.'
            'This read-only entrypoint cannot prevent a separate shell command from deleting files; it is not an OS sandbox.'
        )
    }
    $json = ConvertTo-Json -InputObject $result -Depth 8 -Compress
    [Console]::Out.Write($json + "`n")
    exit $ExitCode
}

try {
    if($PSVersionTable.PSVersion.Major -lt 7) { throw 'PowerShell 7 or newer is required' }

    $script:Root = [IO.Path]::GetFullPath([string]$RepositoryRoot)
    if(-not [IO.Directory]::Exists($script:Root)) { throw "repository root not found: $script:Root" }
    Assert-NoReparseChain $script:Root

    $selfPath = [IO.Path]::GetFullPath([string]$PSCommandPath)
    Assert-NoReparseChain $selfPath
    $selfDigest = Get-FileDigest $selfPath
    $script:ScriptSha256 = $selfDigest.sha256

    Add-Pin -Id 'root.harden_primary_final' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/repo-harden01/reports/final/PRIMARY_FINAL.json' -Bytes 20459 -Sha256 'b38d6abb82660ec5fc3094c5c8bc4341b7fd815865adcb7a625b044e69ec5c28'
    Add-Pin -Id 'root.harden_freeze' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/repo-harden01/reports/final/FREEZE.json' -Bytes 9307 -Sha256 '9ab5b2565e82465c2e2f2f1d8a7058d84a5633e165864dce1424cc999aa6a14c'
    Add-Pin -Id 'root.access_freeze' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/corpus-access01/reports/final/FREEZE.json' -Bytes 6840 -Sha256 '2eb6e70972585c11637486b00f6cec1030fac1a55f7bd5b385a7838802c0a52d'
    Add-Pin -Id 'root.source131_receipt' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/corpus-access01/reports/final/SOURCE131_HASHES_FINAL.json' -Bytes 56844 -Sha256 'bff5714f14f8b958375e1b7622aca265b72ef2654c6cc72245b57a1b81f56da2'
    Add-Pin -Id 'root.frozen_code_manifest' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/repo-harden01/reports/final/FROZEN_CODE_MANIFEST.txt' -Bytes $null -Sha256 'ed7c0016148726ea3b64591773321b303b84104b75062e389d149f8c5c3187d2'
    Add-Pin -Id 'root.final_test_stdout' -Category 'root_chain' -Source 'task_pin' -Path 'tools/re/local-recovery/repo-harden01/reports/final/FINAL_TEST.stdout.txt' -Bytes 20785 -Sha256 '68be7bcc68df001ed3b350749b8fa8c5be2112b411db22019fe47759349030c0'

    foreach($record in $script:Checks) {
        if($record.category -eq 'root_chain') { [void]$script:RootPins.Add($record) }
    }

    foreach($record in $script:RootPins) { Test-PinRecord -Record $record }
    $rootBad = 0
    foreach($record in $script:RootPins) {
        if($record.status -ne 'MATCH') { $rootBad++ }
    }
    if($rootBad -gt 0) {
        $script:RootRecheckStable = $false
        Complete-Run -Status 'FAIL' -ExitCode 2
    }

    try {
        $hardenFreeze = Get-JsonDocument 'tools/re/local-recovery/repo-harden01/reports/final/FREEZE.json'
        Add-HardenAuthorityPins -Freeze $hardenFreeze
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "harden FREEZE enumeration failed: $($_.Exception.Message)"
    }

    try {
        $accessFreeze = Get-JsonDocument 'tools/re/local-recovery/corpus-access01/reports/final/FREEZE.json'
        Add-AccessManifests -Freeze $accessFreeze
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "corpus-access FREEZE enumeration failed: $($_.Exception.Message)"
    }

    try {
        Add-FrozenCodePins 'tools/re/local-recovery/repo-harden01/reports/final/FROZEN_CODE_MANIFEST.txt'
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "frozen code manifest enumeration failed: $($_.Exception.Message)"
    }

    try {
        $sourceDoc = Get-JsonDocument 'tools/re/local-recovery/corpus-access01/reports/final/SOURCE131_HASHES_FINAL.json'
        Add-Source131Pins -Doc $sourceDoc
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "source131 enumeration failed: $($_.Exception.Message)"
    }

    try {
        Add-HistoricalTestEvidence 'tools/re/local-recovery/repo-harden01/reports/final/FINAL_TEST.stdout.txt'
    }
    catch {
        Inc-Stat 'malformed'
        Add-Error "historical test receipt parse failed: $($_.Exception.Message)"
    }

    foreach($record in $script:Checks) {
        if($record.status -eq 'NOT_CHECKED') { Test-PinRecord -Record $record }
    }

    $script:RootRecheckStable = $true
    foreach($record in $script:RootPins) {
        try {
            $full = Resolve-InputPath $record.path
            Assert-NoReparseChain $full
            $digest = Get-FileDigest $full
            if($digest.bytes -ne $record.actual_bytes -or $digest.sha256 -ne $record.actual_sha256) {
                $script:RootRecheckStable = $false
                Add-Error "root chain changed during verification: '$($record.path)'"
            }
        }
        catch {
            $script:RootRecheckStable = $false
            Add-Error "root chain recheck failed for '$($record.path)': $($_.Exception.Message)"
        }
    }

    $matched = 0
    $notChecked = 0
    $missing = 0
    $drift = 0
    foreach($record in $script:Checks) {
        switch($record.status) {
            'MATCH' { $matched++ }
            'MISSING' { $missing++ }
            'NOT_CHECKED' { $notChecked++ }
            'REPARSE_REJECTED' { }
            default { $drift++ }
        }
    }
    $pass = ($script:Checks.Count -gt 0) -and ($matched -eq $script:Checks.Count) -and ($notChecked -eq 0) -and ($missing -eq 0) -and ($drift -eq 0) -and ([int]$script:Stats.reparse_rejected -eq 0) -and ([int]$script:Stats.malformed -eq 0) -and ([int]$script:Stats.conflicting_pins -eq 0) -and ($script:RootRecheckStable -eq $true) -and ($script:Errors.Count -eq 0)
    $statusText = 'FAIL'
    $exitCode = 2
    if($pass) {
        $statusText = 'PASS'
        $exitCode = 0
    }
    Complete-Run -Status $statusText -ExitCode $exitCode
}
catch {
    Add-Error "fatal: $($_.Exception.Message)"
    $script:RootRecheckStable = $false
    Complete-Run -Status 'ERROR' -ExitCode 2
}