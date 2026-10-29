# Gameplay showcase evidence

This is the evidence record for the gameplay showcase. It preserves the stable
method identities, observed behavior, historical execution counts, and
qualifications behind the shorter reader page. It is not a new run, a new
extraction, or a new source review.

Authority: `CSharpScript` SHA-256
`e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, MVID
`2d6d37cb-76e5-4075-8220-3eeca41471e3`. Game version is `UNKNOWN`. Class views
are partial, not original source; compilation does not upgrade a label.
The historical checkpoint recorded publication as `DISABLED`, full backup as
`NOT ESTABLISHED` and runtime as `PARKED`. Publication preparation does not
resolve redistribution rights or upgrade those research qualifications.

The public selection contains curated overlays, two fuller historical class
captures and seven source-derived fixture views. The
[qualified historical source map](README.md#qualified-historical-source-map)
links the included bodies. The complete private canonical corpus, snapshot CIL,
preparation manifests and execution receipts are not distributed. Included
fixture source is not new execution or authentication evidence. A method identity is the full
authority SHA-256, MVID, TypeDef, and MethodDef tuple; a name alone is not an
identity.

## Labels

- `DIRECT_CIL_SUPPORTED` - accepted whole-body CIL, with fidelity status.
- `SAVED_SOURCE_SUPPORTED` - exact saved source; no new whole-body CIL proof.
- `DIRECT_EXECUTION_TESTED` - exact slice under an authored harness; not class
  or runtime proof.
- `DEPENDENCY_DOUBLE_CONDITIONAL` - result depends on a stub, double, or
  synthetic config.
- `STATICALLY_SUPPORTED_ONLY` - source or CIL only; no execution coverage.
- `UNKNOWN` - unsupported or unresolved.

Labels are orthogonal, not ranked. A proven branch is in the slice and asserted
(normalized-base equality skip, dispatch `old != new`). A stub condition is
harness-supplied (permission, update, bounds collection, dispatch
implementation, synthetic data). A pass cannot show game behavior for a stub
condition.

## Slice at a glance

```text
buff harness (exercised): synthetic modifier -> ModifyStateAttribute
  -> SetBaseValue / AddBaseValue -> BaseValues
  -> UpdateCurrentValue (double) -> CurrentValues
  -> DispatchCurrentValueEvent -> dispatch implementation (double)

snapshot harness S08 (exercised value flow): controlled arrays -> TakeSnapshot
  -> GetAttrFromSnapshots -> AttributeSnapshot.GetCurrentValue

formula (static in the showcase checkpoint): CalculateHurt
  -> AttributeSnapshot.GetCurrentValue
```

These are separate exercised or static pieces, not one integrated call graph.
The buff harness does not establish snapshot capture; no cited edge connects
dispatch to `TakeSnapshot`. No tested real buff, config, or capture to
`CalculateHurt` join is established.

## What is distributed

The curated C# overlays are partial. This table separates a stable method
identity from whether its body is actually in the repository.

| Type (TypeDef) | Method (MethodDef) | Body in this repository |
| --- | --- | --- |
| `BaseAttributeComponent` (`0x02002e53`) | `IsWritableAttribute 0x060184ba` | Yes: `BaseAttributeStorage.cs:78` |
| `BaseAttributeComponent` (`0x02002e53`) | `SetBaseValue 0x060184bc` | Yes: `BaseAttributeStorage.cs:120` |
| `BaseAttributeComponent` (`0x02002e53`) | `AddBaseValue 0x060184bd` | Yes: `BaseAttributeStorage.cs:148` |
| `BaseAttributeComponent` (`0x02002e53`) | `GetCurrentValue 0x060184bf` | Yes: `BaseAttributeStorage.cs:156` |
| `BaseAttributeComponent` (`0x02002e53`) | `DispatchCurrentValueEventImplement 0x060184e2` | Yes: `BaseAttributeStorage.cs:203` |
| `BaseAttributeComponent` (`0x02002e53`) | `UpdateCurrentValue 0x060184c4` | Yes: `BaseAttributeComponent.cs:27` |
| `BaseAttributeComponent` (`0x02002e53`) | `DispatchCurrentValueEvent 0x060184e1` | Yes: `BaseAttributeComponent.cs:310` |
| `BaseAttributeComponent` (`0x02002e53`) | `GetBaseValue 0x060184be`, `GetMaxValue 0x060184c2`, `RefreshBounds 0x060184d0` | Yes: qualified historical buff fixture; absent from the curated overlays |
| `BaseAttributeComponent` (`0x02002e53`) | `TakeSnapshot 0x060184c5` | Yes: qualified historical snapshot fixture; historical execution is not a new run |
| `ActiveBuffInternal` (`0x02002e71`) | `ModifyStateAttribute 0x06018716` | Yes: qualified historical buff fixture and fuller ActiveBuffInternal capture; not a new accepted curated method |
| `AbilityUtils` (`0x02002e4e`) | `GetLevelValue<T> 0x0601848d`, `GetAttrValue 0x0601848f` | Yes: qualified historical buff fixture; the level helper is repeated in another fixture |
| `AttributeSnapshot` (`0x02002e52`) | `GetCurrentValue 0x060184b2` | Yes: qualified historical snapshot/getter fixtures |
| `Calculation` (`0x02002eb2`) | `GetAttrFromSnapshots 0x06018865` | Yes: `Calculation.DamageFormula.cs:39` |
| `Calculation` (`0x02002eb2`) | `CalculateHurt 0x06018867` | Yes: `Calculation.DamageFormula.cs:55` |
| `Calculation` (`0x02002eb2`) | five element/attack helpers `0x06018868`-`0x0601886c` and `CalculateFormula 0x06018873` | Yes: `Calculation.DamageFormula.cs:88-230` |
| `Calculation` (`0x02002eb2`) | `CalculateElementMatchUpRate 0x06018872` | Yes: qualified historical damage slices. The token is the historical damage-preflight correction; the publication mapping's unresolved MethodDef is not promoted by this page. |
| `BaseBuffComponent` (`0x02002e55`) | `ApplyPeriodExecution 0x0601854d` | Yes: fuller historical BaseBuffComponent capture; the curated overlay has only the call at `BaseBuffComponent.cs:223` |

The distributed formulas and setters are readable source-like overlays, not a
standalone project. `AttributeSnapshot.GetCurrentValue` is cited as raw method
row 221, not map row 221; the selected getter is `0x060184b2`, not
`BaseAttributeComponent.GetCurrentValue 0x060184bf`.

## Mechanics

| # | Mechanic | Established behavior | Labels and limits |
|---:|---|---|---|
| 1 | Separate storage | `BaseValues` and `CurrentValues` are separate `float[143]`. Take copies base then current; `GetAttrValue` returns base, current, or current minus base. | Take is `DIRECT_CIL_SUPPORTED`; getters are `SAVED_SOURCE_SUPPORTED`; A15/S01/S05 are `DIRECT_EXECUTION_TESTED`. The selected Take body is now included in a qualified historical fixture. The execution label still describes the earlier private run. |
| 2 | Value helpers | `GetLevelValue<T>`: null or empty returns default; 0 returns first; positive is 1-based; out-of-range or negative returns last. `GetAttrValue`: null set returns 0, then Base, Current, Bonus. | `SAVED_SOURCE_SUPPORTED`; A14/A15 `DIRECT_EXECUTION_TESTED`, no stub dependency. Both helper bodies are included in the qualified historical buff fixture. |
| 3 | Set permission | `SetBaseValue` enters only if `IsWritableAttribute(attrId)`. The imported check maps auto-recover maps, Tough, Jump, and special energy; otherwise false. | The permission and Set bodies are distributed and `DIRECT_CIL_SUPPORTED`. B01/B11/B12 are `DEPENDENCY_DOUBLE_CONDITIONAL` because the predicate is doubled. Actual writability of damage-stat targets is `UNKNOWN`; do not claim buff write access. |
| 4 | Bounds and normalization | Optional max triggers `GetMaxValue`/`RefreshBounds`; upper min, lower max last (lower wins); clamp zero unless `attrsNotClampZero`; `Floor`; equal normalized base skips write, update, and dispatch. | `SetBaseValue` is `SAVED_SOURCE_SUPPORTED` and `DIRECT_EXECUTION_TESTED` (B02-B05, B10). The max map, lockers, and permission are doubles. The helper bodies are now included in the qualified historical buff fixture; the dependency qualifications remain. |
| 5 | Write order and event | Base is written before virtual `UpdateCurrentValue`; dispatch runs only when old current differs from new. | The `UpdateCurrentValue` and `DispatchCurrentValueEventImplement` bodies are distributed, but the buff harness substituted both with doubles. Only the `DispatchCurrentValueEvent` wrapper was executed. B06-B09 are `DIRECT_EXECUTION_TESTED`; B08/B09 show partial state after exceptions. Runtime-bound virtual behavior remains `UNKNOWN`. |
| 6 | Add reads old base | `AddBaseValue` reads `BaseValues[(int)attrId]` before Set, so an invalid index throws before permission (B11). B12 is the writable invalid Set, which reaches bounds and then throws on the equality array read before any write. B10 zero add normalizes -5 to 0 from the old base. | `DIRECT_CIL_SUPPORTED`, `DIRECT_EXECUTION_TESTED`. The Add and Set bodies are distributed. |
| 7 | Modifier policies | 0 adds V1 times stack; 1 multiplies base by `1 + V1 * 0.0001 * stack`; 3 sets V1 without stack; 5 adds V1 times time dilation times stack; 2, 4, and 9 share derived reads (9 is a rate); 6 is time-scaled base; 7 and 8 are no-ops. A null-coalescing snapshot preserves zero; the selected-source guard runs before snapshot use. The real `ApplyPeriodExecution` calls this only under `HasBuffAuthority`. | `SAVED_SOURCE_SUPPORTED`; A01-A13/A16 are `DIRECT_EXECUTION_TESTED` and `DEPENDENCY_DOUBLE_CONDITIONAL` for synthetic config. ModifyStateAttribute is now included in the qualified fixture and fuller ActiveBuffInternal capture. ApplyPeriodExecution (`0x0601854d`) is in the fuller BaseBuffComponent capture, not the curated overlay, and was not executed by this harness. |
| 8 | Snapshot capture | Take allocates and copies 143 base then 143 current separately; no recompute or atomicity. The getter returns current. Arrays are public readonly but writable; tests do not prove live immutability. | Take is `DIRECT_CIL_SUPPORTED`; getter is `SAVED_SOURCE_SUPPORTED`; S01-S05 are `DIRECT_EXECUTION_TESTED`. Both selected bodies are now included as qualified historical fixture source; the harness run remains historical. |
| 9 | Helper guard, getter, and owner | `GetAttrFromSnapshots`: an index below `Lv` or at/above `Max` returns 0 before snapshot access; a valid index passes to the selected getter. `AttributeSnapshot.GetCurrentValue` returns `CurrentValues[(int)attrId]` with no internal guard, so raw 0 and 142 are accepted and -1/143 throw. Owner 0 is attacker; every nonzero owner is target. The getter's float value becomes double. | `GetAttrFromSnapshots` is `DIRECT_CIL_SUPPORTED` and distributed at `Calculation.DamageFormula.cs:39`; S06-S08 are `DIRECT_EXECUTION_TESTED`. This is the real helper, not a selector stub. The snapshot getter is also present in qualified historical fixtures. |
| 10 | Formula and attribution | `CalculateHurt` reads target `Def` (10), attacker `DamageChange` (15), and `CritDamage` (9) unconditionally. `CritDamage` is used as a multiplier only when critical; otherwise the multiplier is 1.0. These reads use snapshot getters directly, not `GetAttrValue`. The included damage snapshot excerpts capture both attribute snapshots before conditional attacker replacement and payload assignment. This does not establish a caller-to-`CalculateHurt` join or receiver identity at formula time. | `CalculateHurt` retains its `DIRECT_CIL_SUPPORTED` qualification. The additional excerpts are `SOURCE_ONLY`, with selected CIL call-site checks, not whole-body equivalence. Actual permission, live HP application and server authority remain `UNKNOWN`. |

`CritDamage` is read unconditionally and used conditionally. Do not turn that
read into evidence that a critical hit occurred.

The [damage snapshot walkthrough](../../../docs/DAMAGE_SNAPSHOT_FLOW.md) links
the five included excerpts and their bounded CIL evidence. Replacement requires
a non-null control handle and `ConcomitantPhantomRole`. Counter processing
precedes critical modifiers. Selected successful responses copy `Damage` and
`ChangeLife` fields and reach validity-gated events and effects; those source
edges do not identify an HP-writing consumer or prove server validation or
replication. They are separate from the historical harness results below.

## Method coverage and execution

Thirteen identities were exercised by the two historical harnesses, not a
13-method CIL selection:

| Class (TypeDef) | Exercised identities | Static coverage | Harness cases |
|---|---|---|---|
| `BaseAttributeComponent` (`0x02002e53`) | `SetBaseValue 0x060184bc`, `AddBaseValue 0x060184bd`, `GetCurrentValue 0x060184bf`, `DispatchCurrentValueEvent 0x060184e1`, `TakeSnapshot 0x060184c5`; `GetBaseValue 0x060184be`, `GetMaxValue 0x060184c2`, `RefreshBounds 0x060184d0` are context | Set/Add historical direct CIL; Take selected CIL; GetMax/Refresh and GetBase source context; Dispatch source only. Selected fixture bodies are now included without changing those labels. | buff B01-B12, A01-A13, A16; snapshot S01-S05, S08 |
| `ActiveBuffInternal` (`0x02002e71`) | `ModifyStateAttribute 0x06018716` | `UNSELECTED_SAVED_SOURCE`; included as qualified historical source | buff A01-A13, A16 |
| `AbilityUtils` (`0x02002e4e`) | `GetLevelValue<T> 0x0601848d`; `GetAttrValue 0x0601848f` | `UNSELECTED_SAVED_SOURCE`; included as qualified historical fixture source | buff A14/A15 |
| `AttributeSnapshot` (`0x02002e52`) | `GetCurrentValue 0x060184b2` | Historical source context with metadata binding; included as qualified historical fixture source | snapshot S01, S03-S08 |
| `Calculation` (`0x02002eb2`) | `GetAttrFromSnapshots 0x06018865` | Current full CIL / source-fidelity pass; distributed | snapshot S06-S08 |

`GetAttrFromSnapshots` is declared by `Calculation`, not
`BaseDamageComponent`. The selected `AttributeSnapshot.GetCurrentValue` is
`0x060184b2`; it is not the `BaseAttributeComponent` getter
`0x060184bf`. The current accepted source index records D as eight selected
methods with 1 `DIRECT_CIL_FULL`, 1 `SOURCE_ONLY`, and 6 historical `UNKNOWN`
identities. The later execution result covers nine saved bodies; it does not
promote the six unknown methods or change the evidence class of the two
saved-source/context views.

`Aki.Generated` is separate from `CSharpScript`. Its `CritDamage=9`, `Def=10`,
and `DamageChange=15` values are enum context only; no buff targets, caps, or
values follow.

## Execution records

| Harness or model | Recorded result | Scope |
| --- | --- | --- |
| [Buff and attribute history](../../../docs/history/harnesses/buff-attribute/README.md) | 28 of 28 cases passed, 115 assertions, 0 synthetic models | 26 cases use recovered source with stubbed dependencies plus 2 direct helper tests; 10 exact slices; private prepared fixture |
| [Attribute snapshot history](../../../docs/history/harnesses/attribute-snapshot/README.md) | 8 of 8 groups passed, 351 assertions, 0 synthetic models | 3 exact slices; authored hosts, arrays, constructors, payloads, and enum adapters |
| Buff plus snapshot | 36 historical cases or groups, 13 method identities, 466 assertions | Historical combined scope, not a current integration suite |
| [Damage formula result](../../../docs/history/harnesses/damage-formula/RESULTS.md) | 8 groups, 39 assertions, 0 failures | 9 saved bodies: 7 new tested identities plus 2 previously exercised |
| Transformed damage dispatch | 8 groups, 50 assertions, 0 failures | Explicitly transformed `CalculateFormula`; one changed log span; not the unchanged recovered body |
| Skill interruption | 8 groups, 127 assertions, 0 failures | Four exact `BaseSkillComponent` slices under controlled dependency doubles |
| Event 101 reference | 10 of 10 groups, exit 0 | Synthetic model execution only; no per-assertion count; no public offline build verified |

The buff and snapshot results are historical. The damage result is later and
separate. The transformed dispatch and skill results are later again and do not
join the showcase. The damage result covers `CalculateHurt`, six calculation
helpers new for that task, `GetAttrFromSnapshots`, and
`AttributeSnapshot.GetCurrentValue`. It does not establish the caller,
registries, HP application, server authority, or a buff-to-damage route.

Private receipts and preparation manifests are not distributed; selected fixture
source views are included. These are recorded
observations, not publicly reproducible checks. No compiler, build, test, or
runtime command was run for this page.

## Reproducibility and gaps

- There is no authority-compatible buff policy, target set, or value. The
  configs are synthetic.
- Permission for actual damage-stat targets and runtime-bound update behavior
  are unknown. The permission predicate, virtual update, dispatch
  implementation, and parts of bounds/max input are doubled. The later
  transformed fixture does not execute the unchanged recovered body.
- Live buff-to-capture connection and timing are unknown. Receiver identity at
  formula time is not established.
- The route is conditional, not full buff damage. Selected response-field copies
  and gated callbacks are source-supported; their connection to local formula
  output, actual HP writes, server validation and replication remains unknown.
- No gameplay runtime was used. Game version is `UNKNOWN`.
- The damage-formula checkpoint's executed Hurt slice has no rounding or
  ceiling step and no random seed input. The original unchanged recovered
  `CalculateFormula` and the original formula/seed/round path remain static-only
  in that checkpoint and were not compiled there. The later, separate
  transformed `CalculateFormula` fixture is not the unchanged body; it does not
  establish the original dispatcher, seed distribution, or round policy. No
  runtime upgrade is claimed.
- Full backup was not established at the historical checkpoint. Publication
  preparation does not establish redistribution permission.
- The earlier statement that `CalculateHurt` was static only is bound to the
  showcase checkpoint. The later damage-formula result was executed separately
  and does not promote the old qualifications, the dispatch implementation,
  the seed, or the integration questions.
- Event 101 is separate synthetic work. It is not a recovered route or part of
  this chain, and it provides no buff or snapshot evidence.

## Related work

Event 101 is separate synthetic work:
[README.md](../event101-reference/README.md),
[Event101Model.cs](../event101-reference/Event101Model.cs),
[Event101Tests.cs](../event101-reference/Event101Tests.cs), and
[Event101Reference.csproj](../event101-reference/Event101Reference.csproj).
The model is not a recovered route or part of this chain. A private reference
run reported 10 of 10 groups passing with exit 0; per-assertion counts were not
instrumented, and there is no buff or snapshot evidence.

## Link scope

All links on this page resolve inside the selected repository. The complete
canonical corpus, snapshot CIL, private receipts and private-only aliases are not
distributed. The source map links only the selected historical captures and
fixtures actually included here. Their presence supplies readable representations,
not original-binary authentication or a new execution result.
