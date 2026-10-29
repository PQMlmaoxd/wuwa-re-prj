# Tests and recovered harnesses

This page describes the authored C# harnesses under `tests/managed-recovered/`
and the separate Rust tool regressions. The Event 101 reference model lives
under `examples/mechanics/event101-reference/`, not here. The C# harnesses are
not a current all-in-one test suite, and they do not build a game client or
prove runtime behavior.

Two kinds of testing live near this repository:

- Rust tool regressions under [tools/re/tests](../tools/re/tests/) exercise the
  metadata, registry, provenance, and source-closure CLIs. They are separate
  from the C# harnesses and from the 131 curated source files.
- C# harnesses under `tests/managed-recovered/` compile exact saved bodies under
  authored local hosts. They are fixed, task-scoped runs, not a reusable package
  test target.

The counts below are historical or accepted results. None was produced by a new
run for this documentation change. Read
[REPRODUCIBILITY.md](../docs/REPRODUCIBILITY.md) before treating any count as a
current suite total.

## The three baseline recovered harnesses

| Harness | Public source | Accepted result | Scope |
| --- | --- | --- | --- |
| Buff and attribute | [Editorial history](../docs/history/harnesses/buff-attribute/README.md), [HarnessTests.cs](managed-recovered/buff-attribute/HarnessTests.cs), [TestDoubles.cs](managed-recovered/buff-attribute/TestDoubles.cs) | 28 of 28 cases passed, 115 assertions, 0 synthetic models | 26 cases use recovered source with stubbed dependencies; 2 are direct recovered helper tests |
| Attribute snapshot | [Editorial history](../docs/history/harnesses/attribute-snapshot/README.md), [HarnessTests.cs](managed-recovered/attribute-snapshot/HarnessTests.cs), [TestScaffolding.cs](managed-recovered/attribute-snapshot/TestScaffolding.cs) | 8 of 8 groups passed, 351 assertions, 0 synthetic models | Three saved source bodies under authored hosts, arrays, constructors, payloads, and enum adapters |
| Damage formula | [Editorial scope](../docs/history/harnesses/damage-formula/README.md), [historical RESULTS](../docs/history/harnesses/damage-formula/RESULTS.md), [HarnessTests.cs](managed-recovered/damage-formula/HarnessTests.cs), [TestScaffolding.cs](managed-recovered/damage-formula/TestScaffolding.cs), [run_damage_formula.py](managed-recovered/damage-formula/run_damage_formula.py) | 8 groups passed, 39 assertions, 0 failures | 9 saved bodies; 7 new distinct identities for that task and 2 previously exercised |

The buff and snapshot runs together exercised 13 distinct method identities
across 36 historical cases or groups and 466 assertions. That is a historical
combined scope. It is not a current integration test, not a current full-suite
total, and not a reason to claim that the pieces form one call graph.

The damage-formula result is a later and separate accepted execution. Its result
covers `CalculateHurt` and six calculation helpers that were new for that task,
plus `GetAttrFromSnapshots` and `AttributeSnapshot.GetCurrentValue`, which had
been exercised before. The executed slice does not establish the caller,
registries, HP application, server authority, or a buff-to-damage route.

## Later separate harnesses

Two newer directories have public harness and scaffolding sources but no README.
They are separate from the three baseline harnesses.

| Harness | Public source | Private execution result | Scope |
| --- | --- | --- | --- |
| Transformed damage dispatch | [HarnessTests.cs](managed-recovered/damage-dispatch-transformed/HarnessTests.cs), [TestScaffolding.cs](managed-recovered/damage-dispatch-transformed/TestScaffolding.cs), [run_dispatch.py](managed-recovered/damage-dispatch-transformed/run_dispatch.py) | 8 groups, 50 assertions, 0 failures | Explicitly transformed recovered fixture: the `CalculateFormula` missing-key log span uses an explicit array instead of compiler inline-array helpers. The unchanged recovered body was not executed. |
| Skill interruption | [HarnessTests.cs](managed-recovered/skill-interruption/HarnessTests.cs), [TestScaffolding.cs](managed-recovered/skill-interruption/TestScaffolding.cs) | 8 groups, 127 assertions, 0 failures | Four exact `BaseSkillComponent` slices under controlled dependency doubles; not whole-class or game-runtime evidence |

The transformed dispatch run is not the unchanged recovered `CalculateFormula`,
and it does not execute `DispatchCurrentValueEventImplement`. The skill result
is not part of the buff-to-damage chain. Neither result changes a historical
source label, CIL classification, or equivalence claim.

## Authored examples

The [timer lifecycle experiment](../examples/mechanics/timer-lifecycle/model.py)
is a standalone Python model with three source bindings and six scenarios. Run
`python -B examples/mechanics/timer-lifecycle/model.py` from the repository root.
Its assumptions and conditional lifecycle result are explained in the
[damage snapshot walkthrough](../docs/DAMAGE_SNAPSHOT_FLOW.md#timer-lifecycle-experiment).
Model results are not observed game bugs or runtime measurements.

The [Event 101 reference](../examples/mechanics/event101-reference/README.md) is
an independently authored synthetic C# model. It is not recovered or
decompiled game source. It uses net8 model inputs only, with no game dependency,
package reference, network, native bridge, or filesystem access.

Its public README records ten implemented test groups and no build or execution.
A later private reference run reported 10 of 10 groups passing with exit 0. That
receipt did not instrument per-assertion counts, and no public offline build is
verified. The run is model execution only; it provides no buff, snapshot, or
damage evidence.

## Frozen READMEs and private inputs

The original prepared READMEs for buff, snapshot, and damage remain private
historical records. Their public editorial successors are linked above and name
their predecessors; they are not byte-identical originals. Later private receipts
supersede execution status only, not source labels, CIL classifications,
equivalence claims or runtime claims.

The test directories contain authored harnesses, test doubles, scaffolding and
runners. Historical documentation and the damage result have public successors
under `docs/history/harnesses/`. Seven source-derived fixture views are also
included under `qualified-historical/`; the
[fixture map](../examples/mechanics/gameplay-showcase/README.md#qualified-historical-source-map)
links them. They retain authored-host and transformation qualifications, not
accepted whole-class or game-runtime status. The private preparation roots,
generated manifests and recovered-harness project files are not included.

## Private records and reproduction

Private execution receipts, canonical inputs, pinned reference packs and task
scratch roots are not distributed. The selected fixture source views do not
replace that preparation and execution evidence. The public damage result
records that candidate's historical run; buff and snapshot results are summarized
from private receipts and cannot be independently reproduced from public files
alone.

There is no public clean-clone recipe. Each recovered C# harness consumed a
fixed task root, a prepared fixture, copied recovered bodies, and a pinned
reference pack or SDK. Event 101 has potentially self-contained net8 compile
inputs, but no clean public offline run has been verified. The damage runner is
one-shot and refuses reuse. The buff and snapshot Prepare steps refuse an
existing root, while a fresh run id can be allowed at run time. Re-executing any
recovered harness requires a new locally approved task, fresh roots, a fresh
manifest, and the private inputs.

The Rust crate declares Rust 1.88 and edition 2024 in
[Cargo.toml](../tools/re/Cargo.toml), with dependencies pinned by
[Cargo.lock](../tools/re/Cargo.lock). The dependencies are not vendored. The
historical regression record reports 215 tests: 214 active and one explicitly
ignored administrative test. That is a received count, not a run performed for
this page.

Do not run `cargo test`, a harness script, or a cleanup command merely to see
whether the tree is healthy. Those surfaces can create and delete fixtures, and
`--target-dir` does not redirect every test fixture path. Review the
[reproducibility boundaries](../docs/REPRODUCIBILITY.md) before a separately
authorized run. No build, test, compiler, runtime, or cleanup command was run
for this page.
