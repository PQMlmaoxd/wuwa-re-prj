# Wuthering Waves Gameplay RE

Research into selected gameplay implementations in **Wuthering Waves**, with reconstructed source views, method-level evidence, and small experiments for examining execution behavior.

The repository focuses on skills, attributes, damage, timers, events, and boundaries between managed code, scripts, and native systems. It is a source-oriented reference for further investigation, not a game client or an attempt to reconstruct the entire game.

> [!NOTE]
> Recovered expressions and authored models are different kinds of evidence. The material here is not original developer source, a complete decompilation, or proof of live-game behavior.

**Start here:** [Research highlights](#research-highlights) | [Finding register](docs/RESEARCH_FINDINGS.md) | [Source index](docs/SOURCE_INDEX.md) | [Quick start](#quick-start) | [Evidence limits](#evidence-and-verification)

## Research highlights

**Damage snapshots.** In the included `BaseDamageComponent` views, attacker and target attribute snapshots are captured before conditional attacker replacement ([finding](docs/RESEARCH_FINDINGS.md#damage-snapshot-before-replacement)). Counter processing occurs before critical modifiers ([finding](docs/RESEARCH_FINDINGS.md#damage-counter-before-critical)). The examined response path also shows gated event and effect callbacks. The actual HP-writing consumer, server validation, and replication remain unknown. Read the [damage flow investigation](docs/DAMAGE_SNAPSHOT_FLOW.md) alongside its [source excerpts](qualified-research/damage-snapshots/BaseDamageComponent.SourceOnly.cs).

**Skills and attributes.** The [skill component](src/managed/cs-e3fc67839851/skills/BaseSkillComponent.cs) exposes the order of cooldown, strength, control, and interruption checks ([finding](docs/RESEARCH_FINDINGS.md#skill-begin-gate-order)). The [attribute storage view](src/managed/cs-e3fc67839851/attributes/BaseAttributeStorage.cs) shows how a base-value write leads to current-value updates and conditional event dispatch ([finding](docs/RESEARCH_FINDINGS.md#attribute-write-dispatch-order)). These are separate observations, not a reconstructed end-to-end combat pipeline.

**Timers.** A [Python lifecycle model](examples/mechanics/timer-lifecycle/model.py) exercises six scenarios derived from timer source operations ([finding](docs/RESEARCH_FINDINGS.md#timer-trash-requeue-model)). One scenario explores callback-driven self-removal of a repeating timer. This is a source-bound experiment, not a reproduced game bug.

**Script interfaces.** [EventCSharpBridge.js](src/scripts/reconstructed/event/EventCSharpBridge.js) calls `cpp.FEventSystem` and routes callbacks to global or entity-targeted listeners ([finding](docs/RESEARCH_FINDINGS.md#script-event-bridge-dispatch)). The native implementation, direct C# interoperation, and migration history have not been recovered.

**Authority scope.** All six findings have game version `UNKNOWN`; the managed assembly and historical client-build reference are distinct authorities. The [finding register](docs/RESEARCH_FINDINGS.md) records their identities and limits.

## Repository guide

| Location | What you will find |
| --- | --- |
| [`src/`](src/) | 131 curated C#, TypeScript, JavaScript, C++, and header files, organized by gameplay area. |
| [`docs/SOURCE_INDEX.md`](docs/SOURCE_INDEX.md) and [`metadata/`](metadata/) | Method identities, source qualifications, and evidence navigation. |
| [`qualified-historical/`](qualified-historical/) | 18 historically qualified source and fixture representations. |
| [`qualified-research/damage-snapshots/`](qualified-research/damage-snapshots/) | Five additional `SOURCE_ONLY` excerpts and their evidence record. |
| [`examples/mechanics/`](examples/mechanics/) | Authored damage, skill, event, and timer reference models. |
| [`tools/re/`](tools/re/) | Rust utilities for metadata inspection, source lookup, and preservation checks. |
| [`docs/`](docs/README.md) and [`tests/`](tests/README.md) | Research notes, worked examples, historical results, and test boundaries. |

The historical `AddBuffInner`, `GetStackableBuff`, and `SetDuration` candidates are still **BLOCKED / NOT_APPLIED**. They are not part of the accepted reconstructed implementations.

## Quick start

The timer model can be run with Python from the repository root:

```sh
python -B examples/mechanics/timer-lifecycle/model.py
```

It checks six authored scenarios and their source bindings. No game installation or private capture is needed.

For a structural view of the curated metadata, fetch the locked crates once while online:

```sh
cargo fetch --locked --manifest-path tools/re/Cargo.toml
```

Then, from the checkout root, run the read-only status command offline:

```sh
cargo run --locked --offline --manifest-path tools/re/Cargo.toml --bin re -- status --curated-root . --json
```

This requires **Rust 1.88+** with edition 2024 support; the public workflow pins 1.94.1. Crates are not vendored, so the first fetch needs network access. A successful status result has `ok=true`, `command=status`, `exit=0`, and `findings=[]`. It checks metadata, identities, and path presence. The legacy output label `verify_scope=BYTE_INTEGRITY_ONLY` does not mean that `status` rehashes every source file or verifies CIL bodies, semantics, or the original binary.

See the [tooling guide](tools/re/README.md) for other commands and the [reproducibility matrix](docs/REPRODUCIBILITY.md) for private-input requirements.

## Evidence and verification

A reconstructed method is identified by its available assembly identity, MVID, TypeDef, and MethodDef, rather than by class name alone. The [source index](docs/SOURCE_INDEX.md) provides the relevant records and qualification labels.

| Material | Supported reading |
| --- | --- |
| Reconstructed source views | Expressions and ordering visible in the included representations, not original developer-source authentication. |
| Damage CIL investigation | 24 inspected call sites in ten original methods; no complete source-to-CIL equivalence claim. |
| Rust structural tooling | Metadata and preservation checks within their documented input and validation scope. |
| Mechanics models | Reproducible behavior under stated assumptions, not game-runtime or server equivalence. |

Original game binaries, the full private research corpus, derived indexes, and several harness receipts are not distributed. A number of native interfaces and runtime relationships remain open questions; the [documentation guide](docs/README.md) and [gameplay showcase](examples/mechanics/gameplay-showcase/README.md) explain the boundaries. There is no measured performance improvement claim.

## Research tools

[Mono.Cecil](https://github.com/jbevain/cecil) was used to inspect managed metadata and CIL. Recorded MCP usage includes **ILSpy-MCP** (`ilspy-mcp`) for bounded managed inspection and the official Hex-Rays IDA MCP integration (`official-ida-mcp`), used alongside headless idalib for selected native surfaces. The underlying tool-use receipts are not included here.

**Was REA used for this project?** No.

## Contributing

Changes should be traceable to an identified source or evidence record, with observed behavior separated from interpretation. Read [CONTRIBUTING.md](CONTRIBUTING.md) for source identity, preservation, and review requirements. Do not submit game binaries, extracted assets, credentials, or private research inputs.

## Acknowledgements and references

**Tools and methodology.** [Mono.Cecil](https://github.com/jbevain/cecil) supports the project's managed inspection workflow. [OpenGOAL](https://github.com/open-goal/jak-project) and [zeldaret/oot](https://github.com/zeldaret/oot) were consulted for source-reconstruction documentation and the separation of source, tooling, and verification.

**Wuthering Waves research.** [alt3ri/WuWa_SDK](https://github.com/alt3ri/WuWa_SDK) and [chadlrnsn/wuwa-sdk](https://github.com/chadlrnsn/wuwa-sdk) publish Unreal SDK structures, object dumps, and IDA mappings useful for understanding native-side interfaces. [Arikatsu/WutheringWaves_Data](https://github.com/Arikatsu/WutheringWaves_Data) and [Dimbreath/WutheringData](https://github.com/Dimbreath/WutheringData) provide extracted configuration and text data.

**Related community work.** [Hiro420/WuwaTools](https://github.com/Hiro420/WuwaTools) contains protocol and configuration inspection utilities; [yuhkix/wuwa-ids](https://github.com/yuhkix/wuwa-ids) collects gameplay identifiers; and [Wuthering Waves Mod Starter](https://github.com/aarlin/wuthering-waves-mod-starter) documents JavaScript layouts and modding workflows.

These references acknowledge relevant tools and public research. They do not imply direct code reuse, collaboration, endorsement, or permission to redistribute third-party content.

## Copyright & legal notice

This is an independent, non-commercial reverse-engineering research project. It is not affiliated with, endorsed by, or authorized by Kuro Games. Wuthering Waves, its trademarks, and its original game software remain the property of their respective rights holders.

The repository includes game-derived recovered or reconstructed expressions as well as independently authored research material. The maintainers do not claim ownership of third-party game code or grant any license to redistribute it. Public availability does not establish open-source status or redistribution permission. This notice is not a license, waiver, or legal exemption.

A public maintainer contact channel has not yet been configured. Specific rights-holder concerns will be reviewed promptly once a channel is published.
