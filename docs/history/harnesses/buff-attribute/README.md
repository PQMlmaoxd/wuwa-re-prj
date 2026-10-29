# Buff and attribute harness history

Editorial successor to historical path
`tests/managed-recovered/buff-attribute/README.md`; not a byte-identical
replacement. The saved predecessor SHA-256 is
`803d8b2c2fcd9730e4ef5f325c40fc0cc6405c241b3e28a8324485be15092c68`. This page
preserves the saved-slice ledger, authored-host assumptions, case oracles, and
historical result. It is not a fresh run or a replacement for saved source.

## Authority and slice ledger

The managed authority is `CSharpScript`, assembly SHA-256
`e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, MVID
`2d6d37cb-76e5-4075-8220-3eeca41471e3`; game version is `UNKNOWN`. Source ranges
are inclusive 1-based lines. Slice bytes are exact saved substrings, including
the line feed after each closing brace. X, A, and U identify the
`BaseAttributeComponent`, `ActiveBuffInternal`, and `AbilityUtils` source files;
the complete canonical input set and private preparation manifest are not
included. Selected captures and fixture-source slices are available through the
current source map linked below.

| # | Method | TypeDef / MethodDef | Source / lines | Full source bytes / SHA-256 | Slice bytes / SHA-256 | Fixture offset / length | RVA / code bytes | Historical status |
|---:|---|---|---|---|---|---|---|---|
| 1 | `SetBaseValue` | `0x02002e53 / 0x060184bc` | X / 299-323 | 35847 / `c2b14c11808966b4ee81301b631f52f9e7c07777bdd9e72758bfc81f686fb6b9` | 678 / `59ee5dc0102152176ffea92c874ce8dcd93fb65e0410261e4e2aee4ffc4e3076` | 422 / 678 | 7112520 / 139 B | `ILSPY_FULL_CIL` (map 3831) |
| 2 | `AddBaseValue` | `0x02002e53 / 0x060184bd` | X / 325-328 | same X | 125 / `4d27cfb2ebf8412db1236961df007c792b99af15947c7ce36a4cfc527f82371a` | 1100 / 125 | 7112671 / 18 B | `ILSPY_FULL_CIL` (map 3832) |
| 3 | `GetBaseValue` | `0x02002e53 / 0x060184be` | X / 330-333 | same X | 90 / `1a96350e7e86891bbe509879bec7bcce1d7d5c94312b38c5fa2f54bc81130d03` | 1225 / 90 | 7112690 / 9 B | `UNSELECTED_SOURCE_CONTEXT_ONLY` |
| 4 | `GetCurrentValue` | `0x02002e53 / 0x060184bf` | X / 335-338 | same X | 96 / `2e467b842558ac501d1d3f02ec879a969906a09fefe724fc24f991f57aed41f6` | 1315 / 96 | 7112700 / 9 B | `SOURCE_RAW_ONLY` (map 3833) |
| 5 | `GetMaxValue` | `0x02002e53 / 0x060184c2` | X / 348-355 | same X | 200 / `5d307fb86bf8c0da3e3b9ec03b459d12c7cfe9024e293cc5e44c27aa32f4c410` | 1411 / 200 | 7112716 / 38 B | `UNSELECTED_SOURCE_CONTEXT_ONLY` (map 3987) |
| 6 | `RefreshBounds` | `0x02002e53 / 0x060184d0` | X / 673-699 | same X | 712 / `fb4b2b93a7585df4b9c6ee6acc626b404f2d8659b1b4dc6bdec34cf8e9b3c51a` | 1611 / 712 | 7115160 / 204 B | `UNSELECTED_SOURCE_CONTEXT_ONLY` (map 3987) |
| 7 | `DispatchCurrentValueEvent` | `0x02002e53 / 0x060184e1` | X / 926-932 | same X | 203 / `cffd0f4d5f2353e14b2c2e33786b519ed91c2dbc6a4fc8fd6216477fd956a230` | 2323 / 203 | 7116736 / 15 B | `FULL_SELECTED_BODY` / `SOURCE_ONLY_PRIMARY_ILSPY` (map 513) |
| 8 | `ModifyStateAttribute` | `0x02002e71 / 0x06018716` | A / 933-1040 | 36427 / `38bf6220daa1cef5f4a71e7e39c6a3376eddfb06f133c140bd72dc509667b74d` | 3663 / `412828bef096ae8288e712e553623be340e23045d2cd60b0b26576d97d2ec52c` | 2571 / 3663 | 7181812 / 735 B | `UNSELECTED_SAVED_SOURCE` (scope directive) |
| 9 | `GetLevelValue<T>` | `0x02002e4e / 0x0601848d` | U / 5-20 | 2442 / `3df28587ba45fbf96167ed2f4a941e8be81c17dfb443f8a13207c39bfff6ac0b` | 321 / `3b3c49092b7d0934aae98ebf6bb50f1d6f740e9d3b9c8b96021b6a5cc4cada51` | 6273 / 321 | 7109925 / 54 B | `UNSELECTED_SAVED_SOURCE` (scope directive) |
| 10 | `GetAttrValue` | `0x02002e4e / 0x0601848f` | U / 35-47 | same U | 470 / `2e0078033de6bc04c116318aa912afc28b834ac3c4798830bce2a00c9436843b` | 6594 / 470 | 7110012 / 61 B | `UNSELECTED_SAVED_SOURCE` (scope directive) |

The ten-slice fixture identity was 7066 bytes / SHA-256
`cc42c18839408eecb2e7c818b28513ce3150a4f35f27470ad07aae10466201dc`.
`GetCurrentValue` remains `SOURCE_RAW_ONLY`; compilation and execution do not
change any status above.

## Authored hosts and assumptions

- `IAttributeSet` supplied `GetBaseValue` and `GetCurrentValue`.
- `IBuffModifierData` supplied `AttributeId`, nullable `Value1`/`Value2`, and a
  non-null `CalculationPolicy` contract; one test passes a malformed null.
- `IBoundsLocker` supplied upper/lower lock flags, percentages, and offsets.
- The synthetic `EAttributeType` adapter used `TEST_DOUBLE_*` IDs 101-142 and
  invalid 200. These are not game enum values. The saved enum names and values
  for calculation policy and attribute-based float calculation are separate
  from those synthetic IDs.
- `CharacterAttributeTypes` exposed writable `attrsNotClampZero`,
  `stateAttributeIds`, and `attributeIdsWithMax` collections over 143-entry
  base/current arrays. `CombatLog`, `Singleton`, and `Entity` supplied a minimal
  authored log-sink host.
- Authored host state included writable predicates, update and dispatch
  observers, supplied lockers, trace counters, and 143-entry base/current arrays.
  `UpdateCurrentValue`, permission checks, bound collection, listener dispatch,
  `IsWritableAttribute`, `CollectBoundsLockers`,
  `DispatchCurrentValueEventImplement`, and logging effects were test doubles or
  unknown runtime overrides, not recovered implementations.

The exact saved slices ran under these authored hosts and dependency doubles.
Exact float comparisons were intentional oracles, not a reason to weaken a
failing assertion.

## Twenty-eight case oracles

`REC` means recovered source with stubbed dependencies; `DIR` means a direct
recovered-source helper test. Each row uses exact recovered source bytes.

| ID | Kind | Exact methods | Doubles / input | Expected result |
|---|---|---|---|---|
| B01 | REC | `SetBaseValue` | writable=false; base=4, set=9 | base=4; dispatch=0 |
| B02 | REC | `SetBaseValue` | mirror update; set=-3.7 then 4.9 | base=0 then 4; update=1; dispatch=1 |
| B03 | REC | `SetBaseValue` | `attrsNotClampZero={A}`; set=-3.7 | base=-4; update=1; dispatch=1 |
| B04 | REC | `SetBaseValue`, `GetMaxValue`, `RefreshBounds` | max A->B=100; max-only, upper-only `0.5*100+5`, lower-only `0.75*100`, then conflict | 120->100; 120->55; 20->75; 120->75; collector=4; update=3; dispatch=3 |
| B05 | REC | `SetBaseValue` | base=current=5/7; set=5 | base=5; update=0; dispatch=0 |
| B06 | REC | `SetBaseValue`, `DispatchCurrentValueEvent` | explicit update sets current=base+1; base=2/current=3/set=7.5 | base=7/current=8; trace update,dispatch; dispatch(8,3) |
| B07 | REC | `SetBaseValue`, `DispatchCurrentValueEvent` | update no-op; base=3/current=0/set=7, then (5,5),(6,5) | implement=0,0,1 |
| B08 | REC | `SetBaseValue` | update sets 8 then throws; base=2/current=3/set=7.5 | base=7/current=8; dispatch=0 |
| B09 | REC | `SetBaseValue`, `DispatchCurrentValueEvent` | update sets 8; notify throws; base=2/current=3/set=7.5 | base=7/current=8; dispatch attempted=1 |
| B10 | REC | `AddBaseValue`, `SetBaseValue` | mirror update; base=-5 add=0; base=4 add=2.9 | base=0 then 6; update=2; dispatch=2 |
| B11 | REC | `SetBaseValue`, `AddBaseValue` | writable=false; invalid=200 | set avoids access; add throws before permission |
| B12 | REC | `SetBaseValue`, `GetMaxValue`, `RefreshBounds` | writable; max invalid->B=100; upper=0.5; set invalid=50 | throws after permission+collector; update=0 |
| A01 | REC | `ModifyStateAttribute`, `AddBaseValue` | policy 0; V1=2.5; stack=2; level=1 | base=5; update=1; dispatch=1 |
| A02 | REC | `ModifyStateAttribute`, `SetBaseValue` | policy 3; V1=7.9; stack=5; level=1 | base=7 (floor); stack ignored |
| A03 | REC | `ModifyStateAttribute`, `AddBaseValue` | policy 5; V1=4; time=0.5; stack=3; base=1 | base=7 |
| A04 | REC | `ModifyStateAttribute` | policy 2 snapshot=100; min=20; ratio=2; floor; max=30; V1=10000; stack=2 | base=60; log=0 |
| A05 | REC | `ModifyStateAttribute`, `SetBaseValue` | policy 4 snapshot=10; V1=10000; stack=3; base=5 | base=10; stack ignored |
| A06 | REC | `ModifyStateAttribute`, `AddBaseValue` | policy 9 snapshot=10000; V1=10000; base=10 | base=20 |
| A07 | REC | `ModifyStateAttribute`, `AddBaseValue` | policy 0; V1=5; stack=0; base=3 | base=3; update=0; dispatch=0 |
| A08 | REC | `ModifyStateAttribute`, `SetBaseValue` | policy 4 snapshot=10; stack=0; base=5 | base=10; override not stack-scaled |
| A09 | REC | `ModifyStateAttribute`, `AddBaseValue` | policy 2 snapshot=0; V1=10000; V2=5; source B base/current=20 | base=5; zero snapshot beats fallback 20 |
| A10 | REC | `ModifyStateAttribute`, `GetAttrValue`, `AddBaseValue` | policy 2 snapshot=null; source B base=20; V1=10000 | base=20; live read |
| A11 | REC | `ModifyStateAttribute`, logging branch | policy 2 selector=instigator; instigator=null; snapshot=10; base=3 | base=3; dispatch=0; log=1; attribute pair |
| A12 | REC | `ModifyStateAttribute` | policy null / empty / `[2]`; base=0 | NRE / IOOR / IOOR; storage unchanged |
| A13 | REC | `ModifyStateAttribute` | valid policy 0, then null policy; base=0 | first write 5 survives; dispatch=1 |
| A14 | DIR | `GetLevelValue<T>` | arrays null/empty/3 entries; levels 0,1,2,3,99,-1 | default, first, second, third, last, last |
| A15 | DIR | `GetAttrValue` | base=10/current=13; Base/Current/Bonus/null/invalid | 10,13,3,0,10 |
| A16 | REC | `ModifyStateAttribute` | C not in stateAttributeIds; policy null | early return; no permission/log |

The historical receipt reports **28 of 28 cases passed, 115 assertions, and 0
synthetic-model tests**. The earlier preparation checkpoint said
`NOT COMPILED, NOT EXECUTED`; the later receipt supersedes execution status only.
The receipt and complete private run layout are not distributed; the selected
fixture-source view is linked below. This result
was not independently reproduced from the public files.

A pass means the selected slices ran under the specified authored hosts and
doubles. It does not establish real permission or runtime-bound update behavior,
game values, full classes, source/CIL equivalence, or game-runtime behavior.

## Included navigation

- [Authored harness tests](../../../../tests/managed-recovered/buff-attribute/HarnessTests.cs)
- [Authored test doubles](../../../../tests/managed-recovered/buff-attribute/TestDoubles.cs)
- [Qualified buff fixture](../../../../qualified-historical/tools/re/local-recovery/buff-direct-test01/scratch/project/RecoveredFixture.cs)
- [Current source and evidence index](../../../SOURCE_INDEX.md)
- [Reproduction boundaries](../../../REPRODUCIBILITY.md)
