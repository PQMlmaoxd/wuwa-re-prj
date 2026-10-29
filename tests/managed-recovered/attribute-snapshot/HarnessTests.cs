// Authored self-runner for RE-SNAPSHOT-DIRECT-TEST01.
// Exercises exactly the three recovered source slices generated into the private
// scratch fixture. No test framework, reflection, file, process, network, native,
// background, thread or clock API is used here.
using System;
using System.Collections.Generic;
using Aki.Protocol;

internal static class Program
{
    private const int Count = 143;

    private static int _passedGroups;
    private static int _failedGroups;

    private static readonly Dictionary<string, int> GroupAssertions =
        new Dictionary<string, int>(StringComparer.Ordinal);

    private static readonly (string Sid, Action Body)[] Tests =
    {
        ("S01", S01_CurrentNotBaseAndNoAlias),
        ("S02", S02_AllCopiedAndBoundaries),
        ("S03", S03_MutateAfterCapture),
        ("S04", S04_IndependentCaptures),
        ("S05", S05_BaseCurrentSeparation),
        ("S06", S06_OwnerSelection),
        ("S07", S07_IndexAndNullBehavior),
        ("S08", S08_TwoCapturesThroughHelper)
    };

    private static int Main()
    {
        Console.WriteLine("RE-SNAPSHOT-DIRECT-TEST01 STAGE2 SELF-RUNNER");
        Console.WriteLine("TESTS_PLANNED=" + Tests.Length.ToString());
        Console.WriteLine("SIDS=" + Sids());

        HashSet<string> seen = new HashSet<string>(StringComparer.Ordinal);
        foreach ((string sid, Action body) in Tests)
        {
            if (!seen.Add(sid))
            {
                T.Fail("duplicate SID " + sid);
            }
            RunGroup(sid, body);
        }

        int groupsWithAssertions = 0;
        foreach ((string sid, Action _) in Tests)
        {
            if (GroupAssertions.TryGetValue(sid, out int assertions) && assertions > 0)
            {
                groupsWithAssertions++;
            }
        }

        const int ExpectedGroups = 8;
        bool shapeOk = Tests.Length == ExpectedGroups
            && seen.Count == ExpectedGroups
            && groupsWithAssertions == ExpectedGroups;

        Console.WriteLine("GROUPS_WITH_ASSERTIONS=" + groupsWithAssertions.ToString());
        Console.WriteLine("GROUPS_PASSED=" + _passedGroups.ToString());
        Console.WriteLine("GROUPS_FAILED=" + _failedGroups.ToString());
        Console.WriteLine("TESTS_TOTAL=" + Tests.Length.ToString());
        Console.WriteLine("TESTS_PASSED=" + _passedGroups.ToString());
        Console.WriteLine("TESTS_FAILED=" + _failedGroups.ToString());
        Console.WriteLine("ASSERTIONS_TOTAL=" + T.Assertions.ToString());
        Console.WriteLine("ASSERTIONS_PASSED=" + T.Passed.ToString());
        Console.WriteLine("ASSERTIONS_FAILED=" + T.Failures.ToString());
        Console.WriteLine("SYNTHETIC_MODEL_TESTS=0");

        bool ok = shapeOk
            && _failedGroups == 0
            && T.Failures == 0
            && T.Assertions > 0
            && T.Passed > 0;
        Console.WriteLine(ok ? "RESULT=PASS" : "RESULT=FAIL");
        return ok ? 0 : 1;
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

    private static void RunGroup(string sid, Action body)
    {
        Console.WriteLine("TEST " + sid);
        int assertionsBefore = T.Assertions;
        int passedBefore = T.Passed;
        int failuresBefore = T.Failures;
        try
        {
            body();
        }
        catch (Exception ex)
        {
            T.Failures++;
            Console.WriteLine("  UNEXPECTED EXCEPTION " + ex.GetType().Name + ": " + ex.Message);
        }

        int assertions = T.Assertions - assertionsBefore;
        int passed = T.Passed - passedBefore;
        int failed = T.Failures - failuresBefore;
        GroupAssertions[sid] = assertions;

        bool ok = assertions > 0 && failed == 0 && passed == assertions;
        if (ok)
        {
            _passedGroups++;
            Console.WriteLine("RESULT " + sid + " PASS assertions=" + assertions.ToString()
                + " passed=" + passed.ToString() + " failed=" + failed.ToString());
        }
        else
        {
            _failedGroups++;
            Console.WriteLine("RESULT " + sid + " FAIL assertions=" + assertions.ToString()
                + " passed=" + passed.ToString() + " failed=" + failed.ToString());
        }
    }

    private static void SeedAll(BaseAttributeComponent component)
    {
        for (int i = 0; i < Count; i++)
        {
            component.BaseValues[i] = 1000f + i;
            component.CurrentValues[i] = 1f + i * 0.25f;
        }
    }

    // S01: same index has distinct base/current values. The exact recovered
    // TakeSnapshot copies both arrays; the exact getter must return CurrentValues,
    // not BaseValues, and no array may alias the component arrays.
    private static void S01_CurrentNotBaseAndNoAlias()
    {
        BaseAttributeComponent component = new BaseAttributeComponent();
        component.BaseValues[9] = 100f;
        component.CurrentValues[9] = 7f;

        AttributeSnapshot snapshot = component.TakeSnapshot();

        T.F("S01 getter returns current not base", 7f, snapshot.GetCurrentValue((EAttributeType)9));
        T.F("S01 base copied", 100f, snapshot.BaseValues[9]);
        T.True("S01 component BaseValues distinct from snapshot", !object.ReferenceEquals(component.BaseValues, snapshot.BaseValues));
        T.True("S01 component CurrentValues distinct from snapshot", !object.ReferenceEquals(component.CurrentValues, snapshot.CurrentValues));
        T.True("S01 snapshot BaseValues distinct from snapshot CurrentValues", !object.ReferenceEquals(snapshot.BaseValues, snapshot.CurrentValues));
    }

    // S02: all 143 entries are copied for both arrays; the boundary entries 0 and
    // 142 and the requested indices 9/10/15 are checked. Inputs are authored
    // synthetic patterns, not game values.
    private static void S02_AllCopiedAndBoundaries()
    {
        BaseAttributeComponent component = new BaseAttributeComponent();
        SeedAll(component);

        AttributeSnapshot snapshot = component.TakeSnapshot();

        T.Eq("S02 base length", 143, snapshot.BaseValues.Length);
        T.Eq("S02 current length", 143, snapshot.CurrentValues.Length);
        T.F("S02 base boundary 0", 1000f, snapshot.BaseValues[0]);
        T.F("S02 base boundary 142", 1142f, snapshot.BaseValues[142]);
        T.F("S02 current boundary 0", 1f, snapshot.CurrentValues[0]);
        T.F("S02 current boundary 142", 36.5f, snapshot.CurrentValues[142]);

        for (int i = 0; i < Count; i++)
        {
            T.F("S02 base copied[" + i.ToString() + "]", 1000f + i, snapshot.BaseValues[i]);
            T.F("S02 current copied[" + i.ToString() + "]", 1f + i * 0.25f, snapshot.CurrentValues[i]);
        }

        T.True("S02 current 9/10/15 distinct",
            snapshot.CurrentValues[9] != snapshot.CurrentValues[10]
            && snapshot.CurrentValues[10] != snapshot.CurrentValues[15]
            && snapshot.CurrentValues[9] != snapshot.CurrentValues[15]);
        T.True("S02 current differs from base at 9/10/15",
            snapshot.CurrentValues[9] != snapshot.BaseValues[9]
            && snapshot.CurrentValues[10] != snapshot.BaseValues[10]
            && snapshot.CurrentValues[15] != snapshot.BaseValues[15]);
    }

    // S03: mutating component CurrentValues after capture must not change the old
    // snapshot, and the snapshot's current array must be a distinct array.
    private static void S03_MutateAfterCapture()
    {
        BaseAttributeComponent component = new BaseAttributeComponent();
        component.BaseValues[9] = 10f;
        component.CurrentValues[9] = 20f;
        AttributeSnapshot snapshot = component.TakeSnapshot();

        component.CurrentValues[9] = 99f;

        T.F("S03 old snapshot current unchanged", 20f, snapshot.GetCurrentValue((EAttributeType)9));
        T.F("S03 component current changed", 99f, component.CurrentValues[9]);
        T.F("S03 old snapshot base unchanged", 10f, snapshot.BaseValues[9]);
        T.True("S03 snapshot current array distinct from component", !object.ReferenceEquals(snapshot.CurrentValues, component.CurrentValues));
    }

    // S04: a second capture sees later component changes, while the first
    // snapshot and both snapshots' arrays remain independent.
    private static void S04_IndependentCaptures()
    {
        BaseAttributeComponent component = new BaseAttributeComponent();
        component.BaseValues[9] = 1f;
        component.CurrentValues[9] = 2f;
        AttributeSnapshot first = component.TakeSnapshot();

        component.CurrentValues[9] = 3f;
        AttributeSnapshot second = component.TakeSnapshot();

        T.F("S04 second sees changed current", 3f, second.GetCurrentValue((EAttributeType)9));
        T.F("S04 first remains original", 2f, first.GetCurrentValue((EAttributeType)9));
        T.True("S04 snapshot objects distinct", !object.ReferenceEquals(first, second));
        T.True("S04 base arrays distinct", !object.ReferenceEquals(first.BaseValues, second.BaseValues));
        T.True("S04 current arrays distinct", !object.ReferenceEquals(first.CurrentValues, second.CurrentValues));

        component.CurrentValues[9] = 4f;
        T.F("S04 first unaffected by later mutation", 2f, first.GetCurrentValue((EAttributeType)9));
        T.F("S04 second unaffected by later mutation", 3f, second.GetCurrentValue((EAttributeType)9));
    }

    // S05: base and current are copied separately. Mutating component base and
    // snapshot base separately must not affect the current getter or alias the
    // component arrays. No recovered base getter is used or authored.
    private static void S05_BaseCurrentSeparation()
    {
        BaseAttributeComponent component = new BaseAttributeComponent();
        component.BaseValues[9] = 111f;
        component.CurrentValues[9] = 222f;
        AttributeSnapshot snapshot = component.TakeSnapshot();

        snapshot.BaseValues[9] = 333f;
        component.BaseValues[9] = 444f;

        T.F("S05 getter reads current after base mutations", 222f, snapshot.GetCurrentValue((EAttributeType)9));
        T.F("S05 component current unchanged", 222f, component.CurrentValues[9]);
        T.F("S05 snapshot current unchanged", 222f, snapshot.CurrentValues[9]);
        T.F("S05 snapshot base mutated separately", 333f, snapshot.BaseValues[9]);
        T.F("S05 component base mutated separately", 444f, component.BaseValues[9]);
        T.True("S05 component base not aliased with snapshot base", !object.ReferenceEquals(component.BaseValues, snapshot.BaseValues));
        T.True("S05 component current not aliased with snapshot current", !object.ReferenceEquals(component.CurrentValues, snapshot.CurrentValues));
    }

    // S06: the exact private recovered selector is reached through the authored
    // public wrapper. Owner 0 selects attacker; every tested nonzero owner,
    // including negative and out-of-range enum casts, selects target.
    private static void S06_OwnerSelection()
    {
        AttributeSnapshot attacker = new AttributeSnapshot();
        attacker.CurrentValues[9] = 11f;
        attacker.CurrentValues[10] = 12f;
        attacker.CurrentValues[15] = 13f;

        AttributeSnapshot target = new AttributeSnapshot();
        target.CurrentValues[9] = 21f;
        target.CurrentValues[10] = 22f;
        target.CurrentValues[15] = 23f;

        SnapshotPayload payload = new SnapshotPayload
        {
            AttackerSnapshot = attacker,
            TargetSnapshot = target
        };

        T.Eq("S06 owner 0 selects attacker 9", 11.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Attacker, (EAttributeType)9));
        T.Eq("S06 owner 0 selects attacker 10", 12.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Attacker, (EAttributeType)10));
        T.Eq("S06 owner 0 selects attacker 15", 13.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Attacker, (EAttributeType)15));
        T.Eq("S06 Target selects target 9", 21.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Target, (EAttributeType)9));
        T.Eq("S06 Target selects target 10", 22.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Target, (EAttributeType)10));
        T.Eq("S06 Target selects target 15", 23.0, Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Target, (EAttributeType)15));
        T.Eq("S06 owner 2 selects target", 21.0, Calculation.GetAttrFromSnapshotsForTest(payload, (EAttrOwnerType)2, (EAttributeType)9));
        T.Eq("S06 owner -1 selects target", 22.0, Calculation.GetAttrFromSnapshotsForTest(payload, (EAttrOwnerType)(-1), (EAttributeType)10));
        T.Eq("S06 owner int.MinValue selects target", 23.0, Calculation.GetAttrFromSnapshotsForTest(payload, (EAttrOwnerType)int.MinValue, (EAttributeType)15));
        T.Eq("S06 owner 999 selects target", 21.0, Calculation.GetAttrFromSnapshotsForTest(payload, (EAttrOwnerType)999, (EAttributeType)9));
        T.True("S06 attacker/target sentinels differ at 9/10/15",
            attacker.CurrentValues[9] != target.CurrentValues[9]
            && attacker.CurrentValues[10] != target.CurrentValues[10]
            && attacker.CurrentValues[15] != target.CurrentValues[15]);
    }

    // S07: raw getter bounds and exact helper guard/null behavior. The tiny
    // assertion helper accepts only the exact exception type, so a derived or
    // different exception cannot be counted as a pass.
    private static void S07_IndexAndNullBehavior()
    {
        AttributeSnapshot snapshot = new AttributeSnapshot();
        snapshot.CurrentValues[0] = 100f;
        snapshot.CurrentValues[1] = 101f;
        snapshot.CurrentValues[142] = 1420f;

        T.Throws<IndexOutOfRangeException>("S07 raw getter -1", () => { snapshot.GetCurrentValue((EAttributeType)(-1)); });
        T.Throws<IndexOutOfRangeException>("S07 raw getter 143", () => { snapshot.GetCurrentValue((EAttributeType)143); });
        T.F("S07 raw getter 0 accepted", 100f, snapshot.GetCurrentValue((EAttributeType)0));
        T.F("S07 raw getter 142 accepted", 1420f, snapshot.GetCurrentValue((EAttributeType)142));

        T.Eq("S07 helper -1 returns 0 before dereference", 0.0, Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Attacker, (EAttributeType)(-1)));
        T.Eq("S07 helper 0 returns 0 before dereference", 0.0, Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Attacker, (EAttributeType)0));
        T.Eq("S07 helper 143 returns 0 before dereference", 0.0, Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Attacker, (EAttributeType)143));
        T.Eq("S07 helper 144 returns 0 before dereference", 0.0, Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Attacker, (EAttributeType)144));

        T.Eq("S07 helper valid 1 attacker", 101.0,
            Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = snapshot, TargetSnapshot = null },
                EAttrOwnerType.Attacker,
                (EAttributeType)1));
        T.Eq("S07 helper valid 142 target", 1420.0,
            Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = null, TargetSnapshot = snapshot },
                EAttrOwnerType.Target,
                (EAttributeType)142));

        T.Throws<NullReferenceException>("S07 valid null attacker dereference",
            () => { Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = null, TargetSnapshot = snapshot },
                EAttrOwnerType.Attacker,
                (EAttributeType)1); });
        T.Throws<NullReferenceException>("S07 valid null target dereference",
            () => { Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = snapshot, TargetSnapshot = null },
                EAttrOwnerType.Target,
                (EAttributeType)1); });

        T.Eq("S07 unused null target does not fail", 101.0,
            Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = snapshot, TargetSnapshot = null },
                EAttrOwnerType.Attacker,
                (EAttributeType)1));
        T.Eq("S07 unused null attacker does not fail", 1420.0,
            Calculation.GetAttrFromSnapshotsForTest(
                new SnapshotPayload { AttackerSnapshot = null, TargetSnapshot = snapshot },
                EAttrOwnerType.Target,
                (EAttributeType)142));
    }

    // S08: two real TakeSnapshot results from distinct controlled components feed
    // the exact selector at indices 9/10/15 for both owner branches. The expected
    // value is the exact float getter promoted to double; the full damage formula
    // is not run and no full damage result is claimed.
    private static void S08_TwoCapturesThroughHelper()
    {
        BaseAttributeComponent firstComponent = new BaseAttributeComponent();
        BaseAttributeComponent secondComponent = new BaseAttributeComponent();
        firstComponent.CurrentValues[9] = 9.5f;
        firstComponent.CurrentValues[10] = 10.5f;
        firstComponent.CurrentValues[15] = 15.5f;
        secondComponent.CurrentValues[9] = 19.5f;
        secondComponent.CurrentValues[10] = 20.5f;
        secondComponent.CurrentValues[15] = 25.5f;

        AttributeSnapshot first = firstComponent.TakeSnapshot();
        AttributeSnapshot second = secondComponent.TakeSnapshot();

        SnapshotPayload payload = new SnapshotPayload
        {
            AttackerSnapshot = first,
            TargetSnapshot = second
        };

        T.True("S08 capture objects distinct", !object.ReferenceEquals(first, second));
        T.True("S08 current arrays distinct", !object.ReferenceEquals(first.CurrentValues, second.CurrentValues));
        T.True("S08 captured values differ at 9/10/15",
            first.CurrentValues[9] != second.CurrentValues[9]
            && first.CurrentValues[10] != second.CurrentValues[10]
            && first.CurrentValues[15] != second.CurrentValues[15]);

        int[] indices = { 9, 10, 15 };
        foreach (int index in indices)
        {
            EAttributeType attrId = (EAttributeType)index;
            T.Eq("S08 attacker helper[" + index.ToString() + "] equals getter",
                (double)first.GetCurrentValue(attrId),
                Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Attacker, attrId));
            T.Eq("S08 target helper[" + index.ToString() + "] equals getter",
                (double)second.GetCurrentValue(attrId),
                Calculation.GetAttrFromSnapshotsForTest(payload, EAttrOwnerType.Target, attrId));
        }
    }
}

internal static class T
{
    public static int Assertions;
    public static int Passed;
    public static int Failures;

    public static void F(string label, float expected, float actual)
    {
        Assertions++;
        if (expected == actual)
        {
            Passed++;
        }
        else
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + expected.ToString("R") + " observed=" + actual.ToString("R"));
        }
    }

    public static void Eq<TV>(string label, TV expected, TV actual)
    {
        Assertions++;
        if (EqualityComparer<TV>.Default.Equals(expected, actual))
        {
            Passed++;
        }
        else
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=<" + expected + "> observed=<" + actual + ">");
        }
    }

    public static void True(string label, bool condition)
    {
        Assertions++;
        if (condition)
        {
            Passed++;
        }
        else
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=true observed=false");
        }
    }

    public static void Throws<TEx>(string label, Action action) where TEx : Exception
    {
        Assertions++;
        try
        {
            action();
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + typeof(TEx).Name + " observed=no-exception");
        }
        catch (Exception ex)
        {
            if (ex.GetType() == typeof(TEx))
            {
                Passed++;
            }
            else
            {
                Failures++;
                Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + typeof(TEx).Name + " observed=" + ex.GetType().Name);
            }
        }
    }

    public static void Fail(string label)
    {
        Assertions++;
        Failures++;
        Console.WriteLine("  ASSERT FAIL " + label);
    }
}
