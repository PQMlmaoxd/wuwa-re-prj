# Damage reference diagnostic

Public-copy context: the status and execution gates below are the original model
checkpoint, not the latest execution status of every related project. The model
is independently authored; later recovered-fixture results are separate. Selected
fixture bodies now appear in the
[historical source map](../gameplay-showcase/README.md#qualified-historical-source-map),
but private preparation manifests, original captures and execution receipts do
not become public through that inclusion. No runtime-equivalence claim is added.

This directory contains a small, independently authored .NET 8 numeric diagnostic. It applies a caller-supplied amount, seed callback, integer bounds, float extra rate, and sign flag to the finalization steps represented in the selected static `CalculateFormula` source view. It is not reconstructed or decompiled source, a player calculator, a game simulator, or a claim of CIL, binary, runtime, server, or gameplay parity.

## Status

| Item | State |
| --- | --- |
| Model and authored test runner | PREPARED |
| Build and test execution for this candidate | NOT RUN; awaiting primary and independent-review approval |
| External package references | NONE |
| Game or runtime equivalence | NOT CLAIMED / UNKNOWN |
| Publication | DISABLED |

## Scope and evidence

The reconstructed static source view identifies `Calculation.CalculateFormula` as MethodDef `0x06018873` on TypeDef `0x02002EB2`, under managed assembly authority SHA-256 `e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db` and MVID `2d6d37cb-76e5-4075-8220-3eeca41471e3`. Its final steps read the old seed, call the seed provider, store its result, read upper and lower fluctuation values, form a float factor, apply the extra rate and factor before `Math.Ceiling`, and negate the rounded result only for its Heal branch. The source view is reconstructed semantic source, not original developer source. The original assembly and metadata are not distributed with this example, and this page does not independently reproduce them. The game version remains UNKNOWN.

An earlier accepted damage-formula result remains a historical PASS for 9 unchanged bodies, 8 groups, and 39 assertions; this example does not change or upgrade it. A separate historical transformed-dispatch run recorded PASS for 8 groups and 50 assertions against a transformed `CalculateFormula` fixture and 11 unchanged dependency bodies. Because the dispatcher fixture replaced the missing-key logging block, compilation of the original unchanged dispatcher and its native logging semantics remain NOT_ESTABLISHED. That run's fixture and execution receipt are not distributed here or independently reproduced. Neither historical result verifies this model or live game behavior.

## Operation

`DamageFinalization.FinalizeAmount` accepts all values directly. The callbacks are controlled adapters; `extraRate` and `negateAfterCeiling` are scalars. This API does not model property getters, their original types, calculation-type dispatch, an RNG algorithm, or the original provider's reason argument.

The operation follows this order:

1. Capture `seed` as `OldSeed`.
2. Call `advanceSeed(OldSeed)` once, then store its result through `ref seed`.
3. Call `readUpper()` once, then `readLower()` once.
4. Compute `OldSeed % 10000` using C# signed remainder.
5. Compute `(Upper - Lower) * Remainder + Lower` as unchecked 32-bit integer arithmetic, cast that result to `float`, then multiply by `0.0001f` as `float`.
6. Promote the `float` extra rate to `double`, multiply `Amount` by it, then multiply that result by the promoted float factor. The two double multiplications stay left-to-right.
7. Apply `Math.Ceiling`. If requested by the adapter flag, compute `0.0 - Rounded` afterward.

There is no validation, finite-value sanitization, clamping, retry, or rollback. A seed-provider exception occurs before the model stores its return value. An upper- or lower-bound exception occurs after the new seed is stored. Other callback side effects are not undone.

## Trace

`FinalizationTrace` is a readonly value containing only these captured scalars; it retains no callback. Its generated `ToString()` formats the captured fields and does not rerun callbacks.

| Field | Type | Stage |
| --- | --- | --- |
| `Amount` | `double` | Input amount |
| `OldSeed`, `NewSeed` | `int` | Seed before and after the provider callback |
| `Upper`, `Lower` | `int` | Scalar bounds returned by the callbacks |
| `Remainder`, `Weighted` | `int` | Signed remainder and unchecked weighted integer |
| `Fluctuation`, `ExtraRate` | `float` | Float factor and input extra rate |
| `PromotedExtraRate` | `double` | Float extra rate promoted to double |
| `AfterExtraRate` | `double` | Amount multiplied by the promoted extra rate |
| `BeforeCeiling` | `double` | Left-to-right product before rounding |
| `Rounded`, `Result` | `double` | Ceiling result and optional post-ceiling sign result |

Representative bits for `Amount=10000.0001`, `ExtraRate=1f`, `Upper=Lower=1`, and `OldSeed=2` are: `Fluctuation=0x38D1B717`, `BeforeCeiling=0x3FEFFFFFF7CE63B6`, and `Result=0x3FF0000000000000`. A double `0.0001` factor would cross the ceiling boundary and produce 2 instead of 1.

For the left-to-right discriminator, `ExtraRate=0.1f` has bits `0x3DCCCCCD`; its promoted double has bits `0x3FB99999A0000000`. With `Amount` bits `0x401AAAAAA4000003` and a `1.5f` factor, `AfterExtraRate` is `0x3FE5555555555556` and `BeforeCeiling` is exactly `1.0` (`0x3FF0000000000000`). Regrouping the multiplications crosses the ceiling boundary.

## API example

```csharp
using DamageReference;

int seed = 2;
FinalizationTrace trace = DamageFinalization.FinalizeAmount(
    10000.0001,
    ref seed,
    oldSeed => 0,
    () => 1,
    () => 1,
    1f,
    false);

// trace.BeforeCeiling is just below 1; trace.Result is 1 after ceiling.
```

## Boundaries

This reference does not calculate Hurt or Heal amounts, resolve formulas, access configuration, draw random values, roll critical hits, or represent UI, server authority, HP application, or game state. The boolean sign adapter is not a recovered enum or dispatch rule. Getter evaluation order and original property types are not claimed. The tests use authored inputs and independent expected bits; passing them would validate only this model.

## Build and run

Use an installed .NET 8 SDK, or a newer SDK with the `net8.0` targeting pack and runtime already installed. The prepared candidate's controlled verification profile is SDK 9.0.312, reference pack 8.0.25, and runtime 8.0.11; it has not yet been run. The project has no package references, uses the project directory as its restore source, disables NuGet audit and transitive framework-reference downloads, and does not create an apphost.

```powershell
dotnet build DamageReference.csproj --nologo
dotnet run --project DamageReference.csproj --no-build --no-restore
```

The runner prints one line per named case and a final assertion count. A local MSBuild or NuGet configuration outside this project can still affect a machine's build environment.

## Files

- [DamageReference.csproj](DamageReference.csproj) - standalone BCL-only executable project.
- [DamageReference.cs](DamageReference.cs) - scalar finalization operation and immutable trace.
- [DamageReferenceTests.cs](DamageReferenceTests.cs) - authored numeric test runner.

Publication remains disabled. These files are provisional review-surface content, not a rights-cleared release.
