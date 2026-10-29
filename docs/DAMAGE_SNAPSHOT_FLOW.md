# Damage snapshot and response flow

This page follows the selected `BaseDamageComponent` source around snapshot
creation and the request/response boundary. It is a branch-aware account of
separate source views, not a complete damage pipeline. The five additional
method spans are an exact, noncompilable `SOURCE_ONLY`
[excerpt](../qualified-research/damage-snapshots/BaseDamageComponent.SourceOnly.cs);
the [evidence record](../qualified-research/damage-snapshots/evidence.json)
gives their identities and bounded original-CIL checks. They do not replace or
edit the selected [canonical source view](../src/managed/cs-e3fc67839851/damage/BaseDamageComponent.cs).

## Qualification labels

- `SOURCE_VERIFIED`: the included source shows the expression or ordering.
- `CIL_VERIFIED`: bounded method IL identities and selected call operands were
  checked; this does not establish whole-body source/CIL equivalence.
- `MODEL_TESTED`: the authored timer model's six checks ran; this is not a game
  oracle.
- `INFERRED`: a conditional composition of source-supported steps under stated
  assumptions, not an observed runtime event.
- `UNKNOWN`: the available source and bounded checks do not establish the link.

## Damage branches

### Entry, snapshot, and local toughness result

In the selected `ProcessDamage` view, the invincibility-tag branch returns `0`
when `ImmuneType == 0` before allocating snapshots. Other paths populate
requirement fields, call `AllocateSnapshot`, then `ProcessDamageOptimize`.
Afterward, `GetServerDamage` is called and a separate local value is computed
with `CalculateToughReduce`, `DamageModifier.ApplyEffects`, and
`ExecToughReduceRedirection`; that value is returned as toughness reduction,
not demonstrated HP damage (`BaseDamageComponent.cs`, lines 120-157).

`AllocateSnapshot` selects the attacker attribute holder with
`attacker.FollowComponent?.GetAttributeHolder() ?? attacker.Entity`. It selects
the target holder with
`FollowComponent?.GetAttributeHolderExceptVisionSummon() ?? base.Entity`, then
uses `AttributeComponent.TakeSnapshot()` as the target snapshot fallback. Both
snapshots are evaluated before the payload initializer calls
`ReplaceAttacker(attacker)`. That replacement occurs only when
`VisionControlHandle != null` and the summon type is
`ConcomitantPhantomRole`; it then looks up the summoner's damage component,
which may be null. Otherwise the original attacker is retained. Attribute
holder selection and payload attacker selection are distinct decisions and
may differ; the helper implementations and their invariants are `UNKNOWN`.

The same method builds a filtered transfer list: invalid targets are skipped;
dead targets are skipped when a death component reports death; both damage and
attribute components are required before adding an entry. In contrast,
`HasDamageTransfer` is assigned from the original set's `Count > 0`, not from
the filtered list. It therefore does not guarantee that the resulting list is
nonempty.

`ProcessDamageOptimize` calls `ProcessWhenCounter`, `PreCriticalModify`, then
`PostCriticalModify`. The counter method only triggers attacker/victim buff
events when `IsCounterAttack` is true. This ordering is `SOURCE_VERIFIED`; the
selected call sites are also within the bounded `CIL_VERIFIED` checks. No full
critical calculation is reconstructed here.

### Request and response branches

`GetServerDamage` starts from the original `inputParam` and constructs a
`DamageExecuteRequest` using `inputParam.Attacker`. The selected code calls
`CombatNet.Call`; it does not show that the local snapshot payload is sent in
that request. The local toughness calculation and the response-sourced damage
are therefore separate visible paths, not a proven formula-to-server chain.

The response delegate has two visible branches:

1. `Invincible` logs when applicable and bypasses
   `GetServerDamageCallBack`.
2. Otherwise it copies `Damage = -response.Damage` and `ChangeLife` into a
   `DamageResult` (among other response fields). Only `ErrorCode.Success` calls
   the callback.

The callback then requires a valid target entity and a non-null attacker with a
valid entity. It calls `ExecDamageEvent`, then `ExecDamageEffect`, then optional
`part.OnDamage(..., deductLife: false)`. The selected `ExecDamageEffect` span
calls energy and post-damage extra-effect processing; their transitive effects
are not established. Copying `ChangeLife` and passing `deductLife: false` do
not establish that no other path changes HP.

The selected original CIL checks include `TakeSnapshot`'s constructor and two
`Array.Copy` call sites. They verify those calls, not the complete snapshot
semantics. The ten checked methods and 24 selected call sites are listed in the
[evidence record](../qualified-research/damage-snapshots/evidence.json).

### Open links

These materials do not establish an attribute-update-to-snapshot trigger, local
formula-to-server-damage integration, the actual HP writer or replication
consumer, event subscriber registration, server-authoritative acceptance,
TaskGraph integration, or native/C# ownership or migration. The game version is
`UNKNOWN`; the original assembly is not distributed. No unbranched
attribute-to-formula-to-HP pipeline is claimed. The excerpt remains
`SOURCE_ONLY`, body equivalence is `NOT_ESTABLISHED`, and rights review is
required.

## Timer lifecycle experiment

The related timer model is independently authored and separate from the damage
path. In the included source, `Timer.Do` advances its deadline and count and
clears its handle before invoking the callback. `TimerSystemInstance.Tick`
pops a due timer, checks `Normal`, calls `Do`, and pushes a repeating timer
without a visible post-callback state check. `DoRemove` unregisters, marks the
timer `Trash`, clears it, and removes it from the timer dictionary; it does not
remove it from the heap. `PriorityQueue.Push` appends without a timer-state
check. Pending removals are drained after the due-timer loop.

Under the model's assumptions, if a repeating timer callback reaches
`DoRemove` on that same object, `Do` can still return true and `Tick` can push
the now-`Trash` object. The next due-time pop rejects it by state. This is an
`INFERRED` conditional scenario, not proof of game reachability, a game bug, a
leak, or a performance issue. The omitted public `Remove`/`Get` lookup is not
implemented. Assumptions are one owner/thread and timer, an initially `Normal`
timer, positive unit interval and dilation, nonthrowing scenario callbacks and
registry/logging stand-ins, and authored heap/dictionary stand-ins.

Run the standalone model from any working directory by passing its checkout
path to Python:

```text
python -B <checkout>/examples/mechanics/timer-lifecycle/model.py
```

It checks source hashes and ordered fragments in the included `Timer.cs`,
`TimerSystemInstance.cs`, and `PriorityQueue.cs`, then
checks six scenarios: one-shot completion, repeat catch-up, self-removal and
later trash pop, a counterfactual post-callback guard, pending removal after the
due loop, and removal before the due loop. The counterfactual is a negative
control. The script writes no files and rejects execution without `-B`; a
passing model is not game-runtime equivalence.
