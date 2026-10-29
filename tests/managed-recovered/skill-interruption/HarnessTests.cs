// Authored self-runner for RE-SKILL-INTERRUPTION01.
// PREPARE ONLY: this file is not compiled or executed until the primary
// authorizes the execute phase.
//
// It contains NO recovered method body. The four exact canonical
// BaseSkillComponent slices live in the private fixture. Every expected value
// below is a literal, a reference to an input object, an input-derived ID, or a
// literal reason string; no expected value is obtained by calling the SUT.
//
// No test framework, reflection, file, process, network, thread, clock or RNG
// API is used. Only System.Console, System.Environment and
// System.Runtime.InteropServices.RuntimeInformation are touched.
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

internal static class Program
{
    private static int _groupsPassed;
    private static int _groupsFailed;

    private static readonly (string Sid, Action Body)[] Tests =
    {
        ("G1", G1_CurrentSkillGuards),
        ("G2", G2_Group1PriorityEasyReadyDebug),
        ("G3", G3_Group1ReasonsAndSuccessAppend),
        ("G4", G4_CheckSkillCanInterruptWrapper),
        ("G5", G5_NonGroupDictionaryAndIdentity),
        ("G6", G6_CooldownFirstAndReason),
        ("G7", G7_ExceptionAfterAppend),
        ("G8", G8_NullOptionalAndCurrentFirst)
    };

    private static int Main()
    {
        Console.WriteLine("RE-SKILL-INTERRUPTION01 SELF-RUNNER");
        Console.WriteLine("EXECUTION_PATH=DIRECT_CSC_ASSEMBLY");
        Console.WriteLine("FRAMEWORK=" + RuntimeInformation.FrameworkDescription);
        Console.WriteLine("ENVIRONMENT_VERSION=" + Environment.Version);
        Console.WriteLine("RUNTIME_IDENTIFIER=" + RuntimeInformation.RuntimeIdentifier);
        Console.WriteLine("TESTS_PLANNED=" + Tests.Length.ToString());
        Console.WriteLine("SIDS=" + Sids());

        int groupsWithAssertions = 0;
        for (int i = 0; i < Tests.Length; i++)
        {
            int assertions = RunGroup(Tests[i].Sid, Tests[i].Body);
            if (assertions > 0)
            {
                groupsWithAssertions++;
            }
        }

        bool shapeOk = Tests.Length == 8 && groupsWithAssertions == 8;
        bool resultOk = shapeOk && _groupsFailed == 0 && Assert.Failures == 0 && Assert.Assertions > 0;

        Console.WriteLine("GROUPS_WITH_ASSERTIONS=" + groupsWithAssertions.ToString());
        Console.WriteLine("GROUPS_PASSED=" + _groupsPassed.ToString());
        Console.WriteLine("GROUPS_FAILED=" + _groupsFailed.ToString());
        Console.WriteLine("ASSERTIONS_TOTAL=" + Assert.Assertions.ToString());
        Console.WriteLine("ASSERTIONS_PASSED=" + Assert.Passed.ToString());
        Console.WriteLine("ASSERTIONS_FAILED=" + Assert.Failures.ToString());
        Console.WriteLine("RESULT=" + (resultOk ? "PASS" : "FAIL"));
        Console.WriteLine("EXIT_CODE=" + (resultOk ? "0" : "1"));
        return resultOk ? 0 : 1;
    }

    private static string Sids()
    {
        string[] parts = new string[Tests.Length];
        for (int i = 0; i < Tests.Length; i++)
        {
            parts[i] = Tests[i].Sid;
        }
        return string.Join(",", parts);
    }

    private static int RunGroup(string sid, Action body)
    {
        Console.WriteLine("TEST " + sid);
        int beforeAssertions = Assert.Assertions;
        int beforeFailures = Assert.Failures;
        try
        {
            body();
        }
        catch (Exception ex)
        {
            Assert.Fail(sid + "-UNCAUGHT", "uncaught exception", ex.GetType().Name + ": " + ex.Message);
        }
        int assertions = Assert.Assertions - beforeAssertions;
        int failures = Assert.Failures - beforeFailures;
        Console.WriteLine("GROUP " + sid + " ASSERTIONS=" + assertions.ToString() + " FAILURES=" + failures.ToString());
        if (assertions > 0 && failures == 0)
        {
            _groupsPassed++;
        }
        else
        {
            _groupsFailed++;
        }
        return assertions;
    }

    private static BaseSkillComponentHost NewHost()
    {
        ModelBase<SkillCdModel>.Instance = null;
        return new BaseSkillComponentHost { IsMainSkillReadyEnd = false };
    }

    private static Skill NewSkill(int id, int group, int level)
    {
        return new Skill
        {
            SkillId = id,
            SkillInfo = new SkillInfo { GroupId = group },
            InterruptLevel = level
        };
    }

    private static CharacterSkillCdComponent NewCd(Func<int, bool>? impl = null)
    {
        return new CharacterSkillCdComponent { IsSkillInCdImpl = impl };
    }

    private static void SetDebug(bool value)
    {
        ModelBase<SkillCdModel>.Instance = new SkillCdModel { SkillDebugMode = value };
    }

    private static string CdReason(int id)
    {
        // Exact canonical reason shape: 技能{id}处于CD中.
        return "\u6280\u80fd" + id.ToString() + "\u5904\u4e8eCD\u4e2d";
    }

    private static void G1_CurrentSkillGuards()
    {
        BaseSkillComponentHost host = NewHost();
        Assert.NullSkill("G1-01", "missing group 1", host.CurrentSkill);
        Assert.Int("G1-02", "dictionary count after missing", 0, host.ActivatedSkillGroup.Count);

        host.ActivatedSkillGroup[1] = null!;
        Assert.NullSkill("G1-03", "null group 1 list", host.CurrentSkill);

        host.ActivatedSkillGroup[1] = new List<Skill>();
        Assert.NullSkill("G1-04", "empty group 1 list", host.CurrentSkill);

        Skill first = NewSkill(11, 1, 5);
        Skill second = NewSkill(99, 1, 9);
        host.ActivatedSkillGroup[1] = new List<Skill> { first, second };
        Assert.Same("G1-05", "first entry only", first, host.CurrentSkill);
        Assert.Bool("G1-06", "second higher not selected", false, ReferenceEquals(host.CurrentSkill, second));

        Skill inactive = NewSkill(12, 1, 5);
        inactive.Active = false;
        host.ActivatedSkillGroup[1] = new List<Skill> { inactive };
        Assert.Same("G1-07", "no Active predicate", inactive, host.CurrentSkill);

        CharacterSkillCdComponent cd = NewCd(_ => throw new InvalidOperationException("cd-must-not-be-called"));
        host.SkillCdComp = cd;
        Assert.Same("G1-08", "getter returns first without CD", inactive, host.CurrentSkill);
        Assert.Int("G1-09", "CD calls after getter", 0, cd.Calls.Count);
        Assert.Int("G1-10", "group 1 list count unchanged", 1, host.ActivatedSkillGroup[1].Count);
    }

    private static void G2_Group1PriorityEasyReadyDebug()
    {
        Group1Case("G2-01", 5, 6, false, false, false, true);
        Group1Case("G2-02", 5, 5, false, false, false, false);
        Group1Case("G2-03", 5, 5, true, false, false, true);
        Group1Case("G2-04", 5, 4, true, false, false, false);
        Group1Case("G2-05", 5, 4, false, true, false, true);
        Group1Case("G2-06", 5, 4, false, false, true, true);
        Group1Case("G2-07", 5, 4, false, false, false, false);
        Group1Case("G2-08", 5, 5, true, true, false, true);
        Group1Case("G2-09", 5, 5, false, true, false, true);
        Group1Case("G2-10", 5, 5, false, false, true, true);

        BaseSkillComponentHost host = NewHost();
        Skill current = NewSkill(1, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        bool debugRead = false;
        ModelBase<SkillCdModel>.Instance = new SkillCdModel
        {
            SkillDebugModeGetter = () =>
            {
                debugRead = true;
                throw new InvalidOperationException("debug-read");
            }
        };
        Skill sentinel = NewSkill(900, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };
        Assert.Catch<InvalidOperationException>("G2-11", "debug getter read before OR when priority true",
            () => host.DoCheckInterruptForTest(1, 6, outList, reasons, null));
        Assert.Bool("G2-12", "debug getter was read", true, debugRead);
        Assert.SameSeq("G2-13", "out list not cleared on throw", new List<Skill> { sentinel }, outList);
        Assert.Seq("G2-14", "reasons not changed on throw", new List<string> { "old" }, reasons);
        ModelBase<SkillCdModel>.Instance = null;
    }

    private static void Group1Case(string id, int currentLevel, int incomingLevel, bool easy, bool ready, bool debug, bool expected)
    {
        BaseSkillComponentHost host = NewHost();
        host.IsMainSkillEasyInterrupt = easy;
        host.IsMainSkillReadyEnd = ready;
        SetDebug(debug);
        Skill current = NewSkill(1, 1, currentLevel);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        List<Skill> outList = new List<Skill>();
        List<string> reasons = new List<string>();
        bool actual = host.DoCheckInterruptForTest(1, incomingLevel, outList, reasons, null);

        Assert.Bool(id + "-result", "group 1 result", expected, actual);
        List<Skill> expectedOut = expected ? new List<Skill> { current } : new List<Skill>();
        Assert.SameSeq(id + "-out", "group 1 out list", expectedOut, outList);
        List<string> expectedReasons = expected
            ? new List<string>()
            : new List<string> { currentLevel.ToString(), incomingLevel.ToString(), easy ? "True" : "False", ready ? "True" : "False" };
        Assert.Seq(id + "-reasons", "group 1 reasons", expectedReasons, reasons);
    }

    private static void G3_Group1ReasonsAndSuccessAppend()
    {
        BaseSkillComponentHost host = NewHost();
        Skill current = NewSkill(1, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Skill sentinel = NewSkill(901, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel, NewSkill(902, 2, 0) };
        List<string> reasons = new List<string> { "old" };

        bool actual = host.DoCheckInterruptForTest(1, 5, outList, reasons, null);
        Assert.Bool("G3-01-result", "group 1 reject", false, actual);
        Assert.Int("G3-01-out-count", "whole out list cleared", 0, outList.Count);
        Assert.Seq("G3-01-reasons", "old reason plus four literal fields",
            new List<string> { "old", "5", "5", "False", "False" }, reasons);
        Assert.Str("G3-01-old", "old reason preserved at index 0", "old", reasons.Count > 0 ? reasons[0] : "<missing>");

        host = NewHost();
        current = NewSkill(1, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        sentinel = NewSkill(903, 2, 0);
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };

        actual = host.DoCheckInterruptForTest(1, 6, outList, reasons, null);
        Assert.Bool("G3-02-result", "group 1 success", true, actual);
        Assert.SameSeq("G3-02-out", "success appends current and preserves old", new List<Skill> { sentinel, current }, outList);
        Assert.Seq("G3-02-reasons", "success preserves reasons", new List<string> { "old" }, reasons);

        actual = host.DoCheckInterruptForTest(1, 6, outList, reasons, null);
        Assert.Bool("G3-03-result", "second success", true, actual);
        Assert.SameSeq("G3-03-out", "duplicate current appended twice", new List<Skill> { sentinel, current, current }, outList);
        Assert.Seq("G3-03-reasons", "second success still preserves reasons", new List<string> { "old" }, reasons);

        host = NewHost();
        current = NewSkill(1, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Assert.Bool("G3-04", "null optional lists on reject", false, host.DoCheckInterruptForTest(1, 5, null, null, null));

        host = NewHost();
        current = NewSkill(1, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Assert.Bool("G3-05", "null optional lists on success", true, host.DoCheckInterruptForTest(1, 6, null, null, null));
    }

    private static void G4_CheckSkillCanInterruptWrapper()
    {
        BaseSkillComponentHost host = NewHost();
        CharacterSkillCdComponent cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        Skill other = NewSkill(20, 2, 9);
        Skill target = NewSkill(21, 2, 3);
        host.ActivatedSkillGroup[2] = new List<Skill> { other, target };
        Skill sentinel = NewSkill(904, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };

        bool actual = host.CheckSkillCanInterruptForTest(target, outList, reasons);
        Assert.Bool("G4-01-result", "wrapper group 2 branch", true, actual);
        Assert.SameSeq("G4-01-out", "wrapper passes incoming ref to exact group 2 scan",
            new List<Skill> { sentinel, target }, outList);
        Assert.Seq("G4-01-reasons", "wrapper group 2 preserves reasons", new List<string> { "old" }, reasons);
        Assert.IntSeq("G4-01-cd", "wrapper group 2 CD order", new List<int> { other.SkillId, target.SkillId }, cd.Calls);

        host = NewHost();
        Skill current = NewSkill(30, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Skill incoming = NewSkill(31, 1, 6);
        outList = new List<Skill>();
        reasons = new List<string>();

        actual = host.CheckSkillCanInterruptForTest(incoming, outList, reasons);
        Assert.Bool("G4-02-result", "wrapper group 1 branch", true, actual);
        Assert.SameSeq("G4-02-out", "wrapper group 1 appends current ref, not incoming", new List<Skill> { current }, outList);
        Assert.Seq("G4-02-reasons", "wrapper group 1 success has no reasons", new List<string>(), reasons);

        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        incoming.InterruptLevel = 4;
        outList = new List<Skill>();
        reasons = new List<string>();
        actual = host.CheckSkillCanInterruptForTest(incoming, outList, reasons);
        Assert.Bool("G4-03-result", "same object group 1 priority rejects", false, actual);
        Assert.Int("G4-03-out", "group 1 reject clears out", 0, outList.Count);
        Assert.Seq("G4-03-reasons", "group 1 reject reasons", new List<string> { "5", "4", "False", "False" }, reasons);

        incoming.SkillInfo.GroupId = 2;
        host.ActivatedSkillGroup[2] = new List<Skill> { incoming };
        host.SkillCdComp = NewCd(_ => false);
        outList = new List<Skill>();
        reasons = new List<string>();
        actual = host.CheckSkillCanInterruptForTest(incoming, outList, reasons);
        Assert.Bool("G4-04-result", "same object group 2 passes without priority", true, actual);
        Assert.SameSeq("G4-04-out", "group 2 selects same incoming ref", new List<Skill> { incoming }, outList);
        Assert.Seq("G4-04-reasons", "group 2 success has no reasons", new List<string>(), reasons);
    }

    private static void G5_NonGroupDictionaryAndIdentity()
    {
        BaseSkillComponentHost host = NewHost();
        CharacterSkillCdComponent cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        Skill sentinel = NewSkill(905, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };

        bool actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, null);
        Assert.Bool("G5-01-result", "missing non-group key returns true", true, actual);
        Assert.SameSeq("G5-01-out", "missing key preserves old out", new List<Skill> { sentinel }, outList);
        Assert.Seq("G5-01-reasons", "missing key preserves old reasons", new List<string> { "old" }, reasons);
        Assert.Int("G5-01-cd", "missing key does not call CD", 0, cd.Calls.Count);

        host.ActivatedSkillGroup[2] = null!;
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };
        cd.Calls.Clear();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, null);
        Assert.Bool("G5-02-result", "null non-group list returns true", true, actual);
        Assert.SameSeq("G5-02-out", "null list preserves old out", new List<Skill> { sentinel }, outList);
        Assert.Seq("G5-02-reasons", "null list preserves old reasons", new List<string> { "old" }, reasons);
        Assert.Int("G5-02-cd", "null list does not call CD", 0, cd.Calls.Count);

        host.ActivatedSkillGroup[2] = new List<Skill>();
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };
        cd.Calls.Clear();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, null);
        Assert.Bool("G5-03-result", "empty non-group list returns true", true, actual);
        Assert.SameSeq("G5-03-out", "empty list preserves old out", new List<Skill> { sentinel }, outList);
        Assert.Seq("G5-03-reasons", "empty list preserves old reasons", new List<string> { "old" }, reasons);
        Assert.Int("G5-03-cd", "empty list does not call CD", 0, cd.Calls.Count);

        Skill a = NewSkill(41, 2, 0);
        Skill b = NewSkill(42, 2, 0);
        Skill c = NewSkill(43, 2, 0);
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b, c };
        cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        outList = new List<Skill>();
        reasons = new List<string>();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G5-04-result", "reference identity pass", true, actual);
        Assert.SameSeq("G5-04-out", "only reference-equal B collected", new List<Skill> { b }, outList);
        Assert.Seq("G5-04-reasons", "reference identity success has no reasons", new List<string>(), reasons);
        Assert.IntSeq("G5-04-cd", "CD checks A B C in order", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);

        host = NewHost();
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b, c };
        cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        ModelBase<SkillCdModel>.Instance = new SkillCdModel
        {
            SkillDebugModeGetter = () => throw new InvalidOperationException("non-group-must-not-read-debug")
        };
        outList = new List<Skill>();
        reasons = new List<string>();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G5-05-result", "non-group ignores debug/flags", true, actual);
        Assert.SameSeq("G5-05-out", "non-group appends B despite debug double", new List<Skill> { b }, outList);
        Assert.IntSeq("G5-05-cd", "non-group CD order unchanged", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);
        ModelBase<SkillCdModel>.Instance = null;

        Skill sameId = NewSkill(b.SkillId, 2, 0);
        outList = new List<Skill>();
        reasons = new List<string>();
        cd.Calls.Clear();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, sameId);
        Assert.Bool("G5-06-result", "new object same id passes", true, actual);
        Assert.Int("G5-06-out", "new object same id not selected", 0, outList.Count);
        Assert.IntSeq("G5-06-cd", "new object same id still checks all", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);

        host.ActivatedSkillGroup[2] = new List<Skill> { b, b };
        cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        outList = new List<Skill>();
        reasons = new List<string>();
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G5-07-result", "duplicate B pass", true, actual);
        Assert.SameSeq("G5-07-out", "duplicate B appended twice", new List<Skill> { b, b }, outList);
        Assert.IntSeq("G5-07-cd", "duplicate B checked twice", new List<int> { b.SkillId, b.SkillId }, cd.Calls);
    }

    private static void G6_CooldownFirstAndReason()
    {
        BaseSkillComponentHost host = NewHost();
        Skill a = NewSkill(51, 2, 0);
        Skill b = NewSkill(52, 2, 0);
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b };
        CharacterSkillCdComponent cd = NewCd(id => id == a.SkillId);
        host.SkillCdComp = cd;
        Skill sentinel = NewSkill(906, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };

        bool actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G6-01-result", "nonmatching A in CD rejects", false, actual);
        Assert.Int("G6-01-out", "A CD clears whole out", 0, outList.Count);
        Assert.Seq("G6-01-reasons", "first CD reason and old preserved", new List<string> { "old", CdReason(a.SkillId) }, reasons);
        Assert.IntSeq("G6-01-cd", "A CD stops before B", new List<int> { a.SkillId }, cd.Calls);

        host = NewHost();
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b };
        cd = NewCd(id => id == b.SkillId);
        host.SkillCdComp = cd;
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G6-02-result", "matching B in CD rejects", false, actual);
        Assert.Int("G6-02-out", "B CD does not append and clears out", 0, outList.Count);
        Assert.Seq("G6-02-reasons", "B CD reason", new List<string> { "old", CdReason(b.SkillId) }, reasons);
        Assert.IntSeq("G6-02-cd", "CD checks A then B", new List<int> { a.SkillId, b.SkillId }, cd.Calls);

        Skill c = NewSkill(53, 2, 0);
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b, c };
        cd = NewCd(id => id == c.SkillId);
        host.SkillCdComp = cd;
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };
        actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G6-03-result", "late C CD rejects", false, actual);
        Assert.Int("G6-03-out", "late C CD clears preexisting and appended B", 0, outList.Count);
        Assert.Seq("G6-03-reasons", "late C CD reason", new List<string> { "old", CdReason(c.SkillId) }, reasons);
        Assert.IntSeq("G6-03-cd", "CD checks A B C", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);

        host.SkillCdComp = null;
        Assert.Bool("G6-04", "null CD component returns false", false, host.IsSkillInCd(123));

        cd = NewCd(_ => true);
        host.SkillCdComp = cd;
        Assert.Bool("G6-05", "nonnull CD forwards true", true, host.IsSkillInCd(123));
        Assert.IntSeq("G6-05-cd", "forwarding trace true", new List<int> { 123 }, cd.Calls);

        cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        Assert.Bool("G6-06", "nonnull CD forwards false", false, host.IsSkillInCd(123));
        Assert.IntSeq("G6-06-cd", "forwarding trace false", new List<int> { 123 }, cd.Calls);
    }

    private static void G7_ExceptionAfterAppend()
    {
        BaseSkillComponentHost host = NewHost();
        Skill a = NewSkill(61, 2, 0);
        Skill b = NewSkill(62, 2, 0);
        Skill c = NewSkill(63, 2, 0);
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b, c };
        CharacterSkillCdComponent cd = NewCd(id =>
        {
            if (id == c.SkillId)
            {
                throw new InvalidOperationException("cd-c");
            }
            return false;
        });
        host.SkillCdComp = cd;
        Skill sentinel = NewSkill(907, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };

        Assert.Catch<InvalidOperationException>("G7-01-ex", "CD throw on C after B appended",
            () => host.DoCheckInterruptForTest(2, 0, outList, reasons, b));
        Assert.SameSeq("G7-01-out", "exception leaves preexisting and appended B", new List<Skill> { sentinel, b }, outList);
        Assert.Seq("G7-01-reasons", "exception leaves reasons unchanged", new List<string> { "old" }, reasons);
        Assert.IntSeq("G7-01-cd", "exception trace includes A B C", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);

        host = NewHost();
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b, c };
        cd = NewCd(id => id == c.SkillId);
        host.SkillCdComp = cd;
        outList = new List<Skill> { sentinel };
        reasons = new List<string> { "old" };
        bool actual = host.DoCheckInterruptForTest(2, 0, outList, reasons, b);
        Assert.Bool("G7-02-result", "normal false C rejects", false, actual);
        Assert.Int("G7-02-out", "normal false C clears whole out", 0, outList.Count);
        Assert.Seq("G7-02-reasons", "normal false C appends CD reason", new List<string> { "old", CdReason(c.SkillId) }, reasons);
        Assert.IntSeq("G7-02-cd", "normal false trace A B C", new List<int> { a.SkillId, b.SkillId, c.SkillId }, cd.Calls);
    }

    private static void G8_NullOptionalAndCurrentFirst()
    {
        BaseSkillComponentHost host = NewHost();
        Skill current = NewSkill(71, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Assert.Bool("G8-01", "group 1 pass with null lists", true, host.DoCheckInterruptForTest(1, 6, null, null, null));

        host = NewHost();
        current = NewSkill(71, 1, 5);
        host.ActivatedSkillGroup[1] = new List<Skill> { current };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Assert.Bool("G8-02", "group 1 reject with null lists", false, host.DoCheckInterruptForTest(1, 5, null, null, null));

        host = NewHost();
        Skill a = NewSkill(72, 2, 0);
        Skill b = NewSkill(73, 2, 0);
        host.ActivatedSkillGroup[2] = new List<Skill> { a, b };
        CharacterSkillCdComponent cd = NewCd(_ => false);
        host.SkillCdComp = cd;
        Assert.Bool("G8-03", "non-group null lists", true, host.DoCheckInterruptForTest(2, 0, null, null, null));
        Assert.IntSeq("G8-03-cd", "non-group null lists still check all", new List<int> { a.SkillId, b.SkillId }, cd.Calls);

        host = NewHost();
        Assert.Bool("G8-04", "group 1 missing current with null lists", true, host.DoCheckInterruptForTest(1, 10, null, null, null));

        host = NewHost();
        Skill first = NewSkill(74, 1, 5);
        Skill second = NewSkill(75, 1, 9);
        host.ActivatedSkillGroup[1] = new List<Skill> { first, second };
        host.IsMainSkillEasyInterrupt = false;
        host.IsMainSkillReadyEnd = false;
        SetDebug(false);
        Skill sentinel = NewSkill(908, 2, 0);
        List<Skill> outList = new List<Skill> { sentinel };
        List<string> reasons = new List<string> { "old" };
        bool actual = host.DoCheckInterruptForTest(1, 6, outList, reasons, null);
        Assert.Bool("G8-05-result", "group 1 current first pass", true, actual);
        Assert.SameSeq("G8-05-out", "current first entry selected, second ignored", new List<Skill> { sentinel, first }, outList);
        Assert.Seq("G8-05-reasons", "group 1 current first preserves reasons", new List<string> { "old" }, reasons);
    }

    private static class Assert
    {
        public static int Assertions { get; private set; }
        public static int Passed { get; private set; }
        public static int Failures { get; private set; }

        public static void Bool(string id, string label, bool expected, bool observed)
        {
            Assertions++;
            bool ok = expected == observed;
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + (expected ? "True" : "False")
                + " observed=" + (observed ? "True" : "False")
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void Int(string id, string label, int expected, int observed)
        {
            Assertions++;
            bool ok = expected == observed;
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + expected.ToString()
                + " observed=" + observed.ToString()
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void Str(string id, string label, string expected, string observed)
        {
            Assertions++;
            bool ok = string.Equals(expected, observed, StringComparison.Ordinal);
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=\"" + expected + "\""
                + " observed=\"" + observed + "\""
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void NullSkill(string id, string label, Skill? observed)
        {
            Assertions++;
            bool ok = observed == null;
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=null observed=" + SkillDesc(observed)
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void Same(string id, string label, Skill expected, Skill? observed)
        {
            Assertions++;
            bool ok = ReferenceEquals(expected, observed);
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + SkillDesc(expected)
                + " observed=" + SkillDesc(observed)
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void SameSeq(string id, string label, IReadOnlyList<Skill> expected, IReadOnlyList<Skill> observed)
        {
            Assertions++;
            bool ok = expected.Count == observed.Count;
            if (ok)
            {
                for (int i = 0; i < expected.Count; i++)
                {
                    if (!ReferenceEquals(expected[i], observed[i]))
                    {
                        ok = false;
                        break;
                    }
                }
            }
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + SkillsDesc(expected)
                + " observed=" + SkillsDesc(observed)
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void Seq(string id, string label, IReadOnlyList<string> expected, IReadOnlyList<string> observed)
        {
            Assertions++;
            bool ok = expected.Count == observed.Count;
            if (ok)
            {
                for (int i = 0; i < expected.Count; i++)
                {
                    if (!string.Equals(expected[i], observed[i], StringComparison.Ordinal))
                    {
                        ok = false;
                        break;
                    }
                }
            }
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + StringsDesc(expected)
                + " observed=" + StringsDesc(observed)
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void IntSeq(string id, string label, IReadOnlyList<int> expected, IReadOnlyList<int> observed)
        {
            Assertions++;
            bool ok = expected.Count == observed.Count;
            if (ok)
            {
                for (int i = 0; i < expected.Count; i++)
                {
                    if (expected[i] != observed[i])
                    {
                        ok = false;
                        break;
                    }
                }
            }
            Console.WriteLine("  CASE " + id + " " + label
                + " expected=" + IntsDesc(expected)
                + " observed=" + IntsDesc(observed)
                + (ok ? " MATCH" : " MISMATCH"));
            Count(ok);
        }

        public static void Catch<TEx>(string id, string label, Action action) where TEx : Exception
        {
            Assertions++;
            Console.WriteLine("  CASE " + id + " " + label + " expected=" + typeof(TEx).Name);
            try
            {
                action();
                Console.WriteLine("    observed=no-exception MISMATCH");
                Failures++;
            }
            catch (Exception ex)
            {
                bool ok = ex.GetType() == typeof(TEx);
                Console.WriteLine("    observed=" + ex.GetType().Name + (ok ? " MATCH" : " MISMATCH"));
                if (ok)
                {
                    Passed++;
                }
                else
                {
                    Failures++;
                }
            }
        }

        public static void Fail(string id, string label, string observed)
        {
            Assertions++;
            Failures++;
            Console.WriteLine("  CASE " + id + " " + label + " expected=no-exception observed=" + observed + " MISMATCH");
        }

        private static void Count(bool ok)
        {
            if (ok)
            {
                Passed++;
            }
            else
            {
                Failures++;
            }
        }

        private static string SkillDesc(Skill? skill)
        {
            return skill == null ? "null" : "#" + skill.SkillId.ToString();
        }

        private static string SkillsDesc(IReadOnlyList<Skill> skills)
        {
            string[] parts = new string[skills.Count];
            for (int i = 0; i < skills.Count; i++)
            {
                parts[i] = SkillDesc(skills[i]);
            }
            return "[" + string.Join(",", parts) + "]";
        }

        private static string StringsDesc(IReadOnlyList<string> values)
        {
            string[] parts = new string[values.Count];
            for (int i = 0; i < values.Count; i++)
            {
                parts[i] = "\"" + values[i] + "\"";
            }
            return "[" + string.Join("|", parts) + "]";
        }

        private static string IntsDesc(IReadOnlyList<int> values)
        {
            string[] parts = new string[values.Count];
            for (int i = 0; i < values.Count; i++)
            {
                parts[i] = values[i].ToString();
            }
            return "[" + string.Join(",", parts) + "]";
        }
    }
}
