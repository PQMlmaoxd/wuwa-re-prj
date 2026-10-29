# Gameplay showcase: buff attributes to damage snapshots

This showcase follows three separate pieces: synthetic buff and attribute
mutation, separate snapshot capture, and formula-side attribute access through
`AttributeSnapshot`. They share state shapes and identities, but they are not a
tested end-to-end route. Game version is `UNKNOWN`; class views are partial, not
original developer source.

## What the pieces show

- `TakeSnapshot` copies base and current into separate `float[143]` arrays. A
  snapshot is a copied pair, not a live view; tests do not prove immutability of
  the public arrays.
- `SetBaseValue` checks `IsWritableAttribute` before it writes. Actual
  writability of damage-stat targets remains `UNKNOWN`. Bounds normalization can
  clamp to zero and can skip a write when the normalized base equals the old one.
- `AddBaseValue` reads the old base before `SetBaseValue`, so an invalid index
  throws before permission is reached. A normal zero add can normalize a negative
  old base to zero.
- Base is written before virtual `UpdateCurrentValue`, and dispatch runs only
  when old current differs from new. B08 and B09 show partial state after
  exceptions; runtime-bound virtual behavior remains `UNKNOWN`.
- Modifier policy 1 multiplies base by `1 + V1 * 0.0001 * stack`; policy 6 uses a
  time-scaled base; policies 7 and 8 are no-ops. Policies 2, 4, and 9 share
  derived reads. The real call site runs under `HasBuffAuthority`, which the
  synthetic config does not establish.
- `CalculateHurt` reads target `Def`, attacker `DamageChange`, and `CritDamage`
  unconditionally, then uses `CritDamage` as a multiplier only when the hit is
  critical. `GetAttrFromSnapshots` is declared by `Calculation`.

The separate [damage snapshot walkthrough](../../../docs/DAMAGE_SNAPSHOT_FLOW.md)
covers the included snapshot-allocation caller: both snapshots precede
conditional attacker replacement, counter processing precedes critical modifiers,
and selected successful responses reach gated events and effects. Its selected
CIL call-site checks are not complete source/CIL equivalence or an exercised
buff-to-damage route.

## Source in this repository

The first links below are curated, partial overlays. Additional bodies are
present in the qualified historical views mapped next. Neither representation
establishes original developer source, current acceptance or runtime equivalence.

- Buff and attribute storage:
  [BaseAttributeStorage.cs](../../../src/managed/cs-e3fc67839851/attributes/BaseAttributeStorage.cs)
  contains `IsWritableAttribute`, `SetBaseValue`, `AddBaseValue`,
  `GetCurrentValue`, and `DispatchCurrentValueEventImplement`.
  [BaseAttributeComponent.cs](../../../src/managed/cs-e3fc67839851/attributes/BaseAttributeComponent.cs)
  contains `UpdateCurrentValue` and the `DispatchCurrentValueEvent` wrapper. The
  two files do not contain `GetBaseValue`, `GetMaxValue`, `RefreshBounds`, or
  `TakeSnapshot` bodies.
- [ActiveBuffInternal.cs](../../../src/managed/cs-e3fc67839851/attributes/ActiveBuffInternal.cs)
  contains selected buff-lifecycle methods but not `ModifyStateAttribute`. A
  field named `ModifyStateAttributeStat` is not the method body.
- [Calculation.DamageFormula.cs](../../../src/managed/cs-e3fc67839851/gameplay/Calculation.DamageFormula.cs)
  contains `GetAttrFromSnapshots`, `CalculateHurt`, five element/attack helpers, and
  `CalculateFormula`. [BaseBuffComponent.cs](../../../src/managed/cs-e3fc67839851/buffs/BaseBuffComponent.cs)
  contains a call to `ApplyPeriodExecution`, not that method body.
- Selected `AttributeSnapshot` and `AbilityUtils` bodies are included in the
  historical fixtures below. The private snapshot CIL capture and complete
  canonical research corpus are not distributed. Method identities and historical
  execution labels remain evidence qualifications, not fresh authentication.
- Harnesses:
  [buff and attribute history](../../../docs/history/harnesses/buff-attribute/README.md),
  [attribute snapshot history](../../../docs/history/harnesses/attribute-snapshot/README.md),
  and [damage formula history](../../../docs/history/harnesses/damage-formula/README.md).
  The Event 101 model is separate under `examples/mechanics/event101-reference/`,
  not part of this chain.

## Qualified historical source map

These seven fixtures preserve recovered slices alongside authored hosts or
explicit transformations. They are source for inspection, not a ready-to-run
public harness or a promotion of historical evidence labels.

| Included view | Relevant content |
| --- | --- |
| [Buff fixture](../../../qualified-historical/tools/re/local-recovery/buff-direct-test01/scratch/project/RecoveredFixture.cs) | `GetBaseValue`, `GetMaxValue`, `RefreshBounds`, `ModifyStateAttribute` and both `AbilityUtils` helpers, alongside already curated setters and getters. |
| [Snapshot fixture](../../../qualified-historical/tools/re/local-recovery/snapshot-direct-test01/scratch/project/RecoveredFixture.cs) | `TakeSnapshot`, the snapshot getter and `GetAttrFromSnapshots`. |
| [Snapshot getter](../../../qualified-historical/tools/re/local-recovery/damage-formula01/scratch/project/src/AttributeSnapshot.RecoveredGetter.cs) | `AttributeSnapshot.GetCurrentValue`, distinct from the BaseAttributeComponent getter. |
| [Damage slices](../../../qualified-historical/tools/re/local-recovery/damage-formula01/scratch/project/src/Calculation.RecoveredBodies.cs) | Saved calculation bodies including `CalculateElementMatchUpRate`; no new identity authentication is implied. |
| [Level helper](../../../qualified-historical/tools/re/local-recovery/damage-dispatch02/fixtures/project/src/AbilityUtils.LevelValue.cs) | The repeated `GetLevelValue<T>` slice, not another new method. |
| [Transformed dispatch](../../../qualified-historical/tools/re/local-recovery/damage-dispatch02/fixtures/project/src/Calculation.RecoveredBodies.cs) | `CalculateHeal` and an explicitly changed `CalculateFormula` log span; not an unchanged recovered dispatcher. |
| [Skill fixture](../../../qualified-historical/tools/re/local-recovery/skill-interruption01/RecoveredFixture.cs) | Four selected skill slices with authored hosts and test accessors. |

Two fuller class captures are also retained:
[BaseBuffComponent](../../../qualified-historical/tools/re/local-recovery/curation-v2-candidate01/candidate/research/src/reconstructed/managed/full/CSharpScript/shards/04bd072237df8b3210063a105cf3cb35b3e6b67cba7654466e7c34414c6458a7/source/_global/BaseBuffComponent.cs),
including `ApplyPeriodExecution`, and
[ActiveBuffInternal](../../../qualified-historical/tools/re/local-recovery/curation-v2-candidate01/candidate/research/src/reconstructed/managed/full/CSharpScript/shards/acb55285e74c71a95fb06fb83c0fcc2bde6108a38a8d8e74f23c6f2a46112d0d/source/_global/ActiveBuffInternal.cs),
including `ModifyStateAttribute`. They remain noncanonical historical captures.
The separate AddBuffInner, GetStackableBuff and SetDuration candidate views remain
BLOCKED / NOT_APPLIED. Included source is not new execution or CIL-equivalence
evidence; private preparation manifests and execution receipts remain unavailable.

## Execution scope

The two historical harnesses exercised 13 method identities across 36 cases or
groups and 466 assertions: 28 cases and 115 assertions for buff, 8 groups and 351
assertions for snapshot. That is a historical combined scope, not a current suite
total. A later, separate damage-formula run executed nine saved bodies in eight
groups with 39 assertions and 0 failures;
[historical execution result](../../../docs/history/harnesses/damage-formula/RESULTS.md)
records it.
It is not a full formula or integration result. The damage-formula checkpoint's
executed Hurt slice has no rounding or ceiling step and no seed input. The
original unchanged `CalculateFormula`, seed, and round policy remain static-only
in that checkpoint; the later explicitly transformed fixture is separate and
does not establish them.

A further separate fixture exercised an explicitly transformed `CalculateFormula`
(the missing-key log span changed) with 8 groups and 50 assertions. It is not the
unchanged recovered body. It does not execute `DispatchCurrentValueEventImplement`
and does not make the buff route integrated.

## Boundaries

The permission predicate, virtual update and dispatch implementation are doubled;
the recovered dispatch wrapper is executed. The distributed
`UpdateCurrentValue` and `DispatchCurrentValueEventImplement` bodies were not the
ones run in the buff harness. There is no cited dispatch to `TakeSnapshot` edge,
no tested buff-to-damage join, and no public run recipe. Method execution under
authored hosts is not game integration or runtime behavior. The route is
conditional, not full buff damage. Response-field copies and gated callbacks are
source-supported, but the HP-writing consumer, server validation and replication
remain unknown. See
[reproducibility boundaries](../../../docs/REPRODUCIBILITY.md) for run scope and
[EVIDENCE.md](EVIDENCE.md) for method identities and limits.
