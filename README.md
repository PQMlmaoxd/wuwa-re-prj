# Wuthering Waves Gameplay RE

Independent, non-commercial reverse-engineering research into selected Wuthering
Waves gameplay and engine-boundary systems. This repository contains recovered
and reconstructed expressions, source-like views, authored models, and evidence
records. It is not original developer source, complete game source, a game SDK,
a buildable or playable replacement, or proof of runtime or server equivalence.

## Repository scope

The curated source scope is 131 files: 105 C#, 14 TypeScript, 5 JavaScript, 4 C++,
and 3 headers. The [source index](docs/SOURCE_INDEX.md) maps selected methods to
their evidence qualifications and files.

Separate from that 131-file scope are 18 qualified historical files under
[qualified-historical](qualified-historical/) and five additional `SOURCE_ONLY`
method excerpts in the [damage snapshot view](qualified-research/damage-snapshots/BaseDamageComponent.SourceOnly.cs).
Three independently authored [damage](examples/mechanics/damage-reference/README.md),
[skill-decision](examples/mechanics/skill-decision-reference/README.md), and
[event](examples/mechanics/event101-reference/README.md) references, and the
[timer lifecycle model](examples/mechanics/timer-lifecycle/model.py) are also included. These are
separate evidence categories, not additions to the 131-file count.

Historical AddBuffInner, GetStackableBuff, and SetDuration candidates remain
BLOCKED / NOT_APPLIED. A `DIRECT_CIL_FULL` label is an inherited evidence class,
not fresh authentication of an original game binary. `SOURCE_ONLY` is not a CIL
equivalence claim. See the [gameplay showcase](examples/mechanics/gameplay-showcase/README.md)
for included source and fixture qualifications.

## Selected findings

- Both attribute snapshots precede attacker replacement, which requires a
  non-null control handle and `ConcomitantPhantomRole`. Counter processing
  precedes critical modifiers; selected successful responses reach gated events
  and effects. The HP-writing consumer, server validation, and replication remain
  unknown. The [damage walkthrough](docs/DAMAGE_SNAPSHOT_FLOW.md) distinguishes
  these source observations from checks of 24 CIL call sites in ten methods,
  which do not establish complete source/CIL equivalence.
- The selected `SetBaseValue` path writes the base value before calling
  `UpdateCurrentValue`; event dispatch follows only when the current value
  changes. The actual buff authority and a dispatch-to-snapshot route are not
  established. See [BaseAttributeStorage](src/managed/cs-e3fc67839851/attributes/BaseAttributeStorage.cs).
- The selected skill gate checks cooldown before later strength, control, and
  interruption conditions. The authored model does not establish game behavior.
  See [BaseSkillComponent](src/managed/cs-e3fc67839851/skills/BaseSkillComponent.cs).
- The JavaScript event bridge calls `cpp.FEventSystem` and routes callbacks to
  global or entity-targeted listeners. Native marshalling, direct C# interop, and
  migration chronology remain unknown. See [EventCSharpBridge](src/scripts/reconstructed/event/EventCSharpBridge.js).
- The authored timer model covers six lifecycle scenarios, including a
  self-removal case that remains queued until its due-time pop. Run it from the
  repository root with:

  ```text
  python -B examples/mechanics/timer-lifecycle/model.py
  ```

  It checks selected source bindings and model scenarios; it is not a game oracle
  or runtime test.

## Research tooling

Recorded bounded use includes ILSpy-MCP (`ilspy-mcp`) for managed metadata and
CIL inspection, and the official Hex-Rays IDA MCP (`official-ida-mcp`) together
with headless idalib for selected native static analysis. These records do not
represent fresh whole-game authentication. The underlying tool-use receipts are
not bundled.

The Rust maintainer tools and their prepared offline build command are documented
in [tools/re](tools/re/README.md). Rust 1.88 or newer and a previously prepared
dependency cache are required; dependencies are not vendored. Private captures,
graphs, indexes, reference packs, and harness receipts are not bundled. Missing
input diagnostics are not successful verification. See
[reproducibility boundaries](docs/REPRODUCIBILITY.md).

**Was REA used for this project?** No.

## Evidence and limitations

Selected views, authored models, declarations, and historical fixtures are
different representations. Generated callbacks and accessors are not
automatically independent gameplay methods. Game-version continuity is unknown
where source records say so. Historical build and test results describe their
named runs, not a fresh clean-checkout result. Native semantics and TypeScript
buildability are not established; known malformed declarations remain preserved.

The [publication provenance](docs/PUBLICATION_PROVENANCE.json) records an earlier
240-path selection snapshot. It does not certify this checkout, authenticate
original game binaries, or resolve ownership or redistribution rights.

## Contributing

Start with an existing source view, its metadata, and the
[contribution rules](CONTRIBUTING.md). Report the full assembly/MVID/TypeDef/
MethodDef identity when available, distinguish observation from inference, and
state which checks you actually ran. Do not fill missing bodies with plausible
stubs, silently normalize preserved source, or turn historical candidates into
accepted source. Keep editorial successors separate from preserved records.
Do not submit game binaries, extracted assets, credentials, or private inputs.

## Acknowledgements and References

The [Mono.Cecil](https://github.com/jbevain/cecil) library was used directly for
managed assembly metadata and IL inspection.
The documentation and methods of [OpenGOAL](https://github.com/open-goal/jak-project)
and [zeldaret/oot](https://github.com/zeldaret/oot) were consulted as references.
These projects are not code dependencies, do not endorse this work, and provide
no authority for its claims or rights to third-party game-derived material.

## Copyright & Legal Notice

This is an independent, non-commercial reverse-engineering research project. No
affiliation, endorsement, or authorization from Kuro Games is claimed. Wuthering
Waves and relevant game software, trademarks, and content belong to their
respective rights holders.

The repository contains recovered or reconstructed game-derived expressions and
independently authored material. The maintainers do not claim ownership of
third-party game code. This repository grants no license to redistribute
third-party game-derived content. Public availability does not establish
open-source status or redistribution permission.

A public maintainer contact channel has not yet been configured. Specific
rights-holder concerns will be reviewed promptly once a channel is published.
This notice is not a license, waiver, or legal exemption.
