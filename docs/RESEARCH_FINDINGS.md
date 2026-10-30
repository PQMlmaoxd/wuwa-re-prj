# Research findings

This register separates source-supported observations from authored model
results. It is not original developer source, complete method recovery, runtime
evidence, server approval, or a rights determination. The game version for each
finding is `UNKNOWN`.

## Authority and version scope

| Authority | Recorded identity | Interpretation |
| --- | --- | --- |
| `MANAGED-CS-E3FC67839851` | Managed assembly SHA-256 `e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`; MVID `2d6d37cb-76e5-4075-8220-3eeca41471e3`; game version `UNKNOWN` | The `CSharpScript, Version=1.0.0.0` text is an assembly version label, not a game version. |
| `BLD-WUWA-0001` | Historical client build reference `3.6.0-50004`; client SHA-256 `a6cdb83f6f2909b5a13ca19b1d5ff4f0455f743e8fabf40bb07546500968cdfa` | `managed_build_claim=false`; association is `BUILD_METADATA_ONLY_NOT_DIRECT_BINARY_AUTHORITY`. Do not assign this build to the managed assembly. |
| Non-managed source views | Per-file source identity; no direct binary authority is established here | The historical client build reference does not provide a JavaScript version assignment or automatic cross-build join. |

These authority fields are recorded in [metadata/migration.json](../metadata/migration.json).
All six findings below remain game-version `UNKNOWN`; model inputs and similar
names are not version evidence.

## DAMAGE-SNAPSHOT-BEFORE-REPLACEMENT

**Game version:** `UNKNOWN`.

**Finding:** In the included `AllocateSnapshot` source span, attacker and target
attribute snapshots are captured before the payload initializer calls
`ReplaceAttacker(attacker)`. Replacement is conditional on a non-null
`VisionControlHandle` and `ConcomitantPhantomRole`; the helper may then return a
null summoner damage component.

**Evidence and limits:** `qualified-research/damage-snapshots/BaseDamageComponent.SourceOnly.cs`
has SHA-256 `12e76a78de325fa5b072ee4ac7f1b3dc779f4f95acc99d7a81fe77e50aa46aa3`;
assembly authority `MANAGED-CS-E3FC67839851`, MVID
`2d6d37cb-76e5-4075-8220-3eeca41471e3`, TypeDef `0x02002e65`, and MethodDefs
`0x0601864f` / `0x06018650`. The exact spans are `SOURCE_ONLY`. The
[damage walkthrough](DAMAGE_SNAPSHOT_FLOW.md) and its
[evidence record](../qualified-research/damage-snapshots/evidence.json) document
ten methods and 24 selected original-CIL call sites, not complete body
equivalence. The HP-writing consumer, server validation, and replication remain
unknown.

## DAMAGE-COUNTER-BEFORE-CRITICAL

**Game version:** `UNKNOWN`.

**Finding:** The `ProcessDamageOptimize` source span calls
`ProcessWhenCounter`, `PreCriticalModify`, and `PostCriticalModify` in that
order. `ProcessWhenCounter` triggers the attacker-side `ForWhenCounter` and
target-side `ForWhenBeCountered` events only when `IsCounterAttack` is true.

**Evidence and limits:** This is in the same `SOURCE_ONLY` excerpt and managed
identity as the preceding finding, at TypeDef `0x02002e65` and MethodDefs
`0x06018662` / `0x06018651`. The [damage walkthrough](DAMAGE_SNAPSHOT_FLOW.md)
records selected source and CIL call-order checks; it does not establish runtime
execution, performance, or complete critical-damage semantics.

## ATTRIBUTE-WRITE-DISPATCH-ORDER

**Game version:** `UNKNOWN`.

**Finding:** In `BaseAttributeStorage.SetBaseValue`, the accepted base value is
written to `BaseValues` before the old current value is read. The code then
calls `UpdateCurrentValue`, reads the new current value, and calls
`DispatchCurrentValueEvent`. `BaseAttributeComponent.DispatchCurrentValueEvent`
calls its implementation only when `oldValue != newValue`.

**Evidence and limits:** The included source files are
`src/managed/cs-e3fc67839851/attributes/BaseAttributeStorage.cs` (SHA-256
`20fad70c25a20cca2a1107cab1afef9991b3a9e963c5a39c454ec9acc054e6a9`, TypeDef
`0x02002e53`, MethodDef `0x060184bc`, lines 120-143) and
`src/managed/cs-e3fc67839851/attributes/BaseAttributeComponent.cs` (SHA-256
`ebee23e1fb7a8d1ea6f048fb53b19a669f228f92c001db621b004a8f2ea70b86`, MethodDef
`0x060184e1`, lines 310-315). This is source-supported ordering, not a new CIL
claim. Writable policy, transitive overrides, and live HP effects are unknown.

## SKILL-BEGIN-GATE-ORDER

**Game version:** `UNKNOWN`.

**Finding:** `CheckSkillCanBegin` has earlier authority, tag, frozen-state,
genre, and passive-ability gates before its cooldown check. Later gates include
strength, multi-skill eligibility, a monster control-switch condition, and the
interrupt check. The cooldown check is not the first gate overall.
`CheckSkillCanInterrupt` delegates to `DoCheckInterrupt`.

**Evidence and limits:**
`src/managed/cs-e3fc67839851/skills/BaseSkillComponent.cs` has SHA-256
`e858e22119394e491d0d368ffb10af02b565e334d496adc8d50ce7c53fddbe40`, TypeDef
`0x02003118`, and MethodDefs `0x0601a016`, `0x0601a017`, and `0x0601a018`.
The relevant source spans are lines 91-147 and 151-155. This records the
included source-view ordering only; it is not model parity, server approval, or
a new CIL-body claim.

## TIMER-TRASH-REQUEUE-MODEL

**Game version:** `UNKNOWN`; the model is not version evidence.

**Finding:** Under the authored model assumptions, a repeating timer whose
callback reaches `DoRemove` on that same timer can still be requeued when `Do`
returns true, because no post-callback state guard is modeled. A later due-time
pop skips the timer in `Trash` state. This is a conditional model result, not an
observed game bug, leak, or performance issue.

**Evidence and limits:** The authored
[`model.py`](../examples/mechanics/timer-lifecycle/model.py) checks source
bindings for `Timer.cs` (SHA-256
`c5d437c8f4c80de3a163fbb80fc6c4968b3467cb280d8bf4c1f9d310e09ecbfa`),
`TimerSystemInstance.cs` (SHA-256
`2607a87cd72841399c0f6159ea08a9bd2587317c57e7b92327dee1f819b12201`), and
`PriorityQueue.cs` (SHA-256
`48d2102543b23a10077b86b6621b0f805f60c1dc8b3773b85a0baf2b46b1bb4d`). Its six
scenarios are `one-shot`, `repeat-catch-up`,
`self-remove-trash-until-due`, `counterfactual-post-callback-guard`,
`pending-remove-after-due-loop`, and `pre-removed-skip`. Assumptions include one
owner/thread/timer, an initially `Normal` timer, a positive unit interval and
dilation, and nonthrowing callbacks and registry/logging stand-ins. The model
assumes the callback reaches `DoRemove` for the same timer; the omitted lookup
behavior is not implemented. The six authored checks are `MODEL_TESTED`; the
conditional requeue scenario is `INFERRED`.
See the [timer lifecycle notes](DAMAGE_SNAPSHOT_FLOW.md#timer-lifecycle-experiment).

## SCRIPT-EVENT-BRIDGE-DISPATCH

**Game version:** `UNKNOWN`.

**Finding:** The included reconstructed JavaScript projection calls
`cpp.FEventSystem` for initialization and emission. Its callback routes target
entity ID `0` to global listeners and nonzero target IDs through an entity lookup before
entity-targeted dispatch. The visible emit wrappers forward zero through five
payload arguments.

**Evidence and limits:**
`src/scripts/reconstructed/event/EventCSharpBridge.js` has included projection
SHA-256 `7e746a952bf008f9455f48f26efbdb3b54ae426578f089f0f82cd3888b9a61a4` and
records the logical shipped module path
`JavaScript/Game/Common/Event/EventCSharpBridge.js` with shipped SHA-256
`6c91779f6f0f5c25c72ecfaa61aac679a809100e78aecfc5dd4bc61ef325d9ab`. This is a
`RECONSTRUCTED_JS` projection with source-supported call sites. The relevant lines are 23, 30-44,
47-63, and 66-72. It is not native ABI, direct C# interoperation, Puerts
ownership, or migration evidence. The historical client build reference does
not establish a direct JavaScript version assignment.
