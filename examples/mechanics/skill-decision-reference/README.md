# Skill decision reference model

This directory is an independently authored C# reference model of selected skill
interruption, cooldown-availability, and charge-restoration decisions. It is not
reconstructed or decompiled game source. It is not a port, extraction, game
simulator, or claim of CIL, binary, runtime, server, or visual parity. No
original recovered method body is copied. Names and numeric values are
synthetic. No game data is used or required. The model is self-contained: it
contains no game dependency, package reference, networking, native bridge, clock,
filesystem API, process API, timer, or server state.

## Status

| Item | State |
| --- | --- |
| Model source and tests | IMPLEMENTED |
| Build | PASS (0 warnings, 0 errors) |
| Test execution | PASS, 20 of 20 |
| CIL, binary, game, server, or runtime parity | NOT CLAIMED / UNKNOWN |
| Private input needed to build or run | NONE |
| Publication | DISABLED |

Actual result: the four byte-identical staged public files built with 0 warnings
and 0 errors. The test runner passed 20 of 20 cases with
`TOTAL: passed=20 failed=0`. The run used SDK 9.0.312, runtime 8.0.11, and the
already-installed Microsoft.NETCore.App.Ref 8.0.25 targeting pack. The plain
public recipe was executed from a controlled private staging copy, not from this
directory or a clean public clone. This verification used a controlled
environment; it is not an OS sandbox and does not claim that no environment
variables were present.

## Model

`Skill` is a sealed reference type with immutable integer `Id`, `Group`, and
`Priority` values. It has no overridden equality, operators, or record identity.

`SkillDecisions` owns active group lists, `EasyInterrupt` (default `false`), and
`ReadyEnd` (default `true`). Two optional callbacks supply debug mode and
cooldown availability. A null cooldown callback means the component is missing
and its answer is `false`.

`SkillDecisions.Check(incoming, output, reasons)` implements these rules:

- Null `output` or `reasons` arguments become local throwaway lists. Provided
  lists are mutated in place and their references are not reassigned.
- Group 1 uses only the first list entry. A missing, null, or empty list permits
  the call and changes neither list. There is no active-flag filter, sorting, or
  cooldown read in this branch.
- With a current group-1 skill, the model reads higher priority, equal priority
  plus easy interrupt, ready-end, and debug mode before the logical OR. Debug
  therefore runs even when priority or ready-end already accepts.
- Acceptance appends the current skill, preserves earlier entries, and does not
  de-duplicate. Rejection appends current priority, incoming priority, easy
  interrupt, and ready-end, then clears the whole `output` list and returns
  `false`. Reasons are never cleared.
- Other groups are enumerated in order. Cooldown is read before reference
  identity. The first cooldown hit appends an authored reason containing the
  skill id, clears the whole `output` list, and returns `false` immediately.
  Only the same object reference is appended; duplicate references append twice.
- There is no catch or rollback. Callback or pending-pop exceptions escape with
  the state accumulated before them.

`CooldownAvailability` wraps a nullable map from cooldown id to `ChargeState`.
`ChargeCompletion.Complete` restores an initial charge, may grant catch-up
charges, and consumes caller-supplied pending id and duration pairs. Its
`IPendingCooldowns` port has two counts and two pop methods. Durations are used
exactly as supplied, including zero, negative, and NaN. Scheduling happens after
`notify`, and no rollback is performed.

## Boundaries

The source-supported cooldown boundary is that a null map or null group fails on
use; this model represents that failure as `NullReferenceException`. The
exception messages are authored here, and the exact binary exception type,
message, and stack contract is `UNKNOWN`. The only added adapter guards are null
checks on required arguments and ports (`incoming`, `state`, `pending`, `notify`,
`schedule`) and the finite, non-negative `elapsed` check; no duration validation
is added.

Three further boundaries apply:

- The model has no activation flags, target selection, input, tags, animation
  windows, equality override, ID-based selection, or de-duplication.
- The model has no timer state, finish stamps, clock reads, `StartCd`,
  `OnDelayFinish`, combat logging, event dispatch, server initialization,
  scheduler, or concurrency.
- The test double is a controlled input assumption. It is not evidence that an
  original queue was FIFO or that callback delivery matched the binary.

## API example

```csharp
using System.Collections.Generic;
using SkillDecisionReference;

var current = new Skill(id: 1, group: 1, priority: 10);
var incoming = new Skill(id: 2, group: 1, priority: 20);
var decisions = new SkillDecisions { ReadyEnd = false, ReadDebug = () => false };
decisions.ActiveGroups[1] = new List<Skill> { current };

var interrupted = new List<Skill>();
var reasons = new List<string>();
bool allowed = decisions.Check(incoming, interrupted, reasons);

decisions.ReadCooldown = new CooldownAvailability(
    new Dictionary<long, ChargeState?> { [7] = new ChargeState(0, 2) }).IsUnavailable;
```

## Evidence identity

The anchors below identify a selected static surface for navigation. Their
supporting source bytes are not part of this example and are not independently
reproduced by the tests.

| Surface | TypeDef | MethodDefs |
| --- | --- | --- |
| Base skill current and decision | 0x02003118 | 0x06019FC3, 0x0601A017, 0x0601A018 |
| Base skill missing-component cooldown | 0x02003118 | 0x0601A020 |
| Character cooldown | 0x02003122 | 0x0601A08E |
| Group cooldown context | 0x02005F43 | 0x0603D3FF |
| Group cooldown restore and pair | 0x02005F43 | 0x0603D403, 0x0603D404 |

Assembly SHA-256
`e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, MVID
`2d6d37cb-76e5-4075-8220-3eeca41471e3`. The game version is `UNKNOWN`. The
tokens do not establish that this model matches the binary.

## Checkpoints

Three checkpoints must not be merged:

- A prior historical skill-interruption checkpoint reports four exact
  base-skill slices, eight groups, and 127 assertions under controlled
  dependency doubles. That is a controlled historical result only, not a current
  run, whole-class coverage, or game-runtime evidence.
- The cooldown and charge-lifecycle material used here is static-only. It
  includes no runtime observation, timer, scheduler, or server state.
- The tests in this directory are authored model tests only. They do not execute
  recovered bodies, compare CIL, or reproduce the historical assertions.

## Build and run

Prerequisites: the .NET 8 SDK, or a newer SDK with the `net8.0` targeting pack
and runtime already installed. No package reference, remote feed, network step,
game install, private corpus, or environment variable is required by the
project.

The project targets `net8.0`, has no package reference, disables NuGet audit,
uses no apphost, and disables transitive framework reference downloads. It sets
`RestoreSources` to this directory. A machine-level MSBuild extension that adds
another restore source is outside this project's control.

The plain public recipe is:

```powershell
dotnet build SkillDecisionReference.csproj --nologo
dotnet run --project SkillDecisionReference.csproj --no-build --no-restore
```

The runner prints one `PASS` or `FAIL` line per named case, then a `TOTAL` line,
and returns zero only when all 20 cases pass. The actual result was 20 of 20
passing; see the status above. The commands were executed from a byte-identical
private staging copy with the installed SDK 9.0.312, runtime 8.0.11, and
targeting pack 8.0.25, not from this directory or a clean public clone. The
controlled stage is not an OS sandbox and makes no zero-environment-variable
claim for the verification.

## Files

- [SkillDecisionReference.csproj](SkillDecisionReference.csproj) - `net8.0` executable project, no package references.
- [SkillDecisionReference.cs](SkillDecisionReference.cs) - independent model.
- [SkillDecisionTests.cs](SkillDecisionTests.cs) - 20-case assertion runner.
- [README.md](README.md) - scope and evidence statement.

Publication remains disabled. These four non-ignored files are provisional
review-surface entries, not a rights-cleared release, and they do not publish
source bytes or private execution results.
