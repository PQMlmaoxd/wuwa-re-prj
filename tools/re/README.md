# tools/re developer interface

`tools/re` contains the local Rust tools for validated metadata, source
closure, registry, and preservation checks. It is a bounded maintainer
interface, not a general extractor, a source export tool, a decompiler, or a
replacement for the private research pipeline.

Read the [reproducibility boundaries](../../docs/REPRODUCIBILITY.md) and
[test guidance](../../tests/README.md) before running tests or private workflows.
Several test surfaces create and remove fixture trees, including paths under
the manifest directory. An external Cargo target directory does not isolate all
fixture writes or cleanup.

## Source layout

- [Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock) define the package and
  pinned dependency graph. The manifest declares Rust 1.88, edition 2024, and
  exact dependency versions; the crates are not vendored. Those are manifest
  facts, not a claim that a build ran here.
- [src/main.rs](src/main.rs) starts the `re` binary.
- [src/lib.rs](src/lib.rs) contains the command dispatch and shared validation
  code.
- [src/registry.rs](src/registry.rs) and
  [src/registry/source_closure.rs](src/registry/source_closure.rs) implement the
  `corpus-registry` commands.
- [src/bin/corpus-registry.rs](src/bin/corpus-registry.rs) starts that binary.
- [src/bin/curate-batch.rs](src/bin/curate-batch.rs) is a mutating curation entry
  point, not a read-only verification command.
- [src/provenance.rs](src/provenance.rs) and [src/bulk.rs](src/bulk.rs) contain
  the provenance and bulk-curation checks.
- [tests/](tests/) contains the Rust regression tests, including validation,
  registry, provenance, and bulk suites.

## Finding a method from this repository

For a curated method, start with [SOURCE_INDEX.md](../../docs/SOURCE_INDEX.md)
and the header and token comments in the curated C# view. Those comments carry
the assembly SHA-256, MVID, original TypeDef, selected MethodDefs, source
anchors, and historical evidence labels. Use the full managed key; a filename,
class name, or display name is not an identity.

`re status`, `re lookup`, and `corpus-registry source-status` work from the
curated metadata. `corpus-registry source-lookup` for exact accepted imports,
`source-verify`, `source-inventory`, and `source-locate` need private research
originals, the pinned graph, or the private derived index. They are maintainer
operations, not clean-checkout source browsing. `source-locate` in particular
does not fall back to a local index or a different workspace.


## `re` commands

The `re` binary accepts four read-only commands. The documented forms are:

```text
re status [--curated-root PATH] [--json]
re lookup <authority-id> <MethodDef> [--curated-root PATH] [--json]
re verify [--curated-root PATH] [--research-root PATH] [--json]
re verify-provenance [--curated-root PATH] [--research-root PATH] [--json]
```

`status` reads the validated manifest plus active typed catalogs and the active
receipt. `lookup` requires an exact authority ID and MethodDef together. `verify`
performs byte-integrity checks; without a research root it reports missing or
unchecked items rather than a clean pass. `verify-provenance` adds the effective
provenance view and reports `BLOCKED` when the private research input is absent.

All four are read-only. They do not extract, rewrite, migrate, or write a file.
Exit `0` means found or clean within scope, `1` means a finding or not-found
result, and `2` means a usage, schema, or execution error. The JSON envelope is
written to standard output.

## `corpus-registry` commands

The `corpus-registry` binary adds source-closure and indexed-access commands. The
argument sets implemented in `src/registry.rs` are:

```text
source-status [--manifest PATH] [--curated-root PATH]
source-lookup [--manifest PATH] [--origin-id ID] [--function-id ID]
              [--artifact-root-id ID] [--artifact-path PATH]
              [--assembly-sha SHA256] [--mvid UUID]
              [--type 0x02......] [--method 0x06......]
              [--research-root PATH] [--curated-root PATH]
source-verify [--manifest PATH] [--research-root PATH] [--curated-root PATH]
source-inventory [--manifest PATH] [--research-root PATH] [--curated-root PATH]
source-access-index [--manifest PATH] [--output PATH]
                    [--curated-root PATH] [--research-root PATH]
source-locate [--manifest PATH] [--authority ID] [--mvid UUID]
              [--type 0x02......] [--method 0x06......]
              [--research-root PATH] [--curated-root PATH] [--index PATH]
```

`source-status` is structural only. `source-lookup` resolves accepted origin,
function, artifact, or full managed-key selectors. `source-verify` checks source
and destination bytes, deterministic selection, proof anchors, and the accepted
closure partitions. `source-inventory` recomputes the exact family and file
sets. These commands report semantic status separately from byte checks.

`source-locate` is the indexed query path. It requires the compiled index pin,
a selected graph row, and the private research-backed original and import views.
A missing or mismatched index is an explicit failure such as `INDEX_MISSING`; it
does not fall back to another workspace. The existing index is the accepted
derived artifact. This page does not recommend rebuilding it, and an index
rebuild is not part of a documentation or preservation check.

`source-access-index` writes a new derived index. It requires private inputs and
is not a read-only query.

The `scan`, `verify`, `graph-build`, `graph-verify`, and `graph-lookup` commands
also exist in the parser for private graph work. They are outside the public
navigation path described here. `scan` and `graph-build` write outputs;
`graph-verify` also writes a scratch reconstruction.

## Preservation verifier

[verify-preservation.ps1](verify-preservation.ps1) is a read-only PowerShell
entry point with one declared parameter, `RepositoryRoot`. Run the command from
the repository root; the `tools/re/...` path below is repository-root relative.
When `RepositoryRoot` is omitted, the script resolves its default root from its
own `tools/re` directory to the repository root (`../../`). Its documented forms
are:

```text
pwsh -NoProfile -NonInteractive -File tools/re/verify-preservation.ps1
pwsh -NoProfile -NonInteractive -File tools/re/verify-preservation.ps1 -RepositoryRoot <repository>
```

It verifies a fixed chain of pins: the 131 source files, the 32-row frozen code
manifest, the accepted authority pins and receipts, the accepted manifests, and
the historical test stdout. It opens files read-only, rejects reparse components,
spawns no subprocess, and writes no file.

Its result is byte integrity only. It does not establish semantics,
buildability, runtime behavior, or a fresh backup. It also does not replace the
backup contract. Its required pins are private artifacts; if a pin is missing,
size-drifted, hash-drifted, or ignored, it fails closed with a diagnostic result
and exit 2. It never edits a pin or reseals a mismatch.

## Build and test

With Rust 1.88 or newer and a prepared Cargo cache containing the locked crates,
build the configured binaries from the repository root with:

```sh
cargo build --locked --offline --manifest-path tools/re/Cargo.toml --bins
```

Dependencies are pinned but not vendored, so offline build requires that cache.
This command writes Cargo build output; it is documented here, not run for this
documentation change.

Cargo tests and recovered-harness scripts are not casual health checks. Some
test/build surfaces create or remove fixtures under the manifest directory, and
`--target-dir` alone does not redirect every fixture path. Read the
[test guidance](../../tests/README.md) and
[reproducibility boundaries](../../docs/REPRODUCIBILITY.md) for the required inputs
and filesystem effects.

The historical test record reports 215 tests: 214 active and one explicitly
ignored administrative test. That is a received historical count, not a run
performed for this page. Do not treat it as a current result.
