# Performance Timer Subsystem

Status: `RECONSTRUCTED / SELECTED METHODS`. This note describes two partial Timer imports, not original source, full-class recovery, execution, performance measurement, or binary equivalence. Runtime and publication dispositions in the underlying records are historical, not release authorization.

## Navigation and import closure

The subsystem path is:

1. Performance Timer subsystem.
2. Type `Timer` (`0x02000BEC`) and type `TimerSystemInstance` (`0x02000BED`), both in the observed global namespace.
3. Six selected ordinary methods:
   - `Timer.Do` `0x06003267`
   - `Timer.Clear` `0x06003269`
   - `TimerSystemInstance.Next` `0x06003271`
   - `TimerSystemInstance.DoAdd` `0x0600327E`
   - `TimerSystemInstance.Tick` `0x0600326B`
   - `TimerSystemInstance.DoRemove` `0x0600327F`
4. Exact imported source bytes:
   - [Timer.cs](../src/managed/cs-e3fc67839851/timer/Timer.cs), 2779 bytes, SHA-256 `c5d437c8f4c80de3a163fbb80fc6c4968b3467cb280d8bf4c1f9d310e09ecbfa`
   - [TimerSystemInstance.cs](../src/managed/cs-e3fc67839851/timer/TimerSystemInstance.cs), 4931 bytes, SHA-256 `2607a87cd72841399c0f6159ea08a9bd2587317c57e7b92327dee1f819b12201`
5. Typed records:
   - `metadata/performance-source.v1.json`, 17385 bytes, SHA-256 `bd8b2ef69be52e2c4f18fb9080e049d1b2e9da6bc377777b43e3c984cc6aa040`
   - records `PTIMER` and `PTIMER_INSTANCE`
6. Versioned import ledger, historical Timer checkpoint:
   - `metadata/imports.v3.json`, 16790 bytes, SHA-256 `22898a66f260d88b6f6295149fbf23fb47be73ac5c91e13418f615248f5421c9`
   - seven files and eighteen qualified methods after combining the five historical imports and the two Timer records. This v3 count is historical and is not the active v5 count.
    - The v5 receipt is also historical: 13 files and 102 qualified identities. Current active curated authority is the append-only curation index over that baseline; see the [source index](SOURCE_INDEX.md). Neither historical count describes the full publication selection.

The frozen `metadata/migration.json` is not modified by this supplement. Source-path fields in the typed record are provenance labels, not links to bundled inputs.

The following accepted historical evidence inputs are not included in this repository and were not independently reproduced for this editorial note:

- PERF03 script-runtime bridge report, 20118 bytes, SHA-256 `f3d785ffe8a024fb4a09f47ad23e9a3fa469d9d3f97f27e6a2e438b8cf069fde`; relevant ranges are lines 31-108 and 132-159.
- PERF03 direct-CIL evidence, 32004 bytes, SHA-256 `cd24738a2e66e11f13aaf0c3c0fca7d4323ea6f6821c8a27aba276a2dbc33127`; sections are `Next` 1-35, `DoAdd` 36-166, `Tick` 167-300, `DoRemove` 301-333, `Do` 334-465, and `Clear` 466-488.
- PERF03 evidence record, 29036 bytes, SHA-256 `3a5b97eaace2880d319e1e3941f333901654c624bacb31df03c30982676110ad`; selected-method identities, type identities, projection hashes, review limits, and unknowns.

No fresh binary extraction, decompilation, or native inspection is performed by this note.

## Selected observed behavior

`Timer.Do` reads the scheduled `Next` value, computes the callback delta as `(float)(Next - Now)`, observes the `Stat` field without using it, then advances `Now`, advances `Next` by `Interval / Dilation`, and increments `Count` before invoking the callback. It clears `Handle` before `Action.Invoke(delta)`. The observed exception region filters `System.Exception`; if `ErrorWithStack` returns, the callback exception is consumed. A logger exception itself is not shown to be caught. After the handler, `Loop == 0` returns true; otherwise the result is `Count < Loop`. The exact deadline/count-before-callback order is preserved in the imported bytes.

`Timer.Clear` nulls exactly `Action`, `Handle`, and `Stat`.

`TimerSystemInstance.Next` delegates with the observed numeric arguments `interval 1f`, `loop 1`, and `dilation 1f`. Their clock units and any next-frame meaning are not established.

`TimerSystemInstance.DoAdd` rejects a null action with the observed log path and returns null. Otherwise it resolves a method-associated `Stat` through `StatWeakMap` and `Stat.CreateWithStack("TimerAction", 2, 5)` when needed, creates a `TimerHandle`, creates a `Timer`, assigns `Now` and the initial `Next`, stores the timer by handle id in `Timers`, pushes it to `PriorityQueue<Timer>`, and registers finalization cleanup. The store, push, and register order is preserved.

`TimerSystemInstance.Tick` advances `Now` first. It checks queue emptiness before reading `Top`, breaks on a null or future top item, pops a due item before the state check, skips non-normal states, invokes `Timer.Do` for normal items, requeues when `Do` returns true, and calls `DoRemove` otherwise. After the due loop it drains the pending-removal queue and calls the existing `Remove(handle)` path. The imported method does not establish tick cadence or a guaranteed next-frame interpretation.

`TimerSystemInstance.DoRemove` unregisters from `Registry`, marks state `Trash`, calls `Clear`, removes the dictionary entry, and returns true.

## Unknowns and source quality

The constructors, field initializers, `TimerSystemInstance.Stat` field, unselected members, helpers, comparer behavior, `PriorityQueue` tie behavior, `Registry` and `TimerHandle` lifetimes, `StatWeakMap` behavior, callback-thread ownership, native clock, tick cadence, continuation placement, physical allocation, and runtime effects remain `UNKNOWN`. The logging tuple-array projection in the readable source is not evidence of the game's physical allocation. No native cadence or clock-unit claim is made. The later [lifecycle experiment](DAMAGE_SNAPSHOT_FLOW.md#timer-lifecycle-experiment) combines selected removal and queue source paths under explicit model assumptions; it is not an observed game bug.

Source quality dimensions:

- `EXACT_BYTES`: imported copies match their research source bytes.
- `DIRECT_CIL_FULL`: six selected ordinary bodies have accepted direct-CIL evidence.
- `PARTIAL_PACKAGING`: constructors and unselected members are omitted; this is not full-class recovery.
- `READABILITY`: `ACCEPTABLE_AS_IMPORTED`; the source is source-like and noncanonical.
- `EDITORIAL_PROJECTION`: none applied; no source bytes were changed.
- `EXECUTABLE_CHANGES`: none; no stubs, mocks, or dependency substitutions were added.
- `BUILD`: `NOT_BUILDABLE_WITH_CURRENT_CONTEXT`; C# compilation and C# tests were not run.

A .NET SDK 9.0.312 was reported present by a prior read-only command. It was not used to build these fragments. The Rust validator checks metadata and byte correspondence only; it is not a C# build or runtime proof.

## Historical performance source quality inventory

This is a compact historical index, not a fresh deep review of all thirteen areas and not an active-import list. It uses the accepted private release inventory as historical index material plus the current Timer inspection, the earlier primary TaskGraph inspection, the full GameModePromise source read, and the partial TaskSystem inspection. Unimported families remain historical inventory, not active implementations. No whole-repository or fresh-binary claim is made.

| Area | Indexed state | Boundary |
| --- | --- | --- |
| TaskGraph | Selected direct-CIL and reconstructed graph/score views | Helpers, comparer internals, allocation, and measured cost unresolved. |
| TaskSystem | Partial selected direct CIL plus source-only collaborators | Continuations, workers, cancellation, and dropped promises unresolved. |
| Timer | Six direct-CIL bodies in two partial files | Constructors, registry lifetime, cadence, and units unresolved. |
| Time.Tick | Selected direct CIL and reconstructed time view | Provider cost, units, native origin, and frequency unresolved. |
| Core.Tick and TickSystem | Selected direct CIL and metadata context | Wrapper/native delivery, thread, phase, and cadence unresolved. |
| Entity/component dispatch | Selected direct CIL and reconstructed fragments | Override population, mutation, callback faults, and collection cost unresolved. |
| Range workload | Selected direct CIL | Provider behavior, mutation, duration, and invocation rate unresolved. |
| Pending-event drain and lifecycle | Mixed retained root and source-only helpers | Opaque helpers, aliasing, reentry, and delivery unresolved. |
| Managed receiver and Event dispatch | Five selected Event definitions with stronger later evidence | Historical source-only view is retained; no duplicate deletion. |
| JavaScript ordinary dispatch | Shipped JS plus readable reconstructed projection | VM/JIT/GC, cadence, and managed linkage unresolved. |
| Puerts interface boundary | Observed JS call shapes and metadata | Native marshalling, bootstrap, ownership, and thread unresolved. |
| Loading and visibility | Mixed selected direct CIL and source context; GameModePromise getter current | Provider, overlap, cancellation, settlement, and delivery unresolved. The historical getter source remains unchanged. |
| Native feasibility | No relevant implementation established | `UNKNOWN` / `NON_PLAIN_AT_REST`; no native reconstruction claim. |

Evidence orthogonality is preserved. Old source-only Event representations remain historical; stronger later Event method evidence does not delete them. The current GameModePromise getter is `DIRECT_CIL_FULL`, while its historical source header and bytes remain unchanged. The Timer records do not overlap the five historical imports by qualified method key.

## Historical prioritized evidence-qualified backlog

1. Task scheduling: historical TaskGraph and TaskSystem method/status reconciliation, without runtime or FPS claims. TaskSystem execution is now imported separately under SOURCE03; this is not a TaskGraph runtime join and does not make the unimported families active.
2. Loading and visibility: GameModePromise and SubLevel provenance, overlap, cancellation, and settlement limits.
3. Time and Core tick: clock units, cadence, wrapper delivery, and native boundary questions.
4. Entity, Range, and pending events: callback/provider behavior, mutation, lifetime, and failure paths.
5. JS and Puerts bindings: marshalling, bootstrap, ownership, and thread questions.
6. Native feasibility: retain the read-only `UNKNOWN` boundary; do not propose native execution or reconstruction beyond authorized static evidence.

