// ORIGINAL REFERENCE MODEL TEST RUNNER -- synthetic assertions only.
// These tests exercise the independently authored model, not recovered game code.
// Expected literals are authored, not computed by the model under test.
// A passing model test is not CIL, runtime, server, binary, or game-behavior evidence.
// BCL-only. Console is the only I/O. No filesystem, process, network, or clock API.

using System;
using System.Collections.Generic;
using System.Globalization;

namespace SkillDecisionReference;

internal static class SkillDecisionTests
{
    private static int _passed;
    private static int _failed;

    public static int Main()
    {
        Run("01 first group entry decides, not the highest entry", GroupOneUsesFirstEntry);
        Run("02 priority pass still reads debug before the logical OR", PriorityPassStillReadsDebug);
        Run("03 ready-end or debug allows a lower-priority skill", ReadyOrDebugAllowsLowerPriority);
        Run("04 easy interrupt does not cross unequal priorities", EasyInterruptDoesNotCrossPriorities);
        Run("05 equal priority plus easy interrupt accepts", EqualPriorityEasyInterrupt);
        Run("06 no current skill skips debug and cooldown callbacks", NoCurrentSkipsCallbacks);
        Run("07 other group reads cooldown before reference and clears on hit", OtherGroupCooldownOrderAndClear);
        Run("08 a later cooldown throw keeps earlier accepted state", LaterCooldownThrowKeepsEarlierState);
        Run("09 duplicate matching references append twice", DuplicateMatchingReferencesAppendTwice);
        Run("10 equal ids are different references but share the cooldown key", EqualIdsDifferentReferences);
        Run("11 missing, null, and empty groups are no-ops", MissingNullAndEmptyGroups);
        Run("12 cooldown availability boundary values and errors", CooldownAvailabilityBoundaries);
        Run("13 restored count 0 to 1 changes the lookup", RestoredCountChangesLookup);
        Run("14 two catch-up charges leave a 50 residual and schedule next", CatchUpTwoCharges);
        Run("15 late catch-up caps the count at the limit", LateCatchUpCaps);
        Run("16 incomplete pending pair pops nothing but still restores one", IncompletePairPopsNothing);
        Run("17 duration pop throw keeps the id removed and initial restore", DurationPopThrowKeepsIdRemoved);
        Run("18 notify throw keeps counts and skips scheduling", NotifyThrowKeepsState);
        Run("19 zero, negative, and NaN durations use the exact boundary", DurationBoundaryValues);
        Run("20 invalid elapsed and null ports fail before mutation", GuardBeforeMutation);

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

    private static void EqualSequence<T>(
        IReadOnlyList<T> expected,
        IReadOnlyList<T> actual,
        string message)
    {
        Equal(expected.Count, actual.Count, message + " count");
        for (int i = 0; i < expected.Count; i++)
        {
            Equal(expected[i], actual[i], $"{message} at {i}");
        }
    }

    private static void Same(Skill expected, Skill actual, string message)
    {
        Assert(ReferenceEquals(expected, actual), message);
    }

    private static void Contains(string text, string fragment, string message)
    {
        if (!text.Contains(fragment, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"{message}: '{text}' does not contain '{fragment}'");
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

    private static List<Skill> Skills(params Skill[] items)
    {
        return new List<Skill>(items);
    }

    private static string FormatFloat(float value)
    {
        return float.IsNaN(value)
            ? "nan"
            : value.ToString("R", CultureInfo.InvariantCulture);
    }

    // 1. The first entry decides. Scanning the highest entry would reject the first case.
    private static void GroupOneUsesFirstEntry()
    {
        var firstLow = new Skill(1, 1, 1);
        var secondHigh = new Skill(2, 1, 100);
        var decisions = new SkillDecisions
        {
            ReadyEnd = false,
            EasyInterrupt = false,
            ReadDebug = () => false,
        };
        decisions.ActiveGroups[1] = Skills(firstLow, secondHigh);
        var output = new List<Skill>();
        var reasons = new List<string>();

        Assert(
            decisions.Check(new Skill(3, 1, 50), output, reasons),
            "A lower first entry must allow the incoming skill; a highest-entry scan would reject it.");
        Equal(1, output.Count, "Only the first entry is appended.");
        Same(firstLow, output[0], "The first entry, not the second high entry, is selected.");
        Equal(0, reasons.Count, "An accepted call adds no reasons.");

        var reversed = new SkillDecisions
        {
            ReadyEnd = false,
            EasyInterrupt = false,
            ReadDebug = () => false,
        };
        reversed.ActiveGroups[1] = Skills(secondHigh, firstLow);
        var reversedOutput = new List<Skill> { new Skill(99, 1, 0) };
        var reversedReasons = new List<string>();

        Assert(
            !reversed.Check(new Skill(3, 1, 50), reversedOutput, reversedReasons),
            "A higher first entry must reject; scanning the lower second entry would accept.");
        Equal(0, reversedOutput.Count, "Rejection clears the whole output list.");
        EqualSequence(
            new[] { "100", "50", "False", "False" },
            reversedReasons,
            "Rejection appends current priority, incoming priority, easy flag, and ready flag.");
    }

    // 2. A priority pass must not short-circuit the debug read. A short-circuit would not throw.
    private static void PriorityPassStillReadsDebug()
    {
        var current = new Skill(1, 1, 10);
        var decisions = new SkillDecisions
        {
            ReadyEnd = true,
            EasyInterrupt = false,
            ReadDebug = () => throw new InvalidOperationException("debug callback must run"),
        };
        decisions.ActiveGroups[1] = Skills(current);
        var output = new List<Skill>();
        var reasons = new List<string>();

        Throws<InvalidOperationException>(
            () => decisions.Check(new Skill(2, 1, 20), output, reasons),
            "The debug read must execute even when priority and ready-end already accept.");
        Equal(0, output.Count, "The throw occurs before any output append.");
        Equal(0, reasons.Count, "The throw occurs before any reason append.");
    }

    // 3. Either ready-end or debug allows a lower priority. Using only priority would reject.
    private static void ReadyOrDebugAllowsLowerPriority()
    {
        var current = new Skill(1, 1, 100);
        var lower = new Skill(2, 1, 10);

        var ready = new SkillDecisions
        {
            ReadyEnd = true,
            EasyInterrupt = false,
            ReadDebug = () => false,
        };
        ready.ActiveGroups[1] = Skills(current);
        var output = new List<Skill> { current, new Skill(90, 1, 0) };
        var reasons = new List<string> { "kept" };

        Assert(ready.Check(lower, output, reasons), "ReadyEnd true must accept the lower skill.");
        Equal(3, output.Count, "Accept preserves existing entries and appends without de-duplication.");
        Same(current, output[0], "The preexisting current entry is preserved.");
        Equal(90, output[1].Id, "The unrelated preexisting entry is preserved.");
        Same(current, output[2], "The selected current entry is appended again.");
        Equal(1, reasons.Count, "Accept does not clear reasons.");
        Equal("kept", reasons[0], "Accept does not add reasons.");

        var debug = new SkillDecisions
        {
            ReadyEnd = false,
            EasyInterrupt = false,
            ReadDebug = () => true,
        };
        debug.ActiveGroups[1] = Skills(current);
        var debugOutput = new List<Skill>();
        var debugReasons = new List<string>();

        Assert(debug.Check(lower, debugOutput, debugReasons), "Debug true must accept the lower skill.");
        Same(current, debugOutput[0], "The selected current entry is appended.");
        Equal(0, debugReasons.Count, "An accepted call adds no reasons.");
    }

    // 4. Easy interrupt is equal-priority only. Applying it to a lower skill would accept.
    private static void EasyInterruptDoesNotCrossPriorities()
    {
        var current = new Skill(1, 1, 100);
        var lower = new Skill(2, 1, 10);
        var decisions = new SkillDecisions
        {
            EasyInterrupt = true,
            ReadyEnd = false,
            ReadDebug = () => false,
        };
        decisions.ActiveGroups[1] = Skills(current);
        var output = new List<Skill> { new Skill(99, 1, 0) };
        var reasons = new List<string>();

        Assert(
            !decisions.Check(lower, output, reasons),
            "EasyInterrupt must not accept a lower-priority skill.");
        Equal(0, output.Count, "Rejection clears the whole output list.");
        EqualSequence(
            new[] { "100", "10", "True", "False" },
            reasons,
            "The four reason strings retain the easy flag but no ready-end flag.");
    }

    // 5. Equal priority plus easy interrupt accepts; equal priority without it rejects.
    private static void EqualPriorityEasyInterrupt()
    {
        var current = new Skill(1, 1, 50);
        var incoming = new Skill(2, 1, 50);

        var easy = new SkillDecisions
        {
            EasyInterrupt = true,
            ReadyEnd = false,
            ReadDebug = () => false,
        };
        easy.ActiveGroups[1] = Skills(current);
        var output = new List<Skill>();
        var reasons = new List<string>();

        Assert(easy.Check(incoming, output, reasons), "Equal priority plus easy interrupt must accept.");
        Same(current, output[0], "The equal-priority current entry is selected.");
        Equal(0, reasons.Count, "An accepted call adds no reasons.");

        var strict = new SkillDecisions
        {
            EasyInterrupt = false,
            ReadyEnd = false,
            ReadDebug = () => false,
        };
        strict.ActiveGroups[1] = Skills(current);
        var strictOutput = new List<Skill>();
        var strictReasons = new List<string>();

        Assert(
            !strict.Check(incoming, strictOutput, strictReasons),
            "Equal priority without easy interrupt must reject.");
        Equal(0, strictOutput.Count, "Rejection clears the whole output list.");
        EqualSequence(
            new[] { "50", "50", "False", "False" },
            strictReasons,
            "The four reason strings describe the equal-priority rejection.");
    }

    // 6. With no current group-1 skill, neither debug nor cooldown may be read.
    private static void NoCurrentSkipsCallbacks()
    {
        var decisions = new SkillDecisions
        {
            ReadDebug = () => throw new InvalidOperationException("debug must not run"),
            ReadCooldown = _ => throw new InvalidOperationException("cooldown must not run"),
        };
        var sentinel = new Skill(9, 1, 0);
        var output = new List<Skill> { sentinel };
        var reasons = new List<string> { "kept" };

        Assert(
            decisions.Check(new Skill(7, 1, 5), output, reasons),
            "Missing group-1 state must allow the call.");
        Equal(1, output.Count, "The output list is preserved.");
        Same(sentinel, output[0], "The existing output entry is unchanged.");
        Equal(1, reasons.Count, "The reasons list is preserved.");
        Equal("kept", reasons[0], "The existing reason is unchanged.");
    }

    // 7. Cooldown is read before reference selection, and a hit clears preexisting output.
    private static void OtherGroupCooldownOrderAndClear()
    {
        var first = new Skill(10, 2, 0);
        var matching = new Skill(20, 2, 0);
        var output = new List<Skill> { new Skill(99, 2, 0) };
        int observedOutputCount = -1;
        var decisions = new SkillDecisions();
        decisions.ReadCooldown = id =>
        {
            if (id == 20)
            {
                observedOutputCount = output.Count;
            }
            return false;
        };
        decisions.ActiveGroups[2] = Skills(first, matching);
        var reasons = new List<string> { "kept" };

        Assert(decisions.Check(matching, output, reasons), "No cooldown hit means the call is accepted.");
        Equal(
            1,
            observedOutputCount,
            "Cooldown is read before reference selection; selection would have made the output count 2.");
        Equal(2, output.Count, "The matching reference is appended after the nonmatching entry.");
        Same(matching, output[1], "Only the actual matching reference is appended.");
        Equal(1, reasons.Count, "An accepted call does not clear reasons.");

        var rejectOutput = new List<Skill> { new Skill(99, 2, 0) };
        var rejectReasons = new List<string> { "kept" };
        var rejectDecisions = new SkillDecisions { ReadCooldown = id => id == 10 };
        rejectDecisions.ActiveGroups[2] = Skills(first, matching);

        Assert(
            !rejectDecisions.Check(matching, rejectOutput, rejectReasons),
            "A cooldown hit on a nonmatching entry rejects the whole call.");
        Equal(0, rejectOutput.Count, "Rejection clears preexisting output entries.");
        Equal(2, rejectReasons.Count, "Rejection appends one reason and leaves earlier reasons.");
        Equal("kept", rejectReasons[0], "The preexisting reason is not cleared.");
        Contains(rejectReasons[1], "Skill 10", "The authored reason contains the skill id.");
    }

    // 8. A later cooldown throw must not roll back the earlier accepted reference.
    private static void LaterCooldownThrowKeepsEarlierState()
    {
        var matching = new Skill(1, 2, 0);
        var thrower = new Skill(2, 2, 0);
        var output = new List<Skill> { new Skill(99, 2, 0) };
        var reasons = new List<string> { "kept" };
        var decisions = new SkillDecisions();
        decisions.ReadCooldown = id =>
        {
            if (id == 2)
            {
                throw new InvalidOperationException("later cooldown read");
            }
            return false;
        };
        decisions.ActiveGroups[2] = Skills(matching, thrower);

        Throws<InvalidOperationException>(
            () => decisions.Check(matching, output, reasons),
            "The later cooldown exception escapes the model.");
        Equal(2, output.Count, "The earlier accepted reference remains after the later throw.");
        Equal(99, output[0].Id, "The preexisting output entry remains.");
        Same(matching, output[1], "The incoming reference accepted before the throw remains.");
        Equal(1, reasons.Count, "The reasons list is unchanged by the throw.");
        Equal("kept", reasons[0], "The preexisting reason is unchanged.");
    }

    // 9. Both matching references are appended. A de-duplicating implementation would add one.
    private static void DuplicateMatchingReferencesAppendTwice()
    {
        var incoming = new Skill(5, 2, 0);
        var decisions = new SkillDecisions { ReadCooldown = _ => false };
        decisions.ActiveGroups[2] = Skills(incoming, incoming);
        var output = new List<Skill>();
        var reasons = new List<string>();

        Assert(decisions.Check(incoming, output, reasons), "The group has no cooldown hit.");
        Equal(2, output.Count, "Both matching references are appended with no de-duplication.");
        Same(incoming, output[0], "The first reference is appended.");
        Same(incoming, output[1], "The duplicate reference is appended again.");
        Equal(0, reasons.Count, "An accepted call adds no reasons.");
    }

    // 10. Selection is by reference, but cooldown lookup is by numeric id.
    private static void EqualIdsDifferentReferences()
    {
        var first = new Skill(7, 2, 0);
        var sameId = new Skill(7, 2, 0);
        var decisions = new SkillDecisions { ReadCooldown = _ => false };
        decisions.ActiveGroups[2] = Skills(first);
        var output = new List<Skill>();

        Assert(
            decisions.Check(sameId, output),
            "A different object with the same id must not be selected by id equality.");
        Equal(0, output.Count, "Reference selection does not append the same-id object.");

        var charges = new Dictionary<long, ChargeState?> { [7] = new ChargeState(0, 3) };
        var availability = new CooldownAvailability(charges);
        var cooldownDecisions = new SkillDecisions { ReadCooldown = availability.IsUnavailable };
        cooldownDecisions.ActiveGroups[2] = Skills(first);
        var cooldownOutput = new List<Skill>();

        Assert(
            !cooldownDecisions.Check(sameId, cooldownOutput),
            "The shared cooldown map is keyed by id, so the same-id object is unavailable.");
        Equal(0, cooldownOutput.Count, "The cooldown rejection clears output.");
    }

    // 11. Missing, null, and empty groups are no-ops. Treating them as rejection would fail.
    private static void MissingNullAndEmptyGroups()
    {
        var decisions = new SkillDecisions
        {
            ReadDebug = () => throw new InvalidOperationException("debug must not run"),
            ReadCooldown = _ => throw new InvalidOperationException("cooldown must not run"),
        };
        var output = new List<Skill> { new Skill(99, 1, 0) };
        var reasons = new List<string> { "kept" };

        Assert(decisions.Check(new Skill(1, 1, 0), output, reasons), "Missing group-1 state is allowed.");
        Equal(1, output.Count, "Missing group-1 state preserves output.");
        Equal(1, reasons.Count, "Missing group-1 state preserves reasons.");

        decisions.ActiveGroups[1] = null;
        Assert(decisions.Check(new Skill(1, 1, 0), output, reasons), "A null group-1 list is allowed.");
        Equal(1, output.Count, "A null group-1 list preserves output.");

        decisions.ActiveGroups[1] = new List<Skill>();
        Assert(decisions.Check(new Skill(1, 1, 0), output, reasons), "An empty group-1 list is allowed.");
        Equal(1, output.Count, "An empty group-1 list preserves output.");
        Equal(1, reasons.Count, "No group-1 rejection reason is added.");

        var otherOutput = new List<Skill> { new Skill(99, 2, 0) };
        var otherReasons = new List<string> { "kept" };
        Assert(decisions.Check(new Skill(1, 2, 0), otherOutput, otherReasons), "Missing group-2 state is allowed.");
        Equal(1, otherOutput.Count, "Missing group-2 state preserves output.");

        decisions.ActiveGroups[2] = null;
        Assert(decisions.Check(new Skill(1, 2, 0), otherOutput, otherReasons), "A null group-2 list is allowed.");
        Equal(1, otherOutput.Count, "A null group-2 list preserves output.");

        decisions.ActiveGroups[2] = new List<Skill>();
        Assert(decisions.Check(new Skill(1, 2, 0), otherOutput, otherReasons), "An empty group-2 list is allowed.");
        Equal(1, otherOutput.Count, "An empty group-2 list preserves output.");
        Equal(1, otherReasons.Count, "No group-2 rejection reason is added.");

        Assert(decisions.Check(new Skill(1, 3, 0)), "Null optional lists are allowed.");
    }

    // 12. Missing keys and missing components are false; null state is an error.
    private static void CooldownAvailabilityBoundaries()
    {
        var map = new Dictionary<long, ChargeState?>
        {
            [1] = new ChargeState(1, 3),
            [2] = new ChargeState(0, 3),
            [3] = new ChargeState(-1, 3),
            [4] = null,
        };
        var availability = new CooldownAvailability(map);

        Assert(!availability.IsUnavailable(1), "Remaining count 1 is available (false).");
        Assert(availability.IsUnavailable(2), "Remaining count 0 is unavailable (true).");
        Assert(availability.IsUnavailable(3), "Remaining count -1 is unavailable (true).");
        Assert(!availability.IsUnavailable(99), "A missing key is available (false).");
        Throws<NullReferenceException>(
            () => availability.IsUnavailable(4),
            "A present null group is an error.");

        var uninitialized = new CooldownAvailability(null);
        Throws<NullReferenceException>(
            () => uninitialized.IsUnavailable(1),
            "A null map is an uninitialized component and errors.");

        var skill = new Skill(5, 2, 0);
        var noComponent = new SkillDecisions();
        noComponent.ActiveGroups[2] = Skills(skill);
        Assert(
            noComponent.Check(skill),
            "A null cooldown callback means the missing component answers false.");
    }

    // 13. Completion restores the shared charge; the lookup must observe 1.
    // A cached answer, or a missing completion-to-lookup integration, would fail.
    private static void RestoredCountChangesLookup()
    {
        var charge = new ChargeState(0, 2);
        var map = new Dictionary<long, ChargeState?> { [5] = charge };
        var availability = new CooldownAvailability(map);
        var skill = new Skill(5, 2, 0);
        var decisions = new SkillDecisions { ReadCooldown = availability.IsUnavailable };
        decisions.ActiveGroups[2] = Skills(skill);
        var output = new List<Skill>();
        var reasons = new List<string>();

        Assert(!decisions.Check(skill, output, reasons), "Remaining count 0 rejects the call.");
        Equal(0, output.Count, "The rejection clears output.");
        Contains(reasons[0], "Skill 5", "The rejection names the skill id.");

        var completionTrace = new List<string>();
        var pending = new ScriptedCooldowns(
            Array.Empty<int>(),
            Array.Empty<float>(),
            completionTrace);
        bool scheduleCalled = false;
        ChargeCompletion.Complete(
            charge,
            0f,
            pending,
            () => completionTrace.Add("notify"),
            (id, delay) =>
            {
                scheduleCalled = true;
                completionTrace.Add($"schedule:{id}:{FormatFloat(delay)}");
            });

        Equal(1, charge.RemainingCount, "Completion restores the initial charge.");
        Equal(0, pending.SkillIdCount, "The empty id queue stays empty.");
        Equal(0, pending.DurationCount, "The empty duration queue stays empty.");
        Assert(!scheduleCalled, "Empty pending queues must not schedule.");
        EqualSequence(new[] { "notify" }, completionTrace, "Empty pending queues notify without scheduling.");

        Assert(!availability.IsUnavailable(5), "Remaining count 1 is available (false).");
        var acceptedOutput = new List<Skill>();
        var acceptedReasons = new List<string>();
        Assert(decisions.Check(skill, acceptedOutput, acceptedReasons), "The restored count is visible through the callback.");
        Same(skill, acceptedOutput[0], "The restored skill is accepted by reference.");
        Equal(0, acceptedReasons.Count, "An accepted call adds no reasons.");
    }

    // 14. Wrong pop order, notify order, or a residual other than 50 would fail the trace.
    private static void CatchUpTwoCharges()
    {
        var state = new ChargeState(0, 3);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(
            new[] { 101, 202 },
            new[] { 100f, 200f },
            trace);

        ChargeCompletion.Complete(
            state,
            250f,
            pending,
            () => trace.Add("notify"),
            (id, delay) => trace.Add($"schedule:{id}:{FormatFloat(delay)}"));

        Equal(2, state.RemainingCount, "The initial restore plus one catch-up charge yields 2.");
        Equal(0, pending.SkillIdCount, "Both pending ids were consumed.");
        Equal(0, pending.DurationCount, "Both pending durations were consumed.");
        EqualSequence(
            new[]
            {
                "pop-id:101",
                "pop-duration:100",
                "pop-id:202",
                "pop-duration:200",
                "notify",
                "schedule:202:50",
            },
            trace,
            "Pop order, notify order, and the 50 residual are exact.");
    }

    // 15. Missing the upper cap would leave seven; scheduling after consumed pairs would add a call.
    private static void LateCatchUpCaps()
    {
        var state = new ChargeState(2, 3);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(
            new[] { 1, 2, 3, 4 },
            new[] { 100f, 200f, 300f, 400f },
            trace);

        ChargeCompletion.Complete(
            state,
            1000f,
            pending,
            () => trace.Add("notify"),
            (id, delay) => trace.Add($"schedule:{id}:{FormatFloat(delay)}"));

        Equal(3, state.RemainingCount, "The upper cap assigns LimitCount after the late catch-up.");
        Equal(0, pending.SkillIdCount, "All pending ids were consumed.");
        Equal(0, pending.DurationCount, "All pending durations were consumed.");
        EqualSequence(
            new[]
            {
                "pop-id:1",
                "pop-duration:100",
                "pop-id:2",
                "pop-duration:200",
                "pop-id:3",
                "pop-duration:300",
                "pop-id:4",
                "pop-duration:400",
                "notify",
            },
            trace,
            "No schedule call remains after the final pair is consumed.");
    }

    // 16. An incomplete pair is not popped, but the initial restore still happens.
    private static void IncompletePairPopsNothing()
    {
        var stateA = new ChargeState(0, 3);
        var traceA = new List<string>();
        var pendingA = new ScriptedCooldowns(new[] { 5 }, Array.Empty<float>(), traceA);
        ChargeCompletion.Complete(
            stateA,
            10f,
            pendingA,
            () => traceA.Add("notify"),
            (id, delay) => traceA.Add("schedule"));

        Equal(1, stateA.RemainingCount, "The initial increment happens even with a missing duration.");
        Equal(1, pendingA.SkillIdCount, "The id remains unpopped because the pair is incomplete.");
        Equal(0, pendingA.DurationCount, "The duration queue is still empty.");
        EqualSequence(new[] { "notify" }, traceA, "No pop and no schedule occur for the incomplete pair.");

        var stateB = new ChargeState(0, 3);
        var traceB = new List<string>();
        var pendingB = new ScriptedCooldowns(Array.Empty<int>(), new[] { 5f }, traceB);
        ChargeCompletion.Complete(
            stateB,
            10f,
            pendingB,
            () => traceB.Add("notify"),
            (id, delay) => traceB.Add("schedule"));

        Equal(1, stateB.RemainingCount, "The initial increment happens even with a missing id.");
        Equal(0, pendingB.SkillIdCount, "The id queue is still empty.");
        Equal(1, pendingB.DurationCount, "The duration remains unpopped because the pair is incomplete.");
        EqualSequence(new[] { "notify" }, traceB, "No pop and no schedule occur for the incomplete pair.");
    }

    // 17. The id is popped before the duration. Popping the duration first would leave the id.
    private static void DurationPopThrowKeepsIdRemoved()
    {
        var state = new ChargeState(0, 3);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(new[] { 5 }, new[] { 9f }, trace)
        {
            ThrowOnDurationPop = true,
        };

        Throws<InvalidOperationException>(
            () => ChargeCompletion.Complete(
                state,
                10f,
                pending,
                () => trace.Add("notify"),
                (id, delay) => trace.Add("schedule")),
            "The duration pop exception escapes the model.");
        Equal(1, state.RemainingCount, "The initial increment persists after the throw.");
        Equal(0, pending.SkillIdCount, "The id was removed before the duration pop threw.");
        Equal(1, pending.DurationCount, "The duration was not removed.");
        EqualSequence(new[] { "pop-id:5" }, trace, "No notify or schedule occurs after the throw.");
    }

    // 18. Notify runs after the initial restore and pop. Scheduling before notify
    // would add a schedule trace before the throw, so the zero-elapsed pair is eligible.
    private static void NotifyThrowKeepsState()
    {
        var state = new ChargeState(0, 3);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(new[] { 101 }, new[] { 100f }, trace);

        Throws<InvalidOperationException>(
            () => ChargeCompletion.Complete(
                state,
                0f,
                pending,
                () =>
                {
                    trace.Add("notify");
                    throw new InvalidOperationException("notify");
                },
                (id, delay) => trace.Add("schedule")),
            "The notify exception escapes the model.");
        Equal(1, state.RemainingCount, "The initial restore persists after notify throws.");
        Equal(0, pending.SkillIdCount, "The popped id remains removed after notify throws.");
        Equal(0, pending.DurationCount, "The popped duration remains removed after notify throws.");
        EqualSequence(
            new[] { "pop-id:101", "pop-duration:100", "notify" },
            trace,
            "No schedule call occurs before notify returns, and none occurs after the throw.");
    }

    // 19. A positive-only duration guard would not consume zero and would change the count.
    private static void DurationBoundaryValues()
    {
        var state = new ChargeState(0, 10);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(
            new[] { 1, 2, 3 },
            new[] { 0f, -4f, float.NaN },
            trace);
        int? scheduledId = null;
        float? scheduledDelay = null;

        ChargeCompletion.Complete(
            state,
            0f,
            pending,
            () => trace.Add("notify"),
            (id, delay) =>
            {
                scheduledId = id;
                scheduledDelay = delay;
                trace.Add("schedule");
            });

        Equal(3, state.RemainingCount, "Zero and negative durations consume; NaN stops the loop.");
        Assert(scheduledId.HasValue, "The remaining NaN-duration pair is scheduled.");
        int scheduledIdValue = scheduledId.GetValueOrDefault();
        Equal(3, scheduledIdValue, "The third pending id remains for the next schedule.");
        bool delayIsNaN = scheduledDelay.HasValue && float.IsNaN(scheduledDelay.GetValueOrDefault());
        Assert(
            delayIsNaN,
            "The scheduled delay is NaN because a NaN duration remains.");
        EqualSequence(
            new[]
            {
                "pop-id:1",
                "pop-duration:0",
                "pop-id:2",
                "pop-duration:-4",
                "pop-id:3",
                "pop-duration:nan",
                "notify",
                "schedule",
            },
            trace,
            "The exact zero and negative comparisons and the NaN false comparison are visible.");
    }

    // 20. Incrementing or popping before validation would change the observed state.
    private static void GuardBeforeMutation()
    {
        var state = new ChargeState(0, 3);
        var trace = new List<string>();
        var pending = new ScriptedCooldowns(new[] { 1 }, new[] { 1f }, trace);

        foreach (float invalid in new[] { -1f, float.NaN, float.PositiveInfinity })
        {
            Throws<ArgumentOutOfRangeException>(
                () => ChargeCompletion.Complete(
                    state,
                    invalid,
                    pending,
                    () => trace.Add("notify"),
                    (id, delay) => trace.Add("schedule")),
                "Invalid elapsed must be rejected before mutation.");
            Equal(0, state.RemainingCount, "The count is unchanged after invalid elapsed.");
            Equal(1, pending.SkillIdCount, "The id queue is unchanged after invalid elapsed.");
            Equal(1, pending.DurationCount, "The duration queue is unchanged after invalid elapsed.");
            Equal(0, trace.Count, "No pop, notify, or schedule occurs after invalid elapsed.");
        }

        Throws<ArgumentNullException>(
            () => ChargeCompletion.Complete(null!, 0f, pending, () => { }, (id, delay) => { }),
            "A null state is rejected.");
        Throws<ArgumentNullException>(
            () => ChargeCompletion.Complete(state, 0f, null!, () => { }, (id, delay) => { }),
            "A null pending port is rejected.");
        Throws<ArgumentNullException>(
            () => ChargeCompletion.Complete(state, 0f, pending, null!, (id, delay) => { }),
            "A null notify port is rejected.");
        Throws<ArgumentNullException>(
            () => ChargeCompletion.Complete(state, 0f, pending, () => { }, null!),
            "A null schedule port is rejected.");

        Equal(0, state.RemainingCount, "No null-port guard mutated the count.");
        Equal(0, trace.Count, "No null-port guard popped, notified, or scheduled.");
    }

    private sealed class ScriptedCooldowns : IPendingCooldowns
    {
        private readonly Queue<int> _skillIds;
        private readonly Queue<float> _durations;
        private readonly List<string> _trace;

        public ScriptedCooldowns(
            IEnumerable<int> skillIds,
            IEnumerable<float> durations,
            List<string> trace)
        {
            _skillIds = new Queue<int>(skillIds);
            _durations = new Queue<float>(durations);
            _trace = trace;
        }

        public bool ThrowOnDurationPop { get; set; }

        public int SkillIdCount => _skillIds.Count;

        public int DurationCount => _durations.Count;

        public int PopSkillId()
        {
            int id = _skillIds.Dequeue();
            _trace.Add($"pop-id:{id}");
            return id;
        }

        public float PopDuration()
        {
            if (ThrowOnDurationPop)
            {
                throw new InvalidOperationException("scripted duration pop");
            }

            float duration = _durations.Dequeue();
            _trace.Add($"pop-duration:{FormatFloat(duration)}");
            return duration;
        }
    }
}
