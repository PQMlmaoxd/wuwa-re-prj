# Reproducibility and Input Boundaries

Public-copy context: the operation matrix below retains its tool and private-input
limits. A selected publication copy has since been prepared, but that does not
implement a supported exporter or make private workflows self-contained. Seven
fixture views and two fuller captures are now included as
[qualified historical source](../examples/mechanics/gameplay-showcase/README.md#qualified-historical-source-map),
not as the private corpus, toolchain or execution receipts. See
[README legal notice](../README.md) for rights and distribution limits.

This page states what can be reproduced from the curated repository, what needs
the private research corpus, and what remains unsupported. It does not authorize
publication, export, new extraction, or a stronger evidence claim.

## Status vocabulary

- `SELF_CONTAINED` - the curated repository plus the stated toolchain and cached
  dependencies are sufficient.
- `REQUIRES_OPTIONAL_PRIVATE_CORPUS` - a structural result is available without
  private bytes, but a PASS or richer join needs them.
- `REQUIRES_MANDATORY_PRIVATE_INPUT` - private bytes, graphs, or the derived
  index are required; absence must produce an honest nonzero or NOT_CHECKED
  result, never a clean PASS.
- `UNSUPPORTED` - no reproducible path is claimed under current authority.

## Operation matrix

| Operation | Status | Inputs and limits |
| --- | --- | --- |
| Rust build, `cargo build --locked --offline --manifest-path tools/re/Cargo.toml --bins` | `SELF_CONTAINED` with toolchain | Requires a preinstalled compatible Rust toolchain (1.88 or newer; edition 2024; checked locally with 1.94.1) plus cached crates selected by `Cargo.lock`. Dependencies are pinned but not vendored, so a fresh machine without that toolchain and cargo cache cannot be assumed buildable offline. |
| Core fixture and integration tests (the 71 pre-existing tests in `validation`, `registry`, `registry_hardening`, plus 3 portable-root tests) | `SELF_CONTAINED` with compiler and cargo cache | Uses repository-owned metadata, generated fixtures under `tools/re/target`, and the 131 curated bytes. Does not require the research corpus or the private index. |
| Full `cargo test` suite | `REQUIRES_MANDATORY_PRIVATE_INPUT` | Some actual access tests require the private derived index, the 764 MB graph, the research corpus, and Windows junction support. The actual access tests that need the private index now fail with an explicit dependency assertion instead of returning early. |
| `re status`, `re lookup` | `SELF_CONTAINED` | Validated metadata plus the accepted 131 curated files. Structural and identity output only. |
| `re verify` without `--research-root` | `REQUIRES_OPTIONAL_PRIVATE_CORPUS` | Returns an honest incomplete result with `NOT_CHECKED` items; it is not a PASS. |
| `re verify --research-root ...` | `REQUIRES_MANDATORY_PRIVATE_INPUT` for PASS | Scope is `BYTE_INTEGRITY_ONLY`: it reads original source bytes for the byte check and reports evidence references as `NOT_CHECKED`. PASS does not prove provenance activation, semantics, buildability, or runtime. |
| `re verify-provenance` | `REQUIRES_MANDATORY_PRIVATE_INPUT` for PASS | Without the private research root it reports `BLOCKED`, not PASS. |
| `corpus-registry source-status` | `SELF_CONTAINED` | Structural manifest and curated-set checks. It reports `STRUCTURAL_OK` with semantic status `NOT_CHECKED`, not a semantic claim. |
| `source-lookup` exact accepted curated imports | `REQUIRES_MANDATORY_PRIVATE_INPUT` | Under the current loader, successor loading performs byte verification of all 26 research originals before the exact accepted CURATED shortcut. An empty or unavailable research root fails closed with `MISSING_PATH`; the parent/successor checks are not bypassed to manufacture a self-contained PASS. |
| `source-verify`, `source-inventory` | `REQUIRES_MANDATORY_PRIVATE_INPUT` | Need the private research originals and the pinned historical graph. `--research-root` is required. |
| `source-access-index` build | `REQUIRES_MANDATORY_PRIVATE_INPUT` | Reads the pinned 764 MB `methods.jsonl`; writes a derived 23 MB index with CreateNew semantics. It is not a new authority. |
| `source-locate` | `REQUIRES_MANDATORY_PRIVATE_INPUT` | Needs the compiled index pin, selected graph row, and research-backed original/import views. A missing index is `INDEX_MISSING`, not a fallback to another workspace. |
| Curated documentation navigation | `SELF_CONTAINED` | Public relative Markdown links resolve to included files or directories. Unbundled private evidence is identified as unavailable, not presented as a working local link. |
| Public export, packaging, or redistribution | `UNSUPPORTED` | The selected publication copy is present in this repository, but no supported exporter or rights-cleared package is established. Export and redistribution still require separate rights, license, provenance, and packaging review. |

## Historical harness documents

The [buff and attribute](history/harnesses/buff-attribute/README.md),
[attribute snapshot](history/harnesses/attribute-snapshot/README.md), and
[damage formula](history/harnesses/damage-formula/README.md) pages are editorial
successors to historical preparation documents. The
[damage execution result](history/harnesses/damage-formula/RESULTS.md) records a
historical run. None is byte-identical to its predecessor, a fresh check, or a
clean-clone reproduction; private preparation inputs and receipts remain
unavailable.

## Portable CLI roots

`re status`, `re lookup`, `re verify`, and `re verify-provenance` accept
`--curated-root PATH`.

- An explicit root always wins. If it is invalid, the command fails with exit 2
  and does not fall back to a compiled or previous workspace root.
- Without an explicit root, the command uses the current directory or its
  nearest ancestor containing `metadata/migration.json`. This supports running
  from a repository root or a subdirectory.
- A foreign working directory with no repository ancestor fails with exit 2
  unless `--curated-root` is supplied.
- The public library helpers such as `run_status()` and `run_lookup()` remain
  compile-time fixture helpers built around `CARGO_MANIFEST_DIR`. They are not
  portable CLI root selection and are not used by `run_with_args`.
- No environment variables are mutated to redirect roots. Tests use explicit
  paths or child processes.

For `corpus-registry`, `source-status` accepts `--curated-root`. When
`--manifest` is absent, every source command anchors the default manifest at
the selected curated root plus `metadata/source-closure.v1.json`. An explicit
`--manifest` keeps its previous CWD/absolute semantics and cannot override the
compiled trust pin.

## Historical harness results and private prerequisites

Historical harness receipts report 28 buff-and-attribute cases / 115 assertions,
8 attribute-snapshot groups / 351 assertions, and 8 damage-formula groups / 39
assertions. These reports describe their named runs only; they do not upgrade
source classifications, establish runtime behavior, or provide a public
clean-checkout reproduction. See the [test index](../tests/README.md) for
navigation to the editorial harness records.

The attribute-snapshot preparation script requires and hashes `AGENTS.md` from
its task workspace, and records hashes for any nested instruction files present.
Its workspace-specific generation manifests, complete run layout, pinned reference
pack, and execution receipts are not bundled. Selected fixture-source views are
included as qualified history, not as a reproducible public run. The harness scripts are
not public quickstarts and have not been changed by this documentation pass.

Some build and test surfaces create or remove fixture data; `--target-dir` alone
does not redirect every fixture path. Do not run a test, harness, or cleanup
command as a casual health check. No test, build, or cleanup command was run for
this documentation change.

## Language and semantic limits

- Two TypeScript config declarations remain known malformed and unchanged. No
  TypeScript compiler was available; no TypeScript buildability is claimed.
- Native C++ models remain opaque. Prior quality evidence records one
  syntax-only pass and three opaque/expected-fail surfaces with ABI and
  linkability unknown. This checkpoint added no native build target and no new
  syntax claim.
- JSON, byte, identity, and path checks do not prove source equivalence,
  signatures, bodies, runtime behavior, or a buildable whole program.

## Architecture evidence boundary

- **SOURCE_SUPPORTED:** the included JavaScript bridge calls `cpp.FEventSystem`
  and routes callbacks to global or entity-targeted listeners. Selected managed
  views show attribute-write ordering, skill gates, timer queues, and task-yield
  calls. These are represented source edges, not observed live execution.
- **INFERRED:** managed and scripting views describe related gameplay surfaces.
  Similar names and concepts do not establish shared ownership, a cross-language
  call path, or a migration timeline.
- **UNKNOWN:** native marshalling and ABI, a direct C# bridge, the actual Puerts
  ownership path, a C#-to-TypeScript migration timeline, TaskGraph-to-TaskSystem
  runtime binding, and native or frame-time costs.

## Historical path mapping

The table maps the 13 historical Markdown paths to their editorial successor or
the status `PRIVATE_HISTORICAL_RECORD`. The status is not a local link and does
not assert that a same-named current page has the historical bytes. The four
successors are editorial records, not byte-identical replacements.

| Historical path | Public successor or status |
| --- | --- |
| `AGENTS.md` | `PRIVATE_HISTORICAL_RECORD` |
| `RESEARCH_CHECKPOINT.md` | `PRIVATE_HISTORICAL_RECORD` |
| `docs/DOCUMENTATION_STYLE.md` | Original: `PRIVATE_HISTORICAL_RECORD`; public writing and evidence rules consolidated in [Contributing](../CONTRIBUTING.md). |
| `docs/MIGRATION_PLAN.md` | `PRIVATE_HISTORICAL_RECORD` |
| `docs/PRESERVATION_SAFETY.md` | Original: `PRIVATE_HISTORICAL_RECORD`; public input and test-cleanup limits consolidated in this guide and [tools/re](../tools/re/README.md). |
| `docs/PUBLICATION.md` | Original: `PRIVATE_HISTORICAL_RECORD`; identity, architecture and distribution limits consolidated in this guide. |
| `docs/RECOVERED_SOURCE_CLOSURE.md` | Original: `PRIVATE_HISTORICAL_RECORD`; selected transform and proof scope consolidated in the [source index](SOURCE_INDEX.md). |
| `docs/STRUCTURE.md` | `PRIVATE_HISTORICAL_RECORD` |
| `docs/WORKFLOW_STYLE.md` | Original: `PRIVATE_HISTORICAL_RECORD`; public tool contracts and contribution rules consolidated in [tools/re](../tools/re/README.md) and [Contributing](../CONTRIBUTING.md). |
| `tests/managed-recovered/attribute-snapshot/README.md` | [Attribute snapshot history](history/harnesses/attribute-snapshot/README.md) |
| `tests/managed-recovered/buff-attribute/README.md` | [Buff and attribute history](history/harnesses/buff-attribute/README.md) |
| `tests/managed-recovered/damage-formula/README.md` | [Damage formula history](history/harnesses/damage-formula/README.md) |
| `tests/managed-recovered/damage-formula/RESULTS.md` | [Damage formula results](history/harnesses/damage-formula/RESULTS.md) |

The [publication provenance record](PUBLICATION_PROVENANCE.json) is an earlier
source-selection snapshot, not this historical-document map or a new whole-copy
seal.

That record retains the original and derived identities for six source views
whose authored provenance headers were redacted, plus seven other documentation
or configuration corrections. The preserved method-source spans were unchanged;
the six whole-file hashes differ from their original records. Neither those
derivatives nor this documentation reorganization establish source/CIL equivalence.
The record's historical path strings and hashes have not been rewritten or
re-sealed. The benchmark path in `metadata/migration.json` likewise identifies a
privately preserved historical record, not a required current public page.

## Rights and distribution

The selected publication copy is present in this repository. Its presence is not
external publication, a supported exporter, or rights clearance. Rights and
redistribution remain unresolved; no public maintainer contact channel is
configured. See the [README legal notice](../README.md).

Full research-corpus backup coverage remains **NOT ESTABLISHED**. This guide
makes no complete-backup assertion. Ignoring files in `.gitignore` is not a
rights, privacy, or preservation decision.
