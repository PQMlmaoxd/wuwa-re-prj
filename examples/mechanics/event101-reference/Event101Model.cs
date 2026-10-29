// ORIGINAL REFERENCE MODEL -- synthetic, not reconstructed or decompiled game source.
// This is an independently authored teaching model of conditional Event 101 mechanics.
// It is not a port, extraction, or claim of semantic, runtime, or binary parity.
// Evidence anchors are cited in README.md; no original source bodies are copied.
// No game entity/account/proprietary data, HP state, networking, native bridge,
// filesystem API, process API, or clock is represented here.

using System;
using System.Collections.Generic;

namespace Event101Reference;

// E1: five-argument registration shape observed around Event 101. Synthetic identifiers only.
public sealed class Entity
{
    public Entity(int id)
    {
        Id = id;
    }

    public int Id { get; }
}

// Mutable by design: the model tests whether a consumer re-reads fields after earlier handlers.
public sealed class Requirements
{
    public int? SkillId { get; set; }
    public bool? Critical { get; set; }
    public bool? Immune { get; set; }
    public bool? Killed { get; set; }
}

public enum CalculationType
{
    Hurt = 0,
    Heal = 1,
    Other = 2
}

// Value type: copying this snapshot isolates a consumer from later mutations.
public readonly struct DamageData
{
    public DamageData(long id, CalculationType calculation)
    {
        Id = id;
        Calculation = calculation;
    }

    public long Id { get; }
    public CalculationType Calculation { get; }
}

public sealed class DamageResult
{
    public float Damage { get; set; }
    public DamageData DamageData { get; set; }
}

public readonly struct Position : IEquatable<Position>
{
    public Position(double x, double y, double z)
    {
        X = x;
        Y = y;
        Z = z;
    }

    public double X { get; }
    public double Y { get; }
    public double Z { get; }

    public bool Equals(Position other)
    {
        return X.Equals(other.X) && Y.Equals(other.Y) && Z.Equals(other.Z);
    }

    public override bool Equals(object? obj)
    {
        return obj is Position other && Equals(other);
    }

    public override int GetHashCode()
    {
        return HashCode.Combine(X, Y, Z);
    }

    public static bool operator ==(Position left, Position right)
    {
        return left.Equals(right);
    }

    public static bool operator !=(Position left, Position right)
    {
        return !left.Equals(right);
    }

    public override string ToString()
    {
        return $"({X}, {Y}, {Z})";
    }
}

// E1/E5: the fixed payload order is part of this synthetic contract.
public delegate void DamageEventHandler(
    Entity attacker,
    Entity victim,
    Requirements requirements,
    DamageResult result,
    Position position);

public static class EventIds
{
    public const int CharDamageAll = 101;
}

// E3: registry and dispatch model. Synthetic, not a reproduction of the original container.
//
// MODEL ASSUMPTIONS:
// - Target identity is the Entity object reference, not Entity.Id.
// - Conditional participant identity is also the attacker object reference. The original
//   condition-key comparer is not modelled; this is intentionally an identity substitute.
// - Registration holds strong references. This is not the original implementation's likely
//   weak-reference behavior.
// - Dispatch is single-threaded.
// - Enumeration order inside a bucket is an implementation detail of this model, not a
//   recovered original listener-order contract.
// - Conditional buckets run before the ordinary bucket. Tests rely only on that phase order
//   and use one handler per bucket when ordering matters.
// - A state is retained after its last listener is removed, so "registered, no listeners"
//   is distinguishable from "never registered".
public sealed class EventBus
{
    private sealed class ListenerBucket
    {
        public readonly List<DamageEventHandler> Active = new();
        public readonly List<DamageEventHandler> PendingAdd = new();
        public readonly List<DamageEventHandler> PendingRemove = new();
    }

    private sealed class EventState
    {
        public readonly ListenerBucket Ordinary = new();
        public readonly Dictionary<Entity, ListenerBucket> Conditional =
            new(ReferenceEqualityComparer.Instance);
        public bool Emitting;
    }

    private readonly Dictionary<Entity, Dictionary<int, EventState>> _states =
        new(ReferenceEqualityComparer.Instance);

    private readonly List<Exception> _handlerErrors = new();
    private readonly List<Exception> _dispatchErrors = new();
    private readonly Action<Exception> _handlerErrorSink;
    private readonly Action<Exception> _dispatchErrorSink;

    public EventBus(
        Action<Exception>? handlerErrorSink = null,
        Action<Exception>? dispatchErrorSink = null)
    {
        _handlerErrorSink = handlerErrorSink ?? DefaultHandlerErrorSink;
        _dispatchErrorSink = dispatchErrorSink ?? DefaultDispatchErrorSink;
    }

    public IReadOnlyList<Exception> HandlerErrors => _handlerErrors;
    public IReadOnlyList<Exception> DispatchErrors => _dispatchErrors;

    private void DefaultHandlerErrorSink(Exception exception)
    {
        _handlerErrors.Add(exception);
    }

    private void DefaultDispatchErrorSink(Exception exception)
    {
        _dispatchErrors.Add(exception);
    }

    public bool Add(Entity target, int eventId, DamageEventHandler handler)
    {
        return AddCore(target, eventId, null, handler);
    }

    public bool AddConditional(
        Entity target,
        int eventId,
        Entity attacker,
        DamageEventHandler handler)
    {
        if (attacker is null)
        {
            throw new ArgumentNullException(nameof(attacker));
        }

        return AddCore(target, eventId, attacker, handler);
    }

    public bool Remove(Entity target, int eventId, DamageEventHandler handler)
    {
        return RemoveCore(target, eventId, null, handler);
    }

    public bool RemoveConditional(
        Entity target,
        int eventId,
        Entity attacker,
        DamageEventHandler handler)
    {
        if (attacker is null)
        {
            throw new ArgumentNullException(nameof(attacker));
        }

        return RemoveCore(target, eventId, attacker, handler);
    }

    private bool AddCore(
        Entity target,
        int eventId,
        Entity? attacker,
        DamageEventHandler handler)
    {
        if (target is null)
        {
            throw new ArgumentNullException(nameof(target));
        }

        if (handler is null)
        {
            throw new ArgumentNullException(nameof(handler));
        }

        EventState state = GetOrCreateState(target, eventId);
        ListenerBucket bucket = GetOrCreateBucket(state, attacker);

        // E4: mutations are deferred for every bucket of this state while it is emitting.
        if (state.Emitting)
        {
            // Re-adding a handler that is already scheduled for removal cancels that removal.
            if (bucket.PendingRemove.Remove(handler))
            {
                return true;
            }

            if (bucket.Active.Contains(handler) || bucket.PendingAdd.Contains(handler))
            {
                return false;
            }

            bucket.PendingAdd.Add(handler);
            return true;
        }

        if (bucket.Active.Contains(handler))
        {
            return false;
        }

        bucket.Active.Add(handler);
        return true;
    }

    private bool RemoveCore(
        Entity target,
        int eventId,
        Entity? attacker,
        DamageEventHandler handler)
    {
        if (target is null)
        {
            throw new ArgumentNullException(nameof(target));
        }

        if (handler is null)
        {
            throw new ArgumentNullException(nameof(handler));
        }

        if (!_states.TryGetValue(target, out var byId) ||
            !byId.TryGetValue(eventId, out var state))
        {
            return false;
        }

        ListenerBucket? bucket = FindBucket(state, attacker);
        if (bucket is null)
        {
            return false;
        }

        if (state.Emitting)
        {
            // Removing a pending addition cancels it without ever becoming active.
            if (bucket.PendingAdd.Remove(handler))
            {
                return true;
            }

            if (bucket.PendingRemove.Contains(handler))
            {
                return false;
            }

            if (!bucket.Active.Contains(handler))
            {
                return false;
            }

            bucket.PendingRemove.Add(handler);
            return true;
        }

        return bucket.Active.Remove(handler);
    }

    // Null target has no registration to look up. This never creates state.
    public bool Emit(
        Entity target,
        int eventId,
        Entity? attacker,
        Entity victim,
        Requirements requirements,
        DamageResult result,
        Position position)
    {
        if (target is null)
        {
            return false;
        }

        if (!_states.TryGetValue(target, out var byId))
        {
            return false;
        }

        // E4: finding a target event object is not proof that this name has listeners.
        // An absent name completes successfully without invoking or registering anything.
        if (!byId.TryGetValue(eventId, out var state))
        {
            return true;
        }

        // Same target/id reentry is rejected; other target/id registrations are independent.
        if (state.Emitting)
        {
            return false;
        }

        state.Emitting = true;

        try
        {
            // Conditional matched bucket first, then ordinary. No ordering claim is made
            // between separate conditional attacker keys.
            if (attacker is not null &&
                state.Conditional.TryGetValue(attacker, out var conditional))
            {
                Dispatch(conditional, attacker, victim, requirements, result, position);
            }

            Dispatch(state.Ordinary, attacker, victim, requirements, result, position);

            // The mask is cleared before pending mutations are flushed. A failure before this
            // point leaves the state emitting; there is deliberately no finally.
            state.Emitting = false;

            // E4: ordinary flush before conditional flush; each bucket removes then adds.
            Flush(state.Ordinary);
            foreach (var bucket in state.Conditional.Values)
            {
                Flush(bucket);
            }

            return true;
        }
        catch (Exception exception)
        {
            // No rollback: failures before the pre-flush clear leave the bit set;
            // failures after that clear do not set it again.
            _dispatchErrorSink(exception);
            return false;
        }
    }

    private void Dispatch(
        ListenerBucket bucket,
        Entity? attacker,
        Entity victim,
        Requirements requirements,
        DamageResult result,
        Position position)
    {
        for (int i = 0; i < bucket.Active.Count; i++)
        {
            DamageEventHandler handler = bucket.Active[i];

            // A handler scheduled for removal during this dispatch is skipped.
            if (bucket.PendingRemove.Contains(handler))
            {
                continue;
            }

            try
            {
                handler(attacker!, victim, requirements, result, position);
            }
            catch (Exception exception)
            {
                // The sink has no Boolean stop choice. A normally returning sink permits
                // continuation; a throwing sink escapes to the outer emit wrapper.
                _handlerErrorSink(exception);
            }
        }
    }

    private static void Flush(ListenerBucket bucket)
    {
        foreach (DamageEventHandler handler in bucket.PendingRemove)
        {
            bucket.Active.Remove(handler);
        }

        bucket.PendingRemove.Clear();

        foreach (DamageEventHandler handler in bucket.PendingAdd)
        {
            if (!bucket.Active.Contains(handler))
            {
                bucket.Active.Add(handler);
            }
        }

        bucket.PendingAdd.Clear();
    }

    private EventState GetOrCreateState(Entity target, int eventId)
    {
        if (!_states.TryGetValue(target, out var byId))
        {
            byId = new Dictionary<int, EventState>();
            _states.Add(target, byId);
        }

        if (!byId.TryGetValue(eventId, out var state))
        {
            state = new EventState();
            byId.Add(eventId, state);
        }

        return state;
    }

    private static ListenerBucket GetOrCreateBucket(EventState state, Entity? attacker)
    {
        if (attacker is null)
        {
            return state.Ordinary;
        }

        if (!state.Conditional.TryGetValue(attacker, out var bucket))
        {
            bucket = new ListenerBucket();
            state.Conditional.Add(attacker, bucket);
        }

        return bucket;
    }

    private static ListenerBucket? FindBucket(EventState state, Entity? attacker)
    {
        if (attacker is null)
        {
            return state.Ordinary;
        }

        return state.Conditional.TryGetValue(attacker, out var bucket) ? bucket : null;
    }
}

// E6: synthetic record shapes. DamageRecord mirrors the fields needed by the consumer model;
// KilledRecord intentionally carries only attacker/target. Neither is an original constructor
// or serialization claim, and no presentation clock/date field is modelled.
public abstract class CombatRecord
{
    protected CombatRecord(int attackerId, int targetId)
    {
        AttackerId = attackerId;
        TargetId = targetId;
    }

    public int AttackerId { get; }
    public int TargetId { get; }
}

public sealed class DamageRecord : CombatRecord
{
    public DamageRecord(
        int attackerId,
        int targetId,
        long damageId,
        int damageValue,
        int skillId)
        : base(attackerId, targetId)
    {
        DamageId = damageId;
        DamageValue = damageValue;
        SkillId = skillId;
    }

    public long DamageId { get; }
    public int DamageValue { get; }
    public int SkillId { get; }
}

public sealed class KilledRecord : CombatRecord
{
    public KilledRecord(int attackerId, int targetId)
        : base(attackerId, targetId)
    {
    }
}

// Immutable diagnostic observation: lets tests inspect the freshly read flags without
// storing them on DamageRecord.
public readonly struct CombatSnapshot
{
    public CombatSnapshot(
        bool critical,
        bool immune,
        bool? killed,
        int damageValue,
        DamageData data)
    {
        Critical = critical;
        Immune = immune;
        Killed = killed;
        DamageValue = damageValue;
        Data = data;
    }

    public bool Critical { get; }
    public bool Immune { get; }
    public bool? Killed { get; }
    public int DamageValue { get; }
    public DamageData Data { get; }
}

// E5/R44: synthetic list-view filter keys used by this model only.
public static class DamageFilterKeys
{
    public const int Damage = 0;
    public const int Killed = 4;
}

// E5/E6: consumer model. E7 is an HP/server-authority unknown frontier, not a positive-ID rule.
//
// MODEL ASSUMPTIONS:
// - Conversion and the DamageData value snapshot happen before optional statistics, including
//   when CombatEnabled is false. CombatEnabled gates only combat-record construction.
// - Conversion is restricted to finite Single values representable as Int32. The original
//   NaN/overflow behavior is not established, so this guard is a model input-domain rule,
//   not a recovered game guard. Comparison is in double so float-rounded Int32.MaxValue
//   cannot slip through.
// - Requirements flags are read with GetValueOrDefault after statistics; SkillId uses the
//   nullable default 0.
// - Formatting, All insertion, filter hook, and View insertion are separate observable steps;
//   an exception leaves already-persisted partial state in place (no rollback).
// - CurrentAttackerId/CurrentTargetId restrict View only when positive; zero or negative
//   disables that side's restriction. Record ids need not be positive.
// - Filter values are ignored; typed key presence is the only modeled decision. Only the two
//   families implemented here are supported.
// - Heal statistics and the synthetic heal sink both receive an unchecked negated Int32;
//   Int32.MinValue wraps. No heal-record constructor is modelled.
public sealed class CombatConsumer
{
    public bool CombatEnabled { get; set; } = true;

    public List<CombatRecord> All { get; } = new();
    public List<CombatRecord> View { get; } = new();

    // Values are intentionally ignored; only ContainsKey participates in this model.
    public Dictionary<int, bool> Filter { get; } = new();

    public int CurrentAttackerId { get; set; }
    public int CurrentTargetId { get; set; }

    public Action<CombatRecord>? Formatter { get; set; }
    public Action<CombatRecord, int>? BeforeFilter { get; set; }
    public Action<Requirements, DamageResult, int>? HurtStatistics { get; set; }
    public Action<Requirements, DamageResult, int>? HealStatistics { get; set; }
    public Action<int>? HealConsumer { get; set; }
    public Action<CombatSnapshot>? HurtObservation { get; set; }

    public void OnCharDamageAll(
        Entity attacker,
        Entity victim,
        Requirements requirements,
        DamageResult result,
        Position position)
    {
        double raw = result.Damage;
        if (!double.IsFinite(raw) || raw < int.MinValue || raw > int.MaxValue)
        {
            throw new ArgumentOutOfRangeException(
                nameof(result),
                result.Damage,
                "Synthetic model input domain: Damage must be finite and representable as Int32.");
        }

        int amount = (int)result.Damage;
        DamageData cachedData = result.DamageData;

        // Branch on the copied calculation type, not on a later-mutated result field.
        switch (cachedData.Calculation)
        {
            case CalculationType.Hurt:
            {
                HurtStatistics?.Invoke(requirements, result, amount);

                bool critical = requirements.Critical.GetValueOrDefault();
                bool immune = requirements.Immune.GetValueOrDefault();
                bool? killed = requirements.Killed;
                int skillId = requirements.SkillId.GetValueOrDefault();

                HurtObservation?.Invoke(
                    new CombatSnapshot(critical, immune, killed, amount, cachedData));

                // Only record construction is gated; conversion, statistics, and the
                // diagnostic observation above still ran while combat was disabled.
                if (!CombatEnabled)
                {
                    break;
                }

                EmitRecord(
                    new DamageRecord(
                        attacker.Id,
                        victim.Id,
                        cachedData.Id,
                        amount,
                        skillId),
                    DamageFilterKeys.Damage);

                if (killed == true)
                {
                    EmitRecord(
                        new KilledRecord(attacker.Id, victim.Id),
                        DamageFilterKeys.Killed);
                }

                break;
            }

            case CalculationType.Heal:
            {
                // Explicit unchecked Int32 negation; Int32.MinValue wraps.
                int healedAmount = unchecked(-amount);
                HealStatistics?.Invoke(requirements, result, healedAmount);
                HealConsumer?.Invoke(healedAmount);
                break;
            }

            case CalculationType.Other:
            default:
            {
                break;
            }
        }
    }

    private void EmitRecord(CombatRecord record, int filterKey)
    {
        Formatter?.Invoke(record);
        All.Add(record);
        BeforeFilter?.Invoke(record, filterKey);

        // A positive current restriction must match; zero or negative disables that side.
        // Record participant ids themselves may be zero or negative.
        if (CurrentAttackerId > 0 && CurrentAttackerId != record.AttackerId)
        {
            return;
        }

        if (CurrentTargetId > 0 && CurrentTargetId != record.TargetId)
        {
            return;
        }

        if (!Filter.ContainsKey(filterKey))
        {
            return;
        }

        View.Add(record);
    }
}

// E2: synthetic routing labels. These are strings used by this model's tests, not game tokens.
public static class RouteTokens
{
    public const string Owner = "Owner";
    public const string Local = "Local";
    public const string All = "All";
}

// E2: synthetic routing wrapper.
//
// MODEL ASSUMPTIONS:
// - Owner emission happens first, before team/formation callbacks are queried. Callbacks are
//   evaluated after it returns, so an owner handler can change later fanout.
// - Every selected route target is actually emitted: original target, then conditional Local
//   and All targets. Every emit keeps the ORIGINAL victim payload; broadcast targets are keys.
// - The owner yield follows its emission. A team-item condition emits/yields Local only when
//   the caller reports it is my role, then emits/yields All. Formation independently emits/
//   yields Local then All. No de-duplication.
// - The Emit Boolean is ignored and there is no native bridge.
// - A null target yields nothing, emits nothing, and queries no condition callbacks.
public static class Event101Routing
{
    public static IEnumerable<string> Route(
        Entity attacker,
        Entity? target,
        Entity localTarget,
        Entity allTarget,
        EventBus bus,
        int eventId,
        Requirements requirements,
        DamageResult result,
        Position position,
        Func<bool>? teamItemExists = null,
        Func<bool>? isMyRole = null,
        Func<bool>? formationOwnerMatches = null)
    {
        if (target is null)
        {
            yield break;
        }

        bus.Emit(target, eventId, attacker, target, requirements, result, position);
        yield return RouteTokens.Owner;

        if (teamItemExists is not null && teamItemExists())
        {
            if (isMyRole is not null && isMyRole())
            {
                bus.Emit(localTarget, eventId, attacker, target, requirements, result, position);
                yield return RouteTokens.Local;
            }

            bus.Emit(allTarget, eventId, attacker, target, requirements, result, position);
            yield return RouteTokens.All;
        }

        if (formationOwnerMatches is not null && formationOwnerMatches())
        {
            bus.Emit(localTarget, eventId, attacker, target, requirements, result, position);
            yield return RouteTokens.Local;
            bus.Emit(allTarget, eventId, attacker, target, requirements, result, position);
            yield return RouteTokens.All;
        }
    }
}
