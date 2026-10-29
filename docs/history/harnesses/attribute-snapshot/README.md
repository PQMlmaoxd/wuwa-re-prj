# Attribute snapshot harness history

Editorial successor to historical path
`tests/managed-recovered/attribute-snapshot/README.md`; not a byte-identical
replacement. The saved predecessor SHA-256 is
`648f405b3fd3d558e1a02cc1fd6c154ca9828b4486f1905dc519505e56712410`. This page
records the historical scope and result, not a new receipt or original-binary
authentication.

## Saved-method ledger

The harness exercised three exact source slices under authored hosts. Byte ranges
are inclusive 1-based source lines; source and slice hashes identify the saved
inputs and are not current whole-file pins for this editorial page.

| # | Method | TypeDef / MethodDef | Source / lines | Whole source bytes / SHA-256 | Slice bytes / SHA-256 | Source offset | RVA / code / EH | Historical qualification |
|---:|---|---|---|---|---|---:|---|---|
| 1 | `BaseAttributeComponent.TakeSnapshot()` | `0x02002e53 / 0x060184c5` | BAS / 412-418 | 35847 / `c2b14c11808966b4ee81301b631f52f9e7c07777bdd9e72758bfc81f686fb6b9` | 268 / `6d84953e8e51125da0a08610d1d59c8219068ee653fe06f07bdbd26df531f201` | 11943 | 7113276 / 52 B / EH0 | `FULL_SELECTED_CIL` |
| 2 | `AttributeSnapshot.GetCurrentValue(EAttributeType)` | `0x02002e52 / 0x060184b2` | AS / 14-17 | 372 / `3e2cd1b7799363521318c126faaf27e4ff2e24278e72761f7243b6158b7dd5da` | 96 / `2e467b842558ac501d1d3f02ec879a969906a09fefe724fc24f991f57aed41f6` | 274 | 7111924 / 9 B / EH0 | `HISTORICAL_SOURCE_CONTEXT_METADATA_BOUND` (R38 161-167; raw method row 221) |
| 3 | `Calculation.GetAttrFromSnapshots(SnapshotPayload, EAttrOwnerType, EAttributeType)` | `0x02002eb2 / 0x06018865` | CALC / 20-28 | 24688 / `daa9fff61d83566475f474ab84e2feea1b1b7de987a08a678cc0c87a32ea6e34` | 360 / `28fb4ef1a2962b49c17b987a90b3990d62cbda76729772c53ad80faabbaf5abf` | 739 | 7210036 / 56 B / EH0 | `FULL_CIL; SOURCE_FIDELITY_PASS` (R38 116, 124-140; raw method row 642; map row 289) |

The source keys BAS, AS, and CALC identify `BaseAttributeComponent`,
`AttributeSnapshot`, and `Calculation`. The declaring type for
`GetAttrFromSnapshots` is `Calculation`, not `BaseDamageComponent`.

## Authored host assumptions

- The host used separate 143-element base and current arrays, a minimal
  `SnapshotPayload` adapter, and partial host declarations. These do not
  establish whole-class or build compatibility.
- The authored `EAttributeType` adapter used `Lv=1`, `CritDamage=9`, `Def=10`,
  `DamageChange=15`, and `Max=143`. These selected constants are not a complete
  enum or game configuration. `Max=143` is an adapter boundary.
- `EAttrOwnerType` used saved names and values `Attacker=0`, `Target=1`; the
  selected method's behavior is zero selects attacker and nonzero selects target.
- `SnapshotPayload` exposed only the two snapshot properties consumed by the
  selected method. No `GetBaseValue` method was authored or used.

Inputs were synthetic, not game configuration or damage output. No whole-enum,
whole-class, or source/CIL equivalence claim follows from the host.

## Eight-group oracle

| Group | Exact methods exercised | Synthetic input and expected result |
|---|---|---|
| S01 | `TakeSnapshot`, `GetCurrentValue` | base[9]=100, current[9]=7; getter=7, copied base=100, and no component/snapshot array alias |
| S02 | `TakeSnapshot` | base[i]=1000+i, current[i]=1+0.25i; boundary values base[0]=1000, base[142]=1142, current[0]=1, current[142]=36.5; both arrays copy all 143 entries; current[9/10/15]=3.25/3.5/4.75 and differ from base |
| S03 | `TakeSnapshot`, `GetCurrentValue` | capture base[9]=10/current[9]=20, then component current[9]=99; old snapshot getter=20 and arrays remain distinct |
| S04 | `TakeSnapshot` twice, `GetCurrentValue` | capture current[9]=2, mutate to 3, capture again; second=3, first=2, snapshots and arrays independent |
| S05 | `TakeSnapshot`, `GetCurrentValue` | base[9]=111/current[9]=222, then mutate snapshot base to 333 and component base to 444; getter/current remain 222 and bases remain separate |
| S06 | `GetAttrFromSnapshots` via authored wrapper | attacker[9/10/15]=11/12/13, target[9/10/15]=21/22/23; owner 0 selects attacker; `Target` (1), 2, -1, `int.MinValue`, and 999 select target |
| S07 | `GetCurrentValue`, `GetAttrFromSnapshots` | raw getter -1/143 throws `IndexOutOfRangeException`; raw 0/142 returns 100/1420; helper -1/0/143/144 returns 0.0 before dereference even with null payload; helper valid 1 attacker returns 101 and valid 142 target returns 1420; selected null attacker/target throws `NullReferenceException`; an unused null snapshot succeeds |
| S08 | `TakeSnapshot` twice, `GetCurrentValue`, `GetAttrFromSnapshots` | synthetic float values 9.5/10.5/15.5 and 19.5/20.5/25.5; helper double equals the float getter promoted to double for both owners |

The recorded runner result was **8 of 8 groups passed, 351 assertions, and 0
synthetic-model tests**. The earlier preparation checkpoint said
`NOT COMPILED, NOT EXECUTED`; the later receipt supersedes that execution status
only. The receipt and complete private run layout are not distributed; a selected
fixture-source view is linked below. The result
was not independently reproduced from this public repository.

Compilation or execution does not upgrade the three source qualifications.
`AttributeSnapshot.GetCurrentValue` remains historical source context with
metadata binding, not a newly selected CIL body. No game-runtime,
behavioral-equivalence, or clean-clone claim follows.

## Included navigation

- [Authored harness tests](../../../../tests/managed-recovered/attribute-snapshot/HarnessTests.cs)
- [Authored test scaffolding](../../../../tests/managed-recovered/attribute-snapshot/TestScaffolding.cs)
- [Qualified snapshot fixture](../../../../qualified-historical/tools/re/local-recovery/snapshot-direct-test01/scratch/project/RecoveredFixture.cs)
- [Current source and evidence index](../../../SOURCE_INDEX.md)
- [Reproduction boundaries](../../../REPRODUCIBILITY.md)
