# Project and documentation guide

This is a static-only, source-like reconstruction of selected Wuthering Waves
managed, script, and native-boundary surfaces. It is not original developer
source, a client dump, a whole-program recovery, a buildable game, a runtime
result, or a publication approval. A developer can use it to find accepted
source-like units, understand what was recovered, inspect the private-input
boundaries, and reach the checks and worked examples. Start with
[SOURCE_INDEX.md](SOURCE_INDEX.md); that page is the file-by-file inventory, and
this page is navigation only.

## Finding source and methods

The curated tree has 131 files. The accepted managed baseline is 104 C# files
and 533 qualified identities. One supplemental C# view adds four keys, for a
reported 105/537 candidate; the baseline is not silently replaced. The other 26
files are TypeScript, JavaScript, native C++ models, and declarations. A selected
method, not a whole class, is the recovered unit. Partial class views, missing
dependencies, and source-like projections are normal here; the tree is not one
build project and is not buildable as a whole.

- [SOURCE_INDEX.md](SOURCE_INDEX.md) gives each file's record, subsystem,
  ordinary and associated method counts, full/source/unknown evidence counts,
  dependency boundary, related views, and proof catalog.
- [RESEARCH_FINDINGS.md](RESEARCH_FINDINGS.md) is the compact register of six
  bounded findings, stable IDs, authority scope, and evidence limits.
- Each curated C# view carries a header and token comments with the assembly
  SHA-256, MVID, original TypeDef, selected MethodDefs, and source anchors. Use
  the full managed key; a filename or class name alone is not an identity.
- [tools/re/README.md](../tools/re/README.md) documents the read-only status and
  lookup commands. `re status`, `re lookup`, and `source-status` work from the
  curated metadata. `source-lookup` for accepted imports, `source-verify`,
  `source-inventory`, and the indexed `source-locate` query need private
  research originals or the private derived index. A clean checkout cannot run
  the full index-backed lookup.
- The included typed catalogs retain the accepted source identities and
  qualifications. The public
  [source index](SOURCE_INDEX.md) is navigation, not a replacement for those
  records.
- Subsystem notes: [PERFORMANCE_TIMER.md](PERFORMANCE_TIMER.md) and
  [TASK_SCHEDULING.md](TASK_SCHEDULING.md).
- The [damage snapshot walkthrough](DAMAGE_SNAPSHOT_FLOW.md) maps five additional
  `SOURCE_ONLY` excerpts, snapshot capture before conditional attacker replacement,
  and gated response callbacks. It also documents the separate authored timer
  lifecycle experiment and its assumptions.

## Checks, examples, and tools

- [tools/re/README.md](../tools/re/README.md) describes the Rust tools, their
  read-only command surface, and the preservation verifier. The verifier checks
  pinned bytes; it is not a semantic or build proof.
- The [research-checks workflow](../.github/workflows/research-checks.yml) runs
  the source-bound timer model and a clean-checkout structural status check; it
  does not run private-input or full Cargo test suites.
- [tests/README.md](../tests/README.md) separates Rust regressions, the
  recovered C# harnesses, later transformer and skill harnesses, and the
  separate synthetic Event 101 model.
- The [gameplay showcase](../examples/mechanics/gameplay-showcase/README.md)
  and its [evidence reference](../examples/mechanics/gameplay-showcase/EVIDENCE.md)
  follow three separate pieces: buff and attribute mutation, snapshot capture,
  and formula-side attribute access. They are not a tested end-to-end route.
- Historical harness documentation has separate editorial successors:
  [buff and attribute](history/harnesses/buff-attribute/README.md),
  [attribute snapshot](history/harnesses/attribute-snapshot/README.md), and
  [damage formula](history/harnesses/damage-formula/README.md) with its
  [historical result](history/harnesses/damage-formula/RESULTS.md). The original
  preparation records remain distinct; the successors do not make their private
  receipts reproducible.
- The [skill decision reference](../examples/mechanics/skill-decision-reference/README.md)
  is an independently authored .NET 8/BCL model of interruption, cooldown
  availability, and charge completion; its controlled checkpoint passed 20 of 20
  cases.
- The [damage finalization reference](../examples/mechanics/damage-reference/README.md)
  is an independently authored .NET 8/BCL numeric and seed-order diagnostic;
  its accepted offline checkpoint passed 10 of 10 cases and 101 of 101 assertions.
  Its README remains a frozen preparation checkpoint, so its `NOT RUN` text
  predates that accepted run. The execution receipts for these model checkpoints
  are not distributed here; this documentation update did not rerun them.
  Neither model establishes game equivalence or a verified clean-clone build.
- The damage formula [historical result](history/harnesses/damage-formula/RESULTS.md)
  records an accepted execution result for nine saved bodies. It is not a full
  formula, integration, or game-runtime proof.
- [CONTRIBUTING.md](../CONTRIBUTING.md) summarizes public writing, evidence,
  review, and link rules. Build and test authorization is separate; see the
  [reproduction boundaries](REPRODUCIBILITY.md) and
  [tool interface](../tools/re/README.md).

## Boundaries

The curated tree, metadata and 18 qualified historical files are included.
The [gameplay showcase](../examples/mechanics/gameplay-showcase/README.md#qualified-historical-source-map)
links seven selected fixture views and two fuller class captures. Their presence
does not make the private canonical corpus, graph, index, execution receipts or
pinned toolchain available. Full-check reproduction is not a clean-clone workflow.
Runtime remains parked, game version is unknown and full backup coverage is not
established. Selected response-field copies and gated callbacks are documented;
an end-to-end buff-to-damage route, an HP-writing consumer, server validation or
replication, native ABI and behavioral equivalence remain unproved. This is a selected documentation view,
not a rights-cleared release; see the legal notice in the [README](../README.md).

## Historical provenance

[Publication provenance](PUBLICATION_PROVENANCE.json) records an earlier
publication-selection snapshot. It describes that preparation checkpoint and
does not certify the current checkout, the present Markdown set, or later
editorial changes. Historical preparation and test results remain bound to their
own named scopes; they are not fresh checks.
