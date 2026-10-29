# Contributing

This repository accepts evidence-backed source proposals, curation metadata and technical documentation. It is not original developer source, a game SDK or a standalone game build. Publication preparation does not resolve redistribution rights; see the [README](README.md) legal notice.

For a public contribution, start with an included view and its metadata. Report the qualified identity, the observed discrepancy, your proposed change and the checks actually performed. Documentation proposals do not require access to private research inputs. If authentication needs unavailable originals, state that limitation rather than claiming verification. Source imports still require the evidence and approval below.

## Before a change

- Read the authoritative record and its exact original path, byte size, and source-byte SHA-256. For supplemental Performance, Scheduling, or Task Execution records, the authoritative record is the validated typed catalog rather than the frozen baseline manifest.
- Keep the authority binary hash and MVID separate from the original source-file hash.
- Use the full managed symbol key: assembly SHA-256, MVID, TypeDef, and MethodDef.
- Treat a selected method as the migration unit. Do not promote a method view into whole-class coverage.
- Keep managed, TS, and native build authorities separate. Do not join records by filename, display name, or similar shape.
- Preserve original source bytes and paths. Do not normalize, translate, merge, or silently replace them.

## Review gate

A proposed source import needs all of the following before approval:

1. The exact source path, size, and raw-byte hash are present in `metadata/migration.json`, or for a supplemental Performance, Scheduling, or Task Execution record in the validated typed catalog (`metadata/performance-source.v1.json`, `metadata/scheduling-source.v1.json`, or `metadata/task-execution-source.v1.json`) with its versioned import successor.
2. The authority identity, selected tokens, scope, completeness, and representation are explicit.
3. The import is an exact-byte copy first. Any editorial change is a separate, approved diff.
4. Generic constraints, original identifiers, signatures, branches, boxing, conversions, exception regions, lifetimes, and original string literals are preserved when observed.
5. Unknown dependencies and unresolved behavior remain marked UNKNOWN. No compile stub is added to make a fragment look complete.
6. Duplicate or overlap claims use a qualified same-authority method scope. A source-only historical view is not deleted because a selected method view exists.
7. Validation states distinguish source hash, token scope, compilation, equivalence, runtime, and publication review.
8. A handoff names the exact files, hashes, tokens, proof levels, unknowns, tests, supersession limits, and next action.
9. A supplemental Performance, Scheduling, or Task Execution record does not mutate `metadata/migration.json`; it is authorized only through the validated versioned catalog and receipt.
10. A Scheduling or Task Execution record may carry the immutable root TypeDef plus an explicit catalog-scoped associated-type list for proven nested declaring types. This does not weaken the frozen baseline manifest validator or force associated methods to the root type; each method retains its actual declaring TypeDef and MethodDef.

## Evidence and privacy

Use OBSERVED, INFERRED, RECONSTRUCTED, RUNTIME_VERIFIED, and UNKNOWN consistently. A decompiler result is not original developer source. A successful compile is not proof of CIL equivalence. Matching and equivalence are NOT_ESTABLISHED unless a separately accepted proof says otherwise.

Do not add original game binaries, extracted assets, raw CIL or logs, private absolute paths, credentials, secrets, or agent and session identifiers. The `.gitignore` file is local-noise defense only; it is not a license, secrecy policy, authority record, or fail-closed governance rule.

Rights and provenance review is independent of technical readiness. No license is inferred from benchmark repositories or their documentation. Publication remains disabled until the governing review is complete.

## Documentation changes

- Keep one clear purpose per page and one level-one heading. Use direct language
  and state what is known, what supports it, and what remains unknown.
- Distinguish current repository contents from historical checkpoints. A
  historical `PASS` is not a new run, and inclusion of a source view does not
  authenticate its original binary or prove runtime behavior.
- Use relative links with exact case only to files included in the intended
  public surface. Do not link to unbundled private inputs, scratch outputs or
  excluded historical documents. Selected historical source and evidence links
  must resolve and retain their qualifications.
- Do not invent commands, test outcomes, source evidence, or a public stand-in
  for unavailable bytes. Keep stable method identities and state the limitation
  when supporting evidence is not distributed.
- Keep original source, metadata, accepted receipts, and historical records
  unchanged. Put a reviewed editorial successor in a separate file rather than
  rewriting accepted history.
- Before editing an existing page, retain and verify an exact byte backup. After
  editing, review the whole page, its local links, headings, code fences, claims,
  and any private-path references.

## Rights and contact

Rights review is separate from technical review. No blanket license or
redistribution permission is implied for game-derived material. The current
rights and contact statement is in the [README](README.md).

For test and reproduction limits, see [tests](tests/README.md) and
[reproducibility boundaries](docs/REPRODUCIBILITY.md).
