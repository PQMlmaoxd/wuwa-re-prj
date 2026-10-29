# Damage formula harness history

Editorial successor to historical path
`tests/managed-recovered/damage-formula/README.md`; not a byte-identical
replacement. The saved predecessor SHA-256 is
`de56620fca5f2f9c5a1055beb60278a7f5598d01b674a71b51f52a52c477ed4e`. The
predecessor described a prepared candidate. The separate [execution result](RESULTS.md)
records a later authorized preflight-04 run. Neither page is a new verification
or execution receipt.

## Saved-body scope

The historical harness ran nine saved source-like bodies under authored hosts.
The common managed authority was SHA-256
`e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, MVID
`2d6d37cb-76e5-4075-8220-3eeca41471e3`; game version is `UNKNOWN`. The canonical
input set and private generation manifests are not distributed. Selected
fixture-source views are included through the links below, without the complete
historical run layout.

| Method | TypeDef / MethodDef | Canonical source anchor | Historical qualification |
|---|---|---|---|
| `Calculation.GetAttrFromSnapshots` | `0x02002eb2 / 0x06018865` | `Calculation.cs:20-28` | `DIRECT_CIL_SUPPORTED` (R38 row 289) |
| `Calculation.CalculateHurt` | `0x02002eb2 / 0x06018867` | `Calculation.cs:30-56` | `DIRECT_CIL_SUPPORTED` (R38 row 288) |
| `Calculation.GetElementResistant` | `0x02002eb2 / 0x06018868` | `Calculation.cs:58-71` | `DIRECT_CIL_SUPPORTED` (R38 row 290) |
| `Calculation.GetElementIgnoreResistance` | `0x02002eb2 / 0x06018869` | `Calculation.cs:73-86` | `DIRECT_CIL_SUPPORTED` (R38 row 291) |
| `Calculation.GetElementDamageReduce` | `0x02002eb2 / 0x0601886a` | `Calculation.cs:88-101` | `DIRECT_CIL_SUPPORTED` (R38 row 292) |
| `Calculation.GetElementDamageBonus` | `0x02002eb2 / 0x0601886b` | `Calculation.cs:103-116` | `DIRECT_CIL_SUPPORTED` (R38 row 293) |
| `Calculation.GetAttackTypeDamageBonus` | `0x02002eb2 / 0x0601886c` | `Calculation.cs:118-130` | `DIRECT_CIL_SUPPORTED` (R38 row 294) |
| `Calculation.CalculateElementMatchUpRate` | `0x02002eb2 / 0x06018872` | `Calculation.cs:216-229`; raw method row 655 | `SAVED_SOURCE_SUPPORTED`, unselected helper |
| `AttributeSnapshot.GetCurrentValue` | `0x02002e52 / 0x060184b2` | `AttributeSnapshot.cs:14-17` | `SAVED_SOURCE_SUPPORTED`, context with metadata binding |

`Calculation.cs` was 24,688 bytes, SHA-256
`daa9fff61d83566475f474ab84e2feea1b1b7de987a08a678cc0c87a32ea6e34`;
`AttributeSnapshot.cs` was 372 bytes, SHA-256
`3e2cd1b7799363521318c126faaf27e4ff2e24278e72761f7243b6158b7dd5da`.
These identify historical inputs, not files included here. The seven selected
Calculation bodies retain their classifications; compilation and execution do
not promote them. The two saved-source/context views are likewise unchanged.

## Authored hosts and assumptions

`TestScaffolding.cs` contains no recovered method body. It supplied partial hosts,
enum adapters, and a `SnapshotPayload` with the two properties used by the
selected methods. Each snapshot had separate 143-element `float` arrays. Tests
wrote synthetic values directly; no `TakeSnapshot`, game configuration, Buff,
extracted asset, or runtime value was used.

The `Aki.Protocol.EAttributeType` adapter followed an `int32`-backed external
enum identity, SHA-256
`19f70a3cbe8a306424b860f2174152fd7986dd0133f7c6268180e8752d302ea1`, MVID
`4ce2a693-a46e-45bc-bfb3-7a8bf9085389`. It exposed the 43 names referenced by
the bodies. Retained literals stop at `DamageChangeQte=20`; additional numeric
associations come from R38 CIL operands, not a complete enum table. `Max=143`
is an adapter alias for the guard operand, not a retained enum literal. The
same-short-name byte-backed type in another shard is a different type and is not
authority for these bodies. `EElementType` has no namespace declaration in its
shard view; the adapter follows the namespace in the `CalculateHurt` metadata
signature, with that binding left as an assumption. The owner and attack enums
follow their saved shard names and member order.

## `CalculateHurt` expression trace

The order below follows `Calculation.cs:30-56`. `K` is the double value of the
single-precision literal `1e-4f`, exactly
`9.9999997473787516E-05` (`0x3F1A36E2E0000000`). Float operations remain float
until the stated promotions. The trace is static description of the saved body;
no game execution or independent source verification is claimed.

| # | Expression or operation | Type, order, and limit |
|---:|---|---|
| 1 | Read `snapshots.AttackerSnapshot` | Property read and reference assignment; no copy. |
| 2 | Read `snapshots.TargetSnapshot` | Property read; nullable. This is not yet a target dereference. |
| 3 | `num = attrRate * K` | `double * double`; `attrRate` may come from `AbilityUtils.GetLevelValue` in the unexecuted caller. |
| 4 | `GetElementDamageBonus(attacker, element)` | Switch on element; attacker `DamageChange{Phys,Element1..6} * 0.0001f`; default `0f`; float result promoted on return. No cap. |
| 5 | `num2 = Math.Min(GetElementDamageReduce(target, element), 1.0)` | Target is first dereferenced here. Helper uses target `DamageReduce{Phys,Element1..6}`; upper cap only. Negative values pass through. Float product is promoted, then capped as double. |
| 6 | `GetElementResistant(target, element)` | Target `DamageResistance{Phys,Element1..6}`; switch default `0f`; float product then promotion. |
| 7 | `GetElementIgnoreResistance(attacker, element)` | Attacker `IgnoreDamageResistance{Phys,Element1..6}`; switch default `0f`; float product then promotion. |
| 8 | Compute `R = resistance - ignore`, then `num3` | Double arms: `R <= 0` gives `1 - R/2`; else `R < 0.8` gives `1 - R`; otherwise `1/(1 + R*5)`. NaN reaches the final arm. |
| 9 | `GetAttackTypeDamageBonus(attacker, attackType)` | Attacker `DamageChange{Auto,Cast,Ultra,QTE,NormalSkill,Phantom}`; switch default `0f`; float result promoted on return. |
| 10 | `GetAttrFromSnapshots(snapshots, Attacker, attrId)` | Guard `attrId < Lv || attrId >= Max` is inside this helper and precedes its own snapshot access. It does not precede the target accesses in steps 5-6. A float getter result is promoted to double. |
| 11 | `targetSnapshot.GetCurrentValue(Def)` | Reads target `CurrentValues[10]` as float. A null target can already fail at step 5 for the element arms. |
| 12 | `num4 = attacker[IgnoreDefRate] * 0.0001f` | Float multiplication at index 99; no clamp. |
| 13 | `currentValue2 = attacker[Lv]` | Float read at index 1. |
| 14 | `num5 = Math.Min(2f, 1f / (def * (1f-num4) / (800f+lv*8f) + 1f))` | Entire expression is float; upper cap 2f only. No lower clamp or zero-division guard; `1f/0f` yields positive infinity. |
| 15 | `num6 = attacker[DamageChange] * 0.0001f` | Float multiplication at index 15. |
| 16 | `num7 = (double)(1f + num6) + elementBonus + attackTypeBonus` | First addition is float, then promoted; the two bonuses are added as double. Terms are additive, not multiplicative. |
| 17 | Read target `[ElementPropertyType]` and cast to `EElementType` | Float-to-enum conversion at index 97. The local is read but does not affect the result. |
| 18 | `num8 = Math.Min(target[DamageReduce] * 0.0001f, 1.0)` | Float multiplication, promotion, then double `Math.Min`; upper cap only, no lower clamp. |
| 19 | `num9 = 1f + attacker[SpecialDamageChange] * 0.0001f` | Float expression at index 95; no clamp in this method. |
| 20 | `num10 = CalculateElementMatchUpRate(element, targetElementType)` | Every arm of the unselected saved-source helper returns `1.0`; helper is not replaced by a constant in the harness. |
| 21 | `num11 = attacker[CritDamage] * 0.0001f` | Float multiplication at index 9; read unconditionally. |
| 22 | `num12 = isCritical ? (double)num11 : 1.0` | Double conditional. The boolean selects use of the already-read value; no RNG roll occurs here. |
| 23 | `(num*attr + augment + formula) * num12 * (double)num5 * num7 * num3 * (1.0-num8) * (1.0-num2) * (double)num9 * num10 * gameplayRatio * Math.Max(1.0 + amplify*K, 0.0)` | Strict left-to-right double product; no regrouping. The amplify factor is last and floored at zero. |
| 24 | `Math.Max(0.0, val)` | Returns positive zero for a negative value. NaN remains NaN. No rounding or integer conversion. |

Rows 4-10 and 11/20 call the separately classified helper bodies listed above.
The body has no state write, log, lock, seed update, RNG call, or `Math.Ceiling`;
the getter calls and a possible null-reference exception are the only described
observable effects. The `CalculateHurt` result is a raw double, not applied HP
damage.

## Static caller context, not executed

`Calculation.CalculateFormula` (`0x06018873`, `Calculation.cs:231-276`, R38 row
295) was not in the fixture. Its saved source records this surrounding path:

1. Read the input damage data, skill level, formula type, and calculate type.
2. For `formulaType == 0`, convert `RelatedProperty` and call
   `AbilityUtils.GetLevelValue(damageData.RateLv(), skillLevel, 0)`.
3. For `calculateType == Hurt`, call `CalculateHurt` with the selected elements,
   attack type, related property, rate, critical flag, augment, amplify, and
   gameplay ratio; the optional `formula` argument defaults to `0.0`. The other
   branch calls `CalculateHeal`.
4. For nonzero formula type, look up `Formulas`. A missing key logs and returns
   `0.0` before seed access. A present delegate receives level-evaluated inputs
   and the additional level, critical, augment, amplify, accumulation, multi-ratio,
   and gameplay-ratio values.
5. After a calculation returns, save the old seed, store the next seed, read the
   fluctuation arrays, compute `(float)((upper-lower)*(oldSeed % 10000)+lower) *
   0.0001f`, apply `Math.Ceiling`, then negate only for exact `Heal`.

These are source-context observations, not tests. A later fluctuation getter or
cast failure may leave the seed already changed. The old signed seed is used with
`% 10000`; no division by 10000 is shown. `ForceCritical` and `response.IsCrit`
do not identify the producer of `CalculateHurt`'s `isCritical` argument. No
producer, caller wiring, registry behavior, request construction, HP application,
or server authority is established. Formula type 1 is not re-tested here.

## Case groups and oracle notes

The historical harness used eight named groups, D01-D08, with 39 bit-pattern or
predicate assertions. Inputs were synthetic authored values; expected results
were derived from the expressions, not calculated by calling the tested code.

| Group | Behavior covered |
|---|---|
| D01 | `attrRate*K`; critical false/true; unconditional `CritDamage` read versus conditional use; NaN ignored when noncritical and retained when critical. |
| D02 | Defense factor at defense 800 / level 0; ignore-defense rate; negative defense and the 2f cap; negative final factor and final zero floor. |
| D03 | Damage change, element bonus, and attack bonus are additive; attacker-only values; unknown attack selector defaults to zero. |
| D04 | Resistance arms for `R <= 0`, `0 < R < 0.8`, and `R >= 0.8`; ignore-resistance subtraction. `R=1.0` gives `1/6`. |
| D05 | General-reduction upper cap and negative value; amplify factor and zero floor; gameplay ratio; uncapped special damage change. |
| D06 | Related attribute comes from attacker; invalid IDs return zero inside helper; defense comes from target; `formula` is added before multipliers. |
| D07 | Raw double return: fraction preserved, negative becomes positive zero, NaN and positive infinity retained, null target throws. |
| D08 | Repeat-call bit stability and unchanged arrays; changing inert `ElementPropertyType` has no effect; changing `Def` does. |

For D01a, `10000*K = 0.9999999747378752` (`0x3FEFFFFFF2700000`) and
`(10000*K)*128 = 127.99999676644802` (`0x405FFFFFF2700000`); all other factors
are 1.0. D01b halves this exact value with `0.5f`. Other recorded derivations:
D04c, `R=1.0`, produces `1/6 * 64 = 10.666666666666666`; D05c gives
`(1 + 10000*K)*64 = 127.99999838322401`; D06a gives
`(10000*K)*5000 = 4999.999873689376`; D08f gives a float defense factor
`0.6666666865348816f`, then `*64 = 42.66666793823242`.

The raw value 8000 scales to `0.7999999523162842f`, below 0.8. It was not used
to claim the third resistance arm; that arm uses `R=1.0`. `--prepare-only`
recomputed five oracle constants with the Python standard library and checked
that their bit-pattern literals were present. That historical check was a spot
check of oracle literals, not another formula implementation.

## Recorded errata and limits

- `CalculateElementMatchUpRate` is MethodDef `0x06018872`. The earlier
  `0x06018722` was a transcription error; the correction used raw method row 655
  (RVA 7212772, code size 231 bytes), not a CIL reread.
- The retained enum spelling is `DamageChangeQte=20`. `DamageChangeQTE` belongs
  to a different byte-backed type; no spelling drift in the copied body was found.
- Target `ElementPropertyType` is read and passed to a helper that ignores it.
  D08 tests this inert-but-read dependency separately from `Def`.
- `8000 * 0.0001f` does not establish the `R >= 0.8` branch; see D04 above.

## Included navigation

- [Historical execution result](RESULTS.md)
- [Authored harness tests](../../../../tests/managed-recovered/damage-formula/HarnessTests.cs)
- [Authored test scaffolding](../../../../tests/managed-recovered/damage-formula/TestScaffolding.cs)
- [Qualified damage slices](../../../../qualified-historical/tools/re/local-recovery/damage-formula01/scratch/project/src/Calculation.RecoveredBodies.cs)
- [Current source and evidence index](../../../SOURCE_INDEX.md)
- [Reproduction boundaries](../../../REPRODUCIBILITY.md)
