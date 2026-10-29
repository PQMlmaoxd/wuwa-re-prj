# Event 101 reference model

**This is an independently authored synthetic C# reference model, not reconstructed or
decompiled game source.** It is not a port, extraction, reimplementation of original method
bodies, or a claim of semantic, runtime, binary, or visual parity. It is an executable
teaching model of *conditional mechanics around Event 101*. Names and values are synthetic;
no game entity, account, asset, or proprietary data is used or required.

The code contains no game dependency, NuGet package, networking, native bridge, HP state,
clock, filesystem API, or process API. Synthetic values are test inputs, never game outputs.

## Status

This table records the preparation checkpoint, not a current execution result.
Later historical model results are distinguished in the [test index](../../../tests/README.md).

| Item | Status |
| --- | --- |
| Ten test groups implemented | IMPLEMENTED |
| Build / compilation | **NOT RUN** |
| Test execution | **NOT RUN** (`MODEL_PREPARED_EXECUTION_BLOCKED`) |
| Runtime or game parity | NOT CLAIMED / UNKNOWN |

Building requires a compatible SDK, the .NET 8 targeting pack and runtime.
Normal `dotnet`/MSBuild restore and build operations can create, replace or delete
`bin/`, `obj/` and temporary files. This model has no game-runtime dependencies.

## What the model implements

- A fixed five-argument delegate:
  `(Entity attacker, Entity victim, Requirements requirements, DamageResult result, Position position)`.
- A strong-reference event registry keyed by **target object identity** and event id. Event
  101 is a public constant; synthetic other ids are accepted for reentry-scope tests.
- Per-`(target, id)` state with an ordinary bucket and conditional buckets keyed by
  **attacker object identity**. Conditional matching by attacker identity is an explicit model
  assumption replacing an unmodeled original condition-key comparer; there is no predicate or
  arbitrary ordering engine.
- Single-threaded dispatch: the matched conditional bucket runs before the ordinary bucket.
  Mutations during dispatch are deferred state-wide. Pending removals are skipped; additions
  become active only after dispatch. The emitting mask is cleared **before** pending ordinary
  removals/additions, then conditional removals/additions, are flushed.
- Reentry on the same target/id returns `false`. Different target/id registrations are
  independent and not globally blocked.
- Registration creates target state. Removing the final listener retains it. An existing
  target with no listeners for the emitted id completes with `true` (without creating a name
  entry); a missing target returns `false`. Neither success proves delivery.
- Handler exceptions go to an injected `Action<Exception>` handler-error sink (default:
  collect in memory and continue). The sink has no Boolean stop choice; a normally returning
  sink permits continuation, while a throwing sink escapes to the outer wrapper. Escaping
  dispatch failures go to a separate outer sink, then `Emit` returns `false`. There is
  deliberately no `finally` clearing the emitting bit: a failure before the pre-flush clear
  leaves the state emitting as a visible evidence boundary. If the outer sink itself throws,
  that exception escapes `Emit`.
- A synthetic routing wrapper with explicit Local and All target entities. The owner target is
  emitted first; after it returns, caller callbacks supply team-item existence, my-role, and
  formation-owner match. A team item emits/yields `Local` only when mine, then emits/yields
  `All`; a formation match independently emits/yields `Local` then `All`. Every broadcast uses
  the original victim payload. No de-duplication, no native bridge, `Emit` results ignored,
  null target yields, emits, and queries nothing.
- A consumer that converts `result.Damage` to `int` and copies the `DamageData` value snapshot
  before optional statistics. Conversion and Hurt statistics still run when combat is disabled;
  only combat-record creation is gated, before attacker/victim ids are read. The consumer
  re-reads nullable requirement flags and `SkillId`; maps attacker/target/damage data into
  synthetic records; runs formatter → `All` → filter hook → `View`; and emits a killed record
  when the captured killed flag was true. Heal statistics and the explicit synthetic heal sink
  both receive a negated amount; no heal-record constructor is modelled. Formatting is injected.
  The cited original formatting path reads a Life value for display; this model deliberately
  has no HP field, HP callback, HP mutation, or Life formatter.

### Deliberate model assumptions

- **Strong references, reference comparers, and stable single-threaded storage iteration** are
  model choices. The original system's weak-map behavior and listener ordering are not
  reproduced or claimed.
- Conditional keys model only attacker identity; the full original condition engine is out of
  scope.
- Conversion is restricted to finite `Single` values representable as `Int32`. This is a
  **model input-domain restriction**, not a game guard: NaN/overflow behavior is not
  established statically. The range check compares in `double` because `float` rounds
  `Int32.MaxValue` upward. Heal negation is an explicit unchecked `Int32` operation;
  `Int32.MinValue` wraps.
- `CurrentAttackerId`/`CurrentTargetId` restrict `View` only when positive and mismatched.
  Zero/negative current values disable that side's restriction. Record participant ids may be
  zero or negative. Filter values are ignored; only typed key presence participates.
- Explicitly omitted: once-handlers, weak-map behavior, generic delegate fallback, actual
  statistic instrumentation, native-bridge implementation, concurrency, HP mutation, runtime
  parity, and server authority.
- No statistics/logging/date/format parity and no actual heal-record semantics. Synthetic
  fault hooks are model-only test injection; they do not assert that the original code contains
  equivalent hooks or exception boundaries.
- **E7 is an HP/server-authority unknown frontier only**, not evidence for positive participant
  ID bounds.

## Stable evidence identities

These identities were quoted from the supplied historical evidence. The
supporting raw source and CIL bytes are not distributed here and were not
independently re-read for this model. The common `CSharpScript` assembly SHA-256
is `e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, with
MVID `2d6d37cb-76e5-4075-8220-3eeca41471e3`; game version remains unknown.

| Key | Model aspect | Stable identity and qualification |
| --- | --- | --- |
| E1 | Five-argument Event 101 registration and payload order | TypeDefs `0x02002E65` / `0x02002ED4`, MethodDefs `0x06018655` / `0x060189B0`; accepted canonical CIL. |
| E2 | Synthetic owner/team-item/formation routing | TypeDef `0x02002954`, MethodDefs `0x06015080` / `0x06015083`; generated TypeDef `0x02008C71`, MethodDefs `0x06049793` / `0x06049797`; `SOURCE_ONLY`. |
| E3 | Target-keyed event registry and target lookup | TypeDef `0x0200706E`, MethodDefs `0x06045B76`, `0x06045B7F`, `0x06045B80`, `0x06045B8E`, `0x06045B40`, `0x06045B5E`, `0x06045B58`; `SOURCE_ONLY`. |
| E4 | Emit/reentry and pending add/remove lifecycle | TypeDef `0x020000A3`, MethodDefs `0x06000434` / `0x06000435`; accepted direct-CIL evidence. EventArgs TypeDef `0x020000AC`, MethodDefs `0x06000460` / `0x06000461`; source context. |
| E5 | Consumer flow, record construction, and list-view filter keys | TypeDef `0x02002ED4`, MethodDefs `0x060189B4`, `0x060189EF`, `0x060189F4`; accepted canonical CIL. |
| E6 | Synthetic record field shapes | TypeDefs `0x02002ECB`, `0x02002ECA`, `0x02002ED0`, MethodDefs `0x0601899A`, `0x0601899B`, `0x06018993`, `0x06018994`, `0x060189A4`, `0x060189A5`; `SOURCE_ONLY`. |
| E7 | HP/server-authority unknown frontier only; not a positive participant-ID rule | TypeDef `0x0200305F`, MethodDef `0x06019710`; incoming `deductLife: false` is context only. |

The quoted identities are not a fresh extraction or a public substitute for the
supporting evidence bytes.

## Ten implemented test groups

| # | Group | Focus |
| --- | --- | --- |
| 1 | Payload and mapped Hurt record | Five-argument identity/equality; mapped fields including a synthetic `long` id above `Int32.MaxValue`; All/View same instance; disabled combat record suppression. |
| 2 | Registration and identity | Missing vs registered-but-empty; duplicate add/remove; same numeric id, different references. |
| 3 | Reentry scope | Same target/id false; independent target/id true with finite handler setup. |
| 4 | Deferred mutations | Conditional removes/adds/cancels ordinary listeners; next-emit effects; pending-add cancellation; re-add cancellation. |
| 5 | Shared mutation and snapshot | Conditional mutates shared objects before ordinary; statistics mutation; cached amount/data vs fresh flags/SkillId. |
| 6 | Failure boundaries | Formatter throw before `All`; filter throw after `All`; bus containment and later emission; conditional throw with ordinary continuation; inner sink throw, outer log, sticky emitting bit. |
| 7 | Conversion and routing | Positive/negative truncation; Hurt vs Heal sign; Other no-op; explicit NaN/infinity/out-of-range guard; disabled combat still converts/runs statistics but creates no records. |
| 8 | Filter model | Key presence regardless of value; missing key; positive Current attacker/target restriction and matches; zero/negative Current and record ids; partial `All` on hook failure. |
| 9 | Killed capture | Null/false vs true; Damage then Killed; capture before formatting; later requirement mutation does not cancel captured true; All/View identity; format/filter trace. |
| 10 | Synthetic routing | Actual owner/local/all bus broadcasts; exact shared payload references/positions; duplicates; handler changes later callbacks; absent owner listener; null target; ignored Emit false. |

All ten groups were implemented but not executed at the preparation checkpoint.
Assertions are BCL-only; `Console` is the only
I/O. The runner prints PASS/FAIL per group plus totals and returns nonzero on any failure.

## Files

- `Event101Model.cs`: model, consumer, routing.
- `Event101Tests.cs`: ten-group assertion runner.
- `Event101Reference.csproj`: `net8.0`, executable, nullable enabled, no package references.
- `README.md`: this scope and evidence statement.
