// ORIGINAL REFERENCE MODEL TEST RUNNER -- synthetic assertions only.
// Not reconstructed or decompiled source; no semantic, runtime, or binary parity claim.
// BCL-only, no packages, no fixtures, no filesystem/process APIs. Console is the only I/O.
// STATUS IN THIS TASK: IMPLEMENTED, NOT RUN. Do not read this file as an execution receipt.

using System;
using System.Collections.Generic;
using System.Linq;

namespace Event101Reference;

internal static class Event101Tests
{
    private static int _passed;
    private static int _failed;

    public static int Main()
    {
        Run("01 five payloads and mapped Hurt record", Group01_PayloadAndMappedHurtRecord);
        Run("02 registration lifecycle and target identity", Group02_RegistrationAndIdentity);
        Run("03 same-state reentry versus independent state", Group03_ReentryScope);
        Run("04 deferred conditional mutations", Group04_DeferredMutations);
        Run("05 shared mutation, cached snapshot, fresh flags", Group05_SharedMutationSnapshot);
        Run("06 partial failure and sticky emitting state", Group06_FailureBoundaries);
        Run("07 conversion, Heal and Other routing", Group07_ConversionAndRouting);
        Run("08 filter presence, values and current-ID restrictions", Group08_FilterModel);
        Run("09 killed capture and record ordering", Group09_KilledCapture);
        Run("10 synthetic routing and mutable fanout", Group10_Routing);

        Console.WriteLine($"TOTAL: passed={_passed} failed={_failed}");
        return _failed == 0 ? 0 : 1;
    }

    private static void Run(string name, Action test)
    {
        try
        {
            test();
            _passed++;
            Console.WriteLine($"PASS {name}");
        }
        catch (Exception exception)
        {
            _failed++;
            Console.WriteLine($"FAIL {name}: {exception.GetType().Name}: {exception.Message}");
        }
    }

    private static void Assert(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    private static void Equal<T>(T expected, T actual, string message)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new InvalidOperationException(
                $"{message}: expected '{expected}', actual '{actual}'");
        }
    }

    private static TException Throws<TException>(Action action, string message)
        where TException : Exception
    {
        try
        {
            action();
        }
        catch (TException exception)
        {
            return exception;
        }
        catch (Exception exception)
        {
            throw new InvalidOperationException(
                $"{message}: expected {typeof(TException).Name}, got {exception.GetType().Name}",
                exception);
        }

        throw new InvalidOperationException($"{message}: expected an exception, none was thrown");
    }

    private static DamageResult Hurt(float damage, long id = 1)
    {
        return new DamageResult
        {
            Damage = damage,
            DamageData = new DamageData(id, CalculationType.Hurt),
        };
    }

    private static DamageResult Heal(float damage, long id = 1)
    {
        return new DamageResult
        {
            Damage = damage,
            DamageData = new DamageData(id, CalculationType.Heal),
        };
    }

    private static bool Emit(
        EventBus bus,
        Entity target,
        Entity attacker,
        Entity victim,
        DamageResult result,
        Requirements requirements,
        int eventId = EventIds.CharDamageAll,
        Position position = default)
    {
        return bus.Emit(target, eventId, attacker, victim, requirements, result, position);
    }

    // 1. All five payloads keep identity/equality; enabled Hurt creates the mapped record.
    private static void Group01_PayloadAndMappedHurtRecord()
    {
        var bus = new EventBus();
        var consumer = new CombatConsumer();
        consumer.Filter[DamageFilterKeys.Damage] = true;

        var attacker = new Entity(11);
        var victim = new Entity(22);
        var requirements = new Requirements
        {
            SkillId = 7,
            Critical = true,
            Immune = false,
            Killed = false,
        };
        var result = new DamageResult
        {
            Damage = 12.9f,
            DamageData = new DamageData(3_000_000_000L, CalculationType.Hurt),
        };
        var position = new Position(1.25, -2.5, 3.75);

        Entity? seenAttacker = null;
        Entity? seenVictim = null;
        Requirements? seenRequirements = null;
        DamageResult? seenResult = null;
        Position seenPosition = default;
        int calls = 0;

        DamageEventHandler handler = (a, v, r, d, p) =>
        {
            calls++;
            seenAttacker = a;
            seenVictim = v;
            seenRequirements = r;
            seenResult = d;
            seenPosition = p;
            consumer.OnCharDamageAll(a, v, r, d, p);
        };

        Assert(bus.Add(victim, EventIds.CharDamageAll, handler), "registration should succeed");
        Assert(
            bus.Emit(
                victim,
                EventIds.CharDamageAll,
                attacker,
                victim,
                requirements,
                result,
                position),
            "emit should return true");

        Equal(1, calls, "handler call count");
        Assert(ReferenceEquals(attacker, seenAttacker), "attacker identity");
        Assert(ReferenceEquals(victim, seenVictim), "victim identity");
        Assert(ReferenceEquals(requirements, seenRequirements), "requirements identity");
        Assert(ReferenceEquals(result, seenResult), "result identity");
        Assert(seenPosition == position, "position value equality");

        Equal(1, consumer.All.Count, "All record count");
        Equal(1, consumer.View.Count, "View record count");
        Assert(ReferenceEquals(consumer.All[0], consumer.View[0]), "All/View same instance");

        DamageRecord record = (DamageRecord)consumer.All[0];
        Equal(11, record.AttackerId, "record attacker id");
        Equal(22, record.TargetId, "record target id");
        Equal(3_000_000_000L, record.DamageId, "record damage id above Int32.MaxValue");
        Equal(12, record.DamageValue, "record truncated damage");
        Equal(7, record.SkillId, "record skill id");

        consumer.CombatEnabled = false;
        Assert(
            Emit(bus, victim, attacker, victim, result, requirements),
            "bus emit remains true when consumer is disabled");
        Equal(1, consumer.All.Count, "disabled combat adds no record");
    }

    // 2. Missing state versus retained state; duplicate add/remove; reference target identity.
    private static void Group02_RegistrationAndIdentity()
    {
        var bus = new EventBus();
        var attacker = new Entity(1);
        var targetA = new Entity(42);
        var targetB = new Entity(42);
        var requirements = new Requirements();
        var result = Hurt(9f);

        Assert(
            !Emit(bus, targetA, attacker, targetA, result, requirements),
            "missing target/id must be false");

        int calls = 0;
        DamageEventHandler handler = (a, v, r, d, p) => calls++;

        Assert(bus.Add(targetA, EventIds.CharDamageAll, handler), "first add");
        Assert(!bus.Add(targetA, EventIds.CharDamageAll, handler), "duplicate active add");
        Assert(
            Emit(bus, targetA, attacker, targetA, result, requirements),
            "registered emit");
        Equal(1, calls, "registered handler count");

        Assert(
            Emit(bus, targetA, attacker, targetA, result, requirements, eventId: 999),
            "existing target with an unregistered event name completes without delivery");
        Equal(1, calls, "unregistered event name invokes no listener");
        Assert(
            !bus.Remove(targetA, 999, handler),
            "emission did not register the absent event name");

        Assert(bus.Remove(targetA, EventIds.CharDamageAll, handler), "first remove");
        Assert(!bus.Remove(targetA, EventIds.CharDamageAll, handler), "duplicate remove");
        Assert(
            Emit(bus, targetA, attacker, targetA, result, requirements),
            "retained state emits true with no listeners");
        Equal(1, calls, "removed handler does not run");

        Assert(
            !Emit(bus, targetB, attacker, targetB, result, requirements),
            "same numeric id on a different reference has no state");

        Assert(
            bus.Add(targetB, EventIds.CharDamageAll, (a, v, r, d, p) => calls += 10),
            "add on second reference");
        Assert(
            Emit(bus, targetB, attacker, targetB, result, requirements),
            "second reference emits");
        Equal(11, calls, "second reference handler runs separately");
        Assert(
            Emit(bus, targetA, attacker, targetA, result, requirements),
            "first reference state remains");
        Equal(11, calls, "first reference has no listeners again");
    }

    // 3. Same target/id recursion is rejected; another target/id is not globally blocked.
    private static void Group03_ReentryScope()
    {
        var bus = new EventBus();
        var attacker = new Entity(1);
        var target = new Entity(2);
        var other = new Entity(3);
        var requirements = new Requirements();
        var result = Hurt(1f);

        int sameCalls = 0;
        int otherCalls = 0;
        int differentIdCalls = 0;
        bool? recursiveResult = null;
        bool? differentResult = null;
        bool? differentIdResult = null;

        DamageEventHandler reentrant = (a, v, r, d, p) =>
        {
            sameCalls++;
            recursiveResult = bus.Emit(target, EventIds.CharDamageAll, a, v, r, d, p);
            differentResult = bus.Emit(other, EventIds.CharDamageAll, a, v, r, d, p);
            differentIdResult = bus.Emit(target, 102, a, v, r, d, p);
        };

        bus.Add(target, EventIds.CharDamageAll, reentrant);
        bus.Add(other, EventIds.CharDamageAll, (a, v, r, d, p) => otherCalls++);
        bus.Add(target, 102, (a, v, r, d, p) => differentIdCalls++);

        Assert(
            Emit(bus, target, attacker, target, result, requirements),
            "outer emit returns true");

        Equal(1, sameCalls, "outer handler runs once");
        Assert(recursiveResult == false, "recursive same-target emit must be false");
        Assert(differentResult == true, "different target/id emit must be true");
        Equal(1, otherCalls, "different target state handler runs once");
        Assert(differentIdResult == true, "different event id on same target must be true");
        Equal(1, differentIdCalls, "different event id state handler runs once");
    }

    // 4. Conditional mutations are deferred within the emitting state.
    private static void Group04_DeferredMutations()
    {
        var attacker = new Entity(1);
        var victim = new Entity(2);
        var requirements = new Requirements();

        // 4a. Remove an ordinary listener during conditional dispatch: skip this emit.
        {
            var bus = new EventBus();
            int ordinaryCalls = 0;
            bool removed = false;
            DamageEventHandler ordinary = (a, v, r, d, p) => ordinaryCalls++;
            DamageEventHandler conditional = (a, v, r, d, p) =>
            {
                removed = bus.Remove(victim, EventIds.CharDamageAll, ordinary);
            };

            bus.Add(victim, EventIds.CharDamageAll, ordinary);
            bus.AddConditional(victim, EventIds.CharDamageAll, attacker, conditional);

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4a emit");
            Assert(removed, "4a remove returned true");
            Equal(0, ordinaryCalls, "4a pending-removed ordinary is skipped");
        }

        // 4b. Add an ordinary listener during dispatch: effective only on the next emit.
        {
            var bus = new EventBus();
            int addedCalls = 0;
            bool added = false;
            DamageEventHandler addedHandler = (a, v, r, d, p) => addedCalls++;
            DamageEventHandler conditional = (a, v, r, d, p) =>
            {
                if (!added)
                {
                    added = bus.Add(victim, EventIds.CharDamageAll, addedHandler);
                }
            };

            bus.AddConditional(victim, EventIds.CharDamageAll, attacker, conditional);

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4b first emit");
            Assert(added, "4b deferred add accepted");
            Equal(0, addedCalls, "4b deferred add does not run in same emit");

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4b second emit");
            Equal(1, addedCalls, "4b deferred add runs next emit");
        }

        // 4c. Add then remove a pending addition: cancellation leaves no listener.
        {
            var bus = new EventBus();
            int temporaryCalls = 0;
            bool added = false;
            bool removed = false;
            DamageEventHandler temporary = (a, v, r, d, p) => temporaryCalls++;
            DamageEventHandler conditional = (a, v, r, d, p) =>
            {
                added = bus.Add(victim, EventIds.CharDamageAll, temporary);
                removed = bus.Remove(victim, EventIds.CharDamageAll, temporary);
            };

            bus.AddConditional(victim, EventIds.CharDamageAll, attacker, conditional);

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4c first emit");
            Assert(added, "4c pending add accepted");
            Assert(removed, "4c pending add removed");

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4c second emit");
            Equal(0, temporaryCalls, "4c canceled pending add never runs");
        }

        // 4d. Re-add an active listener scheduled for removal: removal is canceled.
        {
            var bus = new EventBus();
            int ordinaryCalls = 0;
            bool firstRemove = false;
            bool duplicateRemove = false;
            bool readd = false;
            DamageEventHandler ordinary = (a, v, r, d, p) => ordinaryCalls++;
            DamageEventHandler conditional = (a, v, r, d, p) =>
            {
                firstRemove = bus.Remove(victim, EventIds.CharDamageAll, ordinary);
                duplicateRemove = bus.Remove(victim, EventIds.CharDamageAll, ordinary);
                readd = bus.Add(victim, EventIds.CharDamageAll, ordinary);
            };

            bus.Add(victim, EventIds.CharDamageAll, ordinary);
            bus.AddConditional(victim, EventIds.CharDamageAll, attacker, conditional);

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4d first emit");
            Assert(firstRemove, "4d first remove accepted");
            Assert(!duplicateRemove, "4d duplicate pending remove rejected");
            Assert(readd, "4d re-add cancels removal");
            Equal(1, ordinaryCalls, "4d canceled removal still runs now");

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), requirements),
                "4d second emit");
            Equal(2, ordinaryCalls, "4d listener remains active next emit");
        }
    }

    // 5. Conditional mutates shared objects before ordinary; consumer caches then re-reads.
    private static void Group05_SharedMutationSnapshot()
    {
        var bus = new EventBus();
        var consumer = new CombatConsumer();
        consumer.Filter[DamageFilterKeys.Damage] = true;

        var attacker = new Entity(11);
        var victim = new Entity(22);
        var requirements = new Requirements
        {
            SkillId = 3,
            Critical = false,
            Immune = false,
            Killed = false,
        };
        var result = new DamageResult
        {
            Damage = 9.9f,
            DamageData = new DamageData(700, CalculationType.Hurt),
        };

        bool ordinarySawConditionalMutation = false;
        bool observationSeen = false;
        CombatSnapshot observation = default;

        consumer.HurtStatistics = (rq, dr, amount) =>
        {
            Equal(4, amount, "statistics sees cached converted amount");
            dr.Damage = 1000f;
            dr.DamageData = new DamageData(999, CalculationType.Heal);
            rq.Critical = true;
            rq.Immune = true;
            rq.Killed = false;
            rq.SkillId = 123;
        };
        consumer.HurtObservation = snapshot =>
        {
            observationSeen = true;
            observation = snapshot;
        };

        DamageEventHandler conditional = (a, v, r, d, p) =>
        {
            d.Damage = 4.8f;
            d.DamageData = new DamageData(800, CalculationType.Hurt);
            r.Critical = true;
            r.Immune = true;
            r.Killed = true;
            r.SkillId = 99;
        };

        DamageEventHandler ordinary = (a, v, r, d, p) =>
        {
            ordinarySawConditionalMutation =
                ReferenceEquals(r, requirements) &&
                ReferenceEquals(d, result) &&
                r.Killed == true &&
                d.Damage == 4.8f &&
                d.DamageData.Id == 800;

            consumer.OnCharDamageAll(a, v, r, d, p);
        };

        bus.AddConditional(victim, EventIds.CharDamageAll, attacker, conditional);
        bus.Add(victim, EventIds.CharDamageAll, ordinary);

        Assert(
            Emit(bus, victim, attacker, victim, result, requirements),
            "emit should succeed");
        Assert(ordinarySawConditionalMutation, "ordinary sees conditional mutations by reference");

        Equal(1, consumer.All.Count, "one record");
        DamageRecord record = (DamageRecord)consumer.All[0];
        Equal(800L, record.DamageId, "cached DamageData id survives statistics mutation");
        Equal(4, record.DamageValue, "cached converted damage survives statistics mutation");
        Equal(123, record.SkillId, "SkillId is read after statistics");
        Assert(observationSeen, "observation hook ran");
        Assert(observation.Critical, "fresh critical flag");
        Assert(observation.Immune, "fresh immune flag");
        Assert(observation.Killed == false, "fresh nullable killed flag overrides conditional value");
        Equal(4, observation.DamageValue, "observation sees cached amount");
        Equal(800L, observation.Data.Id, "observation sees cached data id");
        Equal(CalculationType.Hurt, observation.Data.Calculation, "observation sees cached calculation");

        Equal(1000f, result.Damage, "shared result was mutated");
        Equal(999L, result.DamageData.Id, "shared result data was replaced");
        Equal(CalculationType.Heal, result.DamageData.Calculation, "shared data calculation mutated");
    }

    // 6. Partial persistence, bus error containment, and sticky emitting after sink failure.
    private static void Group06_FailureBoundaries()
    {
        var attacker = new Entity(1);
        var victim = new Entity(2);

        // 6a. Formatter failure precedes All insertion; the bus catches it and can emit again.
        {
            var bus = new EventBus();
            var consumer = new CombatConsumer();
            consumer.Filter[DamageFilterKeys.Damage] = true;
            consumer.Formatter = _ => throw new InvalidOperationException("formatter fault");

            bus.Add(victim, EventIds.CharDamageAll, consumer.OnCharDamageAll);
            Assert(
                Emit(bus, victim, attacker, victim, Hurt(3f), new Requirements()),
                "6a formatter failure is contained by bus");
            Equal(0, consumer.All.Count, "6a no All record before formatter completes");
            Equal(1, bus.HandlerErrors.Count, "6a handler error logged");

            consumer.Formatter = null;
            Assert(
                Emit(bus, victim, attacker, victim, Hurt(3f), new Requirements()),
                "6a later emission succeeds");
            Equal(1, consumer.All.Count, "6a later formatter-free record lands");
            Equal(1, consumer.View.Count, "6a later record passes filter");
            Equal(1, bus.HandlerErrors.Count, "6a error count unchanged on success");
        }

        // 6b. Filter hook failure leaves All persisted, no View, no killed progression.
        {
            var bus = new EventBus();
            var consumer = new CombatConsumer();
            consumer.Filter[DamageFilterKeys.Damage] = true;
            consumer.Filter[DamageFilterKeys.Killed] = true;
            consumer.BeforeFilter =
                (_, _) => throw new InvalidOperationException("filter fault");

            bus.Add(victim, EventIds.CharDamageAll, consumer.OnCharDamageAll);
            Assert(
                Emit(
                    bus,
                    victim,
                    attacker,
                    victim,
                    Hurt(3f),
                    new Requirements { Killed = true }),
                "6b filter failure is contained by bus");
            Equal(1, consumer.All.Count, "6b All persists after filter failure");
            Equal(0, consumer.View.Count, "6b no View after filter failure");
            Equal(1, bus.HandlerErrors.Count, "6b handler error logged");
            Assert(consumer.All[0] is DamageRecord, "6b no killed record progression");
        }

        // 6c. Inner handler-error sink throws; outer sink logs; emit stays sticky false.
        {
            int handlerSinkCalls = 0;
            int dispatchSinkCalls = 0;
            var bus = new EventBus(
                handlerErrorSink: _ =>
                {
                    handlerSinkCalls++;
                    throw new InvalidOperationException("inner sink fault");
                },
                dispatchErrorSink: _ => dispatchSinkCalls++);
            var target = new Entity(3);

            bus.Add(
                target,
                EventIds.CharDamageAll,
                (a, v, r, d, p) => throw new InvalidOperationException("handler fault"));

            Assert(
                !Emit(bus, target, attacker, target, Hurt(1f), new Requirements()),
                "6c escaping sink failure returns false");
            Equal(1, handlerSinkCalls, "6c handler sink invoked once");
            Equal(1, dispatchSinkCalls, "6c outer sink invoked once");

            Assert(
                !Emit(bus, target, attacker, target, Hurt(1f), new Requirements()),
                "6c emitting bit is sticky after escaped failure");
            Equal(1, handlerSinkCalls, "6c no second handler dispatch");
            Equal(1, dispatchSinkCalls, "6c no second outer log");
        }

        // 6d. Conditional handler throws; a normally returning sink permits ordinary dispatch.
        {
            var bus = new EventBus();
            int ordinaryCalls = 0;

            bus.AddConditional(
                victim,
                EventIds.CharDamageAll,
                attacker,
                (a, v, r, d, p) => throw new InvalidOperationException("conditional fault"));
            bus.Add(victim, EventIds.CharDamageAll, (a, v, r, d, p) => ordinaryCalls++);

            Assert(
                Emit(bus, victim, attacker, victim, Hurt(1f), new Requirements()),
                "6d conditional failure is contained and emit returns true");
            Equal(1, bus.HandlerErrors.Count, "6d handler error logged");
            Equal(1, ordinaryCalls, "6d ordinary listener still runs after conditional failure");
        }
    }

    // 7. Domain conversion, sign/truncation, Heal sign, Other no-op, explicit model guard.
    private static void Group07_ConversionAndRouting()
    {
        var attacker = new Entity(10);
        var victim = new Entity(20);
        var requirements = new Requirements();
        var consumer = new CombatConsumer();
        consumer.Filter[DamageFilterKeys.Damage] = true;

        consumer.OnCharDamageAll(attacker, victim, requirements, Hurt(3.9f, 101), default);
        Equal(3, ((DamageRecord)consumer.All[0]).DamageValue, "positive fractional truncation");

        consumer.OnCharDamageAll(attacker, victim, requirements, Hurt(-3.9f, 102), default);
        Equal(-3, ((DamageRecord)consumer.All[1]).DamageValue, "negative fractional truncation");

        int healStatistics = 0;
        int healConsumer = 0;
        consumer.HealStatistics = (_, _, amount) => healStatistics = amount;
        consumer.HealConsumer = amount => healConsumer = amount;

        consumer.OnCharDamageAll(attacker, victim, requirements, Heal(2.75f, 103), default);
        Equal(-2, healStatistics, "heal statistics sees negated amount");
        Equal(-2, healConsumer, "heal consumer sees negated amount");
        Equal(2, consumer.All.Count, "heal creates no damage record");
        Equal(2, consumer.View.Count, "heal creates no view record");

        int before = consumer.All.Count;
        consumer.OnCharDamageAll(
            attacker,
            victim,
            requirements,
            new DamageResult
            {
                Damage = 5f,
                DamageData = new DamageData(104, CalculationType.Other),
            },
            default);
        Equal(before, consumer.All.Count, "Other creates no record");
        Equal(-2, healConsumer, "Other does not invoke heal consumer");

        Throws<ArgumentOutOfRangeException>(
            () => consumer.OnCharDamageAll(
                attacker, victim, requirements, Hurt(float.NaN), default),
            "NaN is outside the model input domain");
        Throws<ArgumentOutOfRangeException>(
            () => consumer.OnCharDamageAll(
                attacker, victim, requirements, Hurt(float.PositiveInfinity), default),
            "positive infinity is outside the model input domain");
        Throws<ArgumentOutOfRangeException>(
            () => consumer.OnCharDamageAll(
                attacker, victim, requirements, Hurt(2147483648f), default),
            "float-rounded Int32.MaxValue+1 is outside the model input domain");

        Equal(before, consumer.All.Count, "rejected inputs add no partial record");

        int disabledStatsCalls = 0;
        int disabledStatsAmount = 0;
        consumer.HurtStatistics = (_, _, amount) =>
        {
            disabledStatsCalls++;
            disabledStatsAmount = amount;
        };
        consumer.CombatEnabled = false;

        Throws<ArgumentOutOfRangeException>(
            () => consumer.OnCharDamageAll(
                attacker, victim, requirements, Hurt(float.NaN), default),
            "disabled combat still enforces the conversion domain");
        Equal(0, disabledStatsCalls, "conversion guard runs before statistics");

        consumer.OnCharDamageAll(attacker, victim, requirements, Hurt(7.9f, 105), default);
        Equal(1, disabledStatsCalls, "disabled combat still runs Hurt statistics");
        Equal(7, disabledStatsAmount, "disabled statistics sees converted amount");
        Equal(before, consumer.All.Count, "disabled combat adds no All record");
        Equal(before, consumer.View.Count, "disabled combat adds no View record");
    }

    // 8. Presence-based filter, positive Current restriction, and unrestricted record ids.
    private static void Group08_FilterModel()
    {
        var attacker = new Entity(11);
        var victim = new Entity(22);
        var requirements = new Requirements();
        DamageResult result = Hurt(5.8f, 55);

        var falseValue = new CombatConsumer();
        falseValue.Filter[DamageFilterKeys.Damage] = false;
        falseValue.OnCharDamageAll(attacker, victim, requirements, result, default);
        Equal(1, falseValue.All.Count, "false key value still records All");
        Equal(1, falseValue.View.Count, "false key value still admits View by presence");
        Assert(
            ReferenceEquals(falseValue.All[0], falseValue.View[0]),
            "false key value keeps same instance");

        var missing = new CombatConsumer();
        missing.OnCharDamageAll(attacker, victim, requirements, result, default);
        Equal(1, missing.All.Count, "missing key keeps All");
        Equal(0, missing.View.Count, "missing key rejects View");

        var attackerMismatch = new CombatConsumer();
        attackerMismatch.Filter[DamageFilterKeys.Damage] = true;
        attackerMismatch.CurrentAttackerId = attacker.Id + 1;
        attackerMismatch.OnCharDamageAll(attacker, victim, requirements, result, default);
        Equal(1, attackerMismatch.All.Count, "positive current attacker mismatch keeps All");
        Equal(0, attackerMismatch.View.Count, "positive current attacker mismatch rejects View");

        var targetMismatch = new CombatConsumer();
        targetMismatch.Filter[DamageFilterKeys.Damage] = true;
        targetMismatch.CurrentTargetId = victim.Id + 1;
        targetMismatch.OnCharDamageAll(attacker, victim, requirements, result, default);
        Equal(1, targetMismatch.All.Count, "positive current target mismatch keeps All");
        Equal(0, targetMismatch.View.Count, "positive current target mismatch rejects View");

        var matching = new CombatConsumer();
        matching.Filter[DamageFilterKeys.Damage] = true;
        matching.CurrentAttackerId = attacker.Id;
        matching.CurrentTargetId = victim.Id;
        matching.OnCharDamageAll(attacker, victim, requirements, result, default);
        Equal(1, matching.All.Count, "matching current restriction keeps All");
        Equal(1, matching.View.Count, "matching current restriction admits View");

        var unrestrictedRecordIds = new CombatConsumer();
        unrestrictedRecordIds.Filter[DamageFilterKeys.Damage] = true;
        unrestrictedRecordIds.CurrentAttackerId = 0;
        unrestrictedRecordIds.CurrentTargetId = -1;
        unrestrictedRecordIds.OnCharDamageAll(
            new Entity(0),
            new Entity(-1),
            requirements,
            result,
            default);
        Equal(1, unrestrictedRecordIds.All.Count, "zero/negative record ids keep All");
        Equal(
            1,
            unrestrictedRecordIds.View.Count,
            "zero/negative current restriction admits zero/negative record ids");

        var partial = new CombatConsumer();
        partial.Filter[DamageFilterKeys.Damage] = true;
        partial.BeforeFilter = (_, _) => throw new InvalidOperationException("filter hook fault");
        Throws<InvalidOperationException>(
            () => partial.OnCharDamageAll(attacker, victim, requirements, result, default),
            "BeforeFilter fault escapes the consumer");
        Equal(1, partial.All.Count, "BeforeFilter fault retains partial All");
        Equal(0, partial.View.Count, "BeforeFilter fault prevents View");
    }

    // 9. Null/false killed emits only Damage; true emits Damage then Killed; captured early.
    private static void Group09_KilledCapture()
    {
        var attacker = new Entity(11);
        var victim = new Entity(22);
        DamageResult result = Hurt(7.9f, 77);

        foreach (bool? killed in new bool?[] { null, false })
        {
            var consumer = new CombatConsumer();
            consumer.Filter[DamageFilterKeys.Damage] = true;
            consumer.Filter[DamageFilterKeys.Killed] = true;
            consumer.OnCharDamageAll(
                attacker,
                victim,
                new Requirements { Killed = killed },
                result,
                default);
            Equal(1, consumer.All.Count, $"killed={killed} emits one record");
            Assert(consumer.All[0] is DamageRecord, $"killed={killed} record is DamageRecord");
        }

        var killedConsumer = new CombatConsumer();
        killedConsumer.Filter[DamageFilterKeys.Damage] = true;
        killedConsumer.Filter[DamageFilterKeys.Killed] = true;
        var requirements = new Requirements { Killed = true };
        var trace = new List<string>();

        killedConsumer.Formatter = record =>
        {
            trace.Add(record is DamageRecord ? "format:damage" : "format:killed");
            if (record is DamageRecord)
            {
                // Killed capture happened before formatting. This later mutation does not
                // cancel the already captured true; the second record is still expected.
                requirements.Killed = false;
            }
        };
        killedConsumer.BeforeFilter = (_, key) => trace.Add($"filter:{key}");

        killedConsumer.OnCharDamageAll(attacker, victim, requirements, result, default);

        Equal(2, killedConsumer.All.Count, "true killed emits two records");
        Equal(2, killedConsumer.View.Count, "both records pass filter");
        Assert(killedConsumer.All[0] is DamageRecord, "first record is DamageRecord");
        Assert(killedConsumer.All[1] is KilledRecord, "second record is KilledRecord");
        Assert(ReferenceEquals(killedConsumer.All[0], killedConsumer.View[0]), "damage All/View ref");
        Assert(ReferenceEquals(killedConsumer.All[1], killedConsumer.View[1]), "killed All/View ref");
        Equal(11, killedConsumer.All[1].AttackerId, "killed attacker id");
        Equal(22, killedConsumer.All[1].TargetId, "killed target id");

        Equal(
            "format:damage|filter:0|format:killed|filter:4",
            string.Join("|", trace),
            "format/filter trace order");

        var falseKilledValue = new CombatConsumer();
        falseKilledValue.Filter[DamageFilterKeys.Damage] = true;
        falseKilledValue.Filter[DamageFilterKeys.Killed] = false;
        falseKilledValue.OnCharDamageAll(
            attacker,
            victim,
            new Requirements { Killed = true },
            result,
            default);
        Equal(2, falseKilledValue.All.Count, "false killed key value still emits");
        Equal(2, falseKilledValue.View.Count, "false killed key value still admits");
    }

    // 10. Actual owner/local/all broadcasts; callbacks observe prior mutation; null target.
    private static void Group10_Routing()
    {
        var bus = new EventBus();
        var attacker = new Entity(1);
        var victim = new Entity(2);
        var localTarget = new Entity(3);
        var allTarget = new Entity(4);
        bool teamItemExists = false;
        bool isMyRole = true;
        bool formationMatches = true;
        bool ownerEmitted = false;
        bool teamSawOwner = false;
        int teamChecks = 0;
        int roleChecks = 0;
        int formationChecks = 0;

        var requirements = new Requirements();
        DamageResult result = Hurt(1f);
        var position = new Position(1.25, -2.5, 3.75);
        var trace = new List<string>();
        var payloadChecks = new List<bool>();

        void Record(string label, Entity a, Entity v, Requirements r, DamageResult d, Position p)
        {
            trace.Add(label);
            payloadChecks.Add(
                ReferenceEquals(a, attacker) &&
                ReferenceEquals(v, victim) &&
                ReferenceEquals(r, requirements) &&
                ReferenceEquals(d, result) &&
                p == position);
        }

        bus.Add(
            victim,
            EventIds.CharDamageAll,
            (a, v, r, d, p) =>
            {
                Record("Owner", a, v, r, d, p);
                ownerEmitted = true;
                teamItemExists = true;
            });
        bus.Add(
            localTarget,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => Record("Local", a, v, r, d, p));
        bus.Add(
            allTarget,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => Record("All", a, v, r, d, p));

        List<string> tokens = Event101Routing.Route(
                attacker,
                victim,
                localTarget,
                allTarget,
                bus,
                EventIds.CharDamageAll,
                requirements,
                result,
                position,
                () =>
                {
                    teamSawOwner = ownerEmitted;
                    teamChecks++;
                    return teamItemExists;
                },
                () =>
                {
                    roleChecks++;
                    return isMyRole;
                },
                () =>
                {
                    formationChecks++;
                    return formationMatches;
                })
            .ToList();

        Equal(
            "Owner|Local|All|Local|All",
            string.Join("|", tokens),
            "route token order");
        Equal(
            "Owner|Local|All|Local|All",
            string.Join("|", trace),
            "actual bus handler trace");
        Equal(5, tokens.Count, "no token de-duplication");
        Equal(2, tokens.Count(token => token == RouteTokens.Local), "two Local tokens");
        Equal(2, tokens.Count(token => token == RouteTokens.All), "two All tokens");
        Equal(1, tokens.Count(token => token == RouteTokens.Owner), "one Owner token");
        Equal(2, trace.Count(item => item == "Local"), "Local broadcast twice");
        Equal(2, trace.Count(item => item == "All"), "All broadcast twice");
        Equal(1, trace.Count(item => item == "Owner"), "owner broadcast once");
        Equal(5, payloadChecks.Count, "every actual broadcast observed");
        Assert(
            payloadChecks.All(value => value),
            "every broadcast keeps original attacker/victim/requirements/result/position");
        Assert(teamSawOwner, "owner emission precedes condition callback");
        Assert(teamItemExists, "owner callback mutated team state before fanout lookup");
        Equal(1, teamChecks, "team callback invoked once");
        Equal(1, roleChecks, "role callback invoked once");
        Equal(1, formationChecks, "formation callback invoked once");

        var teamOnlyBus = new EventBus();
        var teamOnlyVictim = new Entity(22);
        var teamOnlyLocal = new Entity(32);
        var teamOnlyAll = new Entity(42);
        int teamOnlyLocalCalls = 0;
        int teamOnlyAllCalls = 0;
        teamOnlyBus.Add(
            teamOnlyLocal,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => teamOnlyLocalCalls++);
        teamOnlyBus.Add(
            teamOnlyAll,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => teamOnlyAllCalls++);

        List<string> teamOnlyTokens = Event101Routing.Route(
                attacker,
                teamOnlyVictim,
                teamOnlyLocal,
                teamOnlyAll,
                teamOnlyBus,
                EventIds.CharDamageAll,
                requirements,
                result,
                position,
                () => true,
                () => false,
                () => false)
            .ToList();
        Equal(
            "Owner|All",
            string.Join("|", teamOnlyTokens),
            "team item without my-role broadcasts All only");
        Equal(0, teamOnlyLocalCalls, "not-my-role suppresses Local broadcast");
        Equal(1, teamOnlyAllCalls, "not-my-role still broadcasts All");

        var noOwnerBus = new EventBus();
        var noOwnerVictim = new Entity(21);
        var noOwnerLocal = new Entity(31);
        var noOwnerAll = new Entity(41);
        int noOwnerLocalCalls = 0;
        int noOwnerAllCalls = 0;
        noOwnerBus.Add(
            noOwnerLocal,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => noOwnerLocalCalls++);
        noOwnerBus.Add(
            noOwnerAll,
            EventIds.CharDamageAll,
            (a, v, r, d, p) => noOwnerAllCalls++);

        List<string> noOwnerTokens = Event101Routing.Route(
                attacker,
                noOwnerVictim,
                noOwnerLocal,
                noOwnerAll,
                noOwnerBus,
                EventIds.CharDamageAll,
                requirements,
                result,
                position,
                () => true,
                () => true,
                () => true)
            .ToList();
        Equal(
            "Owner|Local|All|Local|All",
            string.Join("|", noOwnerTokens),
            "absent owner listener does not stop later broadcasts");
        Equal(2, noOwnerLocalCalls, "Local broadcasts continue without owner state");
        Equal(2, noOwnerAllCalls, "All broadcasts continue without owner state");

        int nullTargetEmissions = 0;
        int nullCallbacks = 0;
        var nullBus = new EventBus();
        var nullLocal = new Entity(51);
        var nullAll = new Entity(61);
        nullBus.Add(victim, EventIds.CharDamageAll, (a, v, r, d, p) => nullTargetEmissions++);
        List<string> nullTargetTokens = Event101Routing.Route(
                attacker,
                null,
                nullLocal,
                nullAll,
                nullBus,
                EventIds.CharDamageAll,
                requirements,
                result,
                position,
                () =>
                {
                    nullCallbacks++;
                    return true;
                },
                () =>
                {
                    nullCallbacks++;
                    return true;
                },
                () =>
                {
                    nullCallbacks++;
                    return true;
                })
            .ToList();
        Equal(0, nullTargetTokens.Count, "null target yields no tokens");
        Equal(0, nullTargetEmissions, "null target emits nothing");
        Equal(0, nullCallbacks, "null target does not evaluate routing callbacks");
    }
}
