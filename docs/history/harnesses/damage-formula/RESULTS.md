# Damage formula execution result

Editorial successor to historical path
`tests/managed-recovered/damage-formula/RESULTS.md`; not a byte-identical
replacement. The saved predecessor SHA-256 is
`47b6f5ab2932cc0cdcdf701f6d29d666cd72321fa53ab4b859f2b10683f8bd19`. This is a
historical result summary, not a new receipt or execution. The private manifest,
full expanded argument vector, complete run layout, and execution receipts are not
distributed.

## Historical invocation and working directories

The recorded command ran from the **workspace root**, meaning the directory that
contained `github_src`, not from inside `github_src`. The command and hash below
are historical values for one prepared workspace; the current public checkout is
not a replayable copy of that workspace.

```powershell
cd <workspace root>
$env:PYTHONDONTWRITEBYTECODE=1
python -B github_src/tests/managed-recovered/damage-formula/run_damage_formula.py --execute --manifest manifest.preflight04.json --expected-manifest-sha da641f0513a87a3ddc205519e0e89e4a9714897c1776eaf76e1db02d796a947b
```

The recorded Python runner exited 0; external wall time was 1.83 seconds. It
verified the approved manifest hash and its pre-compiler gates before launching
the child process. The harness process working directory was
`<scratch>\run`. The recorded `execution-observations.json` called its cwd
`repo root`; read that as the workspace root above, not as `github_src`.
That receipt was not edited.

The compiler was invoked directly, without MSBuild or NuGet. The historical
manifest argv began `<dotnet> <csc.dll>`, not `dotnet exec <csc.dll>`. The
predecessor README's illustrated `exec` verb was inaccurate for the compiler
process. The recorded argument shape was:

```text
<dotnet> <csc.dll> /noconfig /nostdlib+ /target:exe /deterministic+ /checked- /optimize+ /debug- /nullable:enable /langversion:latest /out:<scratch>\build\FormulaRegression.dll /r:<each of 163 pinned reference DLLs> <scratch>\project\src\Calculation.RecoveredBodies.cs <scratch>\project\src\AttributeSnapshot.RecoveredGetter.cs <harness>\TestScaffolding.cs <harness>\HarnessTests.cs
```

The placeholder for the 163 reference arguments is descriptive, not the exact
expanded argv. The pinned reference pack was `Microsoft.NETCore.App.Ref` 8.0.25;
the harness runtime was .NET 8.0.11. `csc.dll` was version 4.14.0-3.26064.1.
The child environment was constructed from an empty map, with writable profile,
temporary, NuGet, and CLI paths redirected under scratch. The recorded runner
was not an OS sandbox.

## Recorded result

Status: **PASS** for one authorized execution of the frozen preflight-04
candidate. Historical output reports exit code 0, 8 of 8 groups passed, 39 of 39
assertions passed, 0 failed assertions, and successful post-run byte checks. The
nine saved bodies ran unchanged under authored hosts; this does not upgrade
their source or CIL classifications.

The compiler emitted no errors and exactly two nullable-analysis warnings in
copied bodies:

```text
Calculation.RecoveredBodies.cs(16,39): warning CS8602: Dereference of a possibly null reference.
Calculation.RecoveredBodies.cs(26,49): warning CS8604: Possible null reference argument for parameter 'attrSet' in 'double Calculation.GetElementDamageReduce(AttributeSnapshot attrSet, EElementType elementType)'.
```

The output assembly was 26,112 bytes, SHA-256
`ab9e99d671731498f0a5a2c1a8e7bd3dfdc4f8e6b16a860d43e566b81c422014`. Harness
stdout was 6,714 bytes and stderr was empty; compiler stderr was empty.
`GROUPS_PASSED=8`, `ASSERTIONS_TOTAL=39`, `ASSERTIONS_PASSED=39`,
`ASSERTIONS_FAILED=0`, and `RESULT=PASS`. Per-group assertion counts were D01=5,
D02=4, D03=2, D04=4, D05=6, D06=7, D07=5, and D08=6. The reported post-run
byte equality, authored hashes, and scoped preservation checks all succeeded.

## Synthetic vectors and outputs

Each snapshot had two zero-filled `float[143]` arrays except the listed slots.
Call defaults were element `Ice` (1), attack `Auto` (0), related attribute 7
(`Atk`), `attrRate=0`, `isCritical=false`, `augment=64`, `amplify=0`,
`gameplayRatio=1`, and `formula=0`. No game configuration, asset, Buff, or
runtime value was used. Expected IEEE-754 bits were the assertions.

| Case | Synthetic overrides | Recorded result | Bits |
|---|---|---:|---|
| D01a | attacker[7]=128f, `attrRate=10000`, `augment=0`, noncritical, neutral target | 127.99999676644802 | `0x405FFFFFF2700000` |
| D02a | target[10]=800f (`Def`), attacker level=0, ignore=0 | 32 | `0x4040000000000000` |
| D03a | attacker[15]=5000, attacker[22]=2500, attacker[17]=2500, target[22]=10000 decoy | 128 | `0x4060000000000000` |
| D04c | target[29]=10000 (Ice resistance), `augment=64` | 10.666666666666666 | `0x4025555555555555` |
| D05c | `augment=64`, `amplify=10000` | 127.99999838322401 | `0x405FFFFFF9380000` |
| D06a | attacker[7]=5000, target[7]=9999, `attrRate=10000`, `augment=0` | 4999.999873689376 | `0x40B387FFF7B8DC00` |
| D07a | `augment=0.25` | 0.25 | `0x3FD0000000000000` |
| D08f | attacker[7]=128, target[10]=400 (`Def`), `augment=64`, after two calls | 42.66666793823242 | `0x4045555560000000` |

Other recorded checks: D01c returned zero (`0x0`); D01d returned 64; D01e was
NaN. D02c returned 128. D02d, D05a, D05d, and D07b returned positive zero.
D05b and D05f returned 96. D06b/c/f/g returned 32. D07c was NaN and D07d was
positive infinity. D07e threw exactly `NullReferenceException`. D08a/e returned
32; D08b/c/d were boolean dependency and array-stability assertions. NaN was
asserted with `IsNaN`; the reported arithmetic NaN bits were
`0xFFF8000000000000`. The 143-float arrays remained unchanged.

## Qualification and errata

- `DIRECT_RECOVERED_EXECUTION` applies only to these nine saved bodies under
  authored hosts. Seven Calculation bodies retain `DIRECT_CIL_SUPPORTED`;
  `CalculateElementMatchUpRate` and `AttributeSnapshot.GetCurrentValue` remain
  saved-source/context views. No qualification was promoted.
- Hosts and adapters are `MODEL_ASSUMPTION`, including the 43-member enum subset,
  the `Max` alias, element namespace binding, 143-float arrays, and payload shape.
  Caller and registry wiring, `ForceCritical`/`IsCrit` production, request
  construction, and HP application are `INTEGRATION_NOT_TESTED`; live-game
  behavior remains `RUNTIME_UNKNOWN`.
- The predecessor README said compiled sources and output were under scratch.
  Only the nine fixture slices were there; `TestScaffolding.cs` and
  `HarnessTests.cs` were read from the harness directory. Containment refers to
  generated outputs under scratch.
- The predecessor README said no existing file was edited. Precisely, no file
  that existed at task start was edited; four new, unaccepted authored files in
  that historical workspace were corrected before freeze. Accepted source and
  receipt bytes were not modified.
- The recorded workspace was one-shot: its build, run, and execute receipts
  existed and the runner refused reuse. Existing-assembly replay was documented
  from `<scratch>\run` but not run as part of this result. No cleanup, deletion,
  or in-place rerun is supported.
- This was not a public clean-checkout reproduction. The private canonical
  inputs, preflight manifest, complete prepared run layout, pinned reference pack,
  and run receipts are not included. Selected fixture-source views are linked from
  the current test index. No game-runtime, binary-equivalence, caller,
  registry, seed, rounding, formula-dispatch, or HP-application claim follows.

See the [harness scope](README.md) and [test index](../../../../tests/README.md)
for repository navigation. This editorial successor did not compile or execute
the harness.
