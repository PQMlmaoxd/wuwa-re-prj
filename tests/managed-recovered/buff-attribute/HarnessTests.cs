// Authored self-runner for RE-BUFF-DIRECT-TEST01.
// Exercises exact recovered source slices generated into the private scratch
// fixture. No test framework, reflection, clock, process, file, network or
// thread API is used here.
using System;
using System.Collections.Generic;

internal static class Program
{
    private const string KindRecovered = "RECOVERED_SOURCE_WITH_STUBBED_DEPENDENCY";
    private const string KindDirect = "DIRECT_RECOVERED_SOURCE_TEST";

    private const EAttributeType A = EAttributeType.TEST_DOUBLE_101;
    private const EAttributeType B = EAttributeType.TEST_DOUBLE_102;
    private const EAttributeType C = EAttributeType.TEST_DOUBLE_103;
    private const EAttributeType INVALID = EAttributeType.TEST_DOUBLE_INVALID_200;

    private static int _passedTests;
    private static int _failedTests;

    private static readonly List<(string Id, string Kind, Action Body)> Tests = new List<(string, string, Action)>
    {
        ("B01", KindRecovered, B01_NonWritableSkips),
        ("B02", KindRecovered, B02_WritableClampZeroAndFloor),
        ("B03", KindRecovered, B03_NegativeExemptFromZeroClamp),
        ("B04", KindRecovered, B04_OptionalMaxAndLockerConflict),
        ("B05", KindRecovered, B05_UnchangedBaseSuppresses),
        ("B06", KindRecovered, B06_ChangedBaseUpdatesThenDispatches),
        ("B07", KindRecovered, B07_UnchangedCurrentDispatchSuppressed),
        ("B08", KindRecovered, B08_UpdateExceptionPersistsBaseMutation),
        ("B09", KindRecovered, B09_NotifyExceptionPersistsStorage),
        ("B10", KindRecovered, B10_ZeroAddNormalizesAndOldBaseDependency),
        ("B11", KindRecovered, B11_InvalidIndexSetVsAdd),
        ("B12", KindRecovered, B12_WritableInvalidThrowsAfterHooks),
        ("A01", KindRecovered, A01_Policy0AddValue),
        ("A02", KindRecovered, A02_Policy3OverrideValue),
        ("A03", KindRecovered, A03_Policy5AddTimeScaledValue),
        ("A04", KindRecovered, A04_Policy2SnapshotMinRatioFloorMax),
        ("A05", KindRecovered, A05_Policy4OverrideFromAttrIgnoresStack),
        ("A06", KindRecovered, A06_Policy9AddFromAttrRate),
        ("A07", KindRecovered, A07_StackZeroAdditive),
        ("A08", KindRecovered, A08_StackZeroOverride),
        ("A09", KindRecovered, A09_SnapshotZeroHonored),
        ("A10", KindRecovered, A10_NullSnapshotLiveRead),
        ("A11", KindRecovered, A11_MissingInstigatorGuardWithSnapshot),
        ("A12", KindRecovered, A12_MalformedPolicyArrays),
        ("A13", KindRecovered, A13_SequentialWriteSurvivesLaterThrow),
        ("A14", KindDirect, A14_GetLevelValueCases),
        ("A15", KindDirect, A15_GetAttrValueCases),
        ("A16", KindRecovered, A16_StateAttributeMissEarlyReturn)
    };

    private static int Main()
    {
        Console.WriteLine("RE-BUFF-DIRECT-TEST01 STAGE2 SELF-RUNNER");
        Console.WriteLine("TESTS_PLANNED=" + Tests.Count.ToString());

        foreach (var test in Tests)
        {
            RunTest(test.Id, test.Kind, test.Body);
        }

        int recovered = 0;
        int direct = 0;
        foreach (var test in Tests)
        {
            if (test.Kind == KindRecovered)
            {
                recovered++;
            }
            else if (test.Kind == KindDirect)
            {
                direct++;
            }
        }

        Console.WriteLine("TESTS_TOTAL=" + Tests.Count.ToString());
        Console.WriteLine("TESTS_PASSED=" + _passedTests.ToString());
        Console.WriteLine("TESTS_FAILED=" + _failedTests.ToString());
        Console.WriteLine("ASSERTIONS_TOTAL=" + T.Assertions.ToString());
        Console.WriteLine("RECOVERED_SOURCE_WITH_STUBBED_DEPENDENCY=" + recovered.ToString());
        Console.WriteLine("DIRECT_RECOVERED_SOURCE_TEST=" + direct.ToString());
        Console.WriteLine("SYNTHETIC_MODEL_TESTS=0");
        Console.WriteLine(_failedTests == 0 ? "RESULT=PASS" : "RESULT=FAIL");
        return _failedTests == 0 ? 0 : 1;
    }

    private static void RunTest(string id, string kind, Action body)
    {
        Console.WriteLine("TEST " + id + " [" + kind + "]");
        CharacterAttributeTypes.ResetForTest();
        CombatLog.Instance.Reset();
        int assertionsBefore = T.Assertions;
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

        bool ok = T.Failures == failuresBefore && T.Assertions > assertionsBefore;
        if (ok)
        {
            _passedTests++;
            Console.WriteLine("RESULT " + id + " PASS assertions=" + (T.Assertions - assertionsBefore).ToString());
        }
        else
        {
            _failedTests++;
            Console.WriteLine("RESULT " + id + " FAIL assertions=" + (T.Assertions - assertionsBefore).ToString());
        }
    }

    private static BaseAttributeComponent Owner(bool writable = true, bool mirror = true)
    {
        BaseAttributeComponent owner = new BaseAttributeComponent
        {
            WritablePredicate = _ => writable
        };
        if (mirror)
        {
            owner.UpdateObserver = (component, attrId) => component.SetRawCurrent(attrId, component.GetRawBase(attrId));
        }
        return owner;
    }

    private static void State(params EAttributeType[] attrIds)
    {
        foreach (EAttributeType attrId in attrIds)
        {
            CharacterAttributeTypes.stateAttributeIds.Add(attrId);
        }
    }

    private static TestBuffModifier Mod(EAttributeType attrId, float[]? value1, float[]? value2, int[] policy)
    {
        return new TestBuffModifier(attrId, value1, value2, policy);
    }

    private static TestBoundsLocker Upper(float percent, float offset)
    {
        return new TestBoundsLocker
        {
            LockUpperBounds = true,
            UpperPercent = percent,
            UpperOffset = offset
        };
    }

    private static TestBoundsLocker Lower(float percent, float offset)
    {
        return new TestBoundsLocker
        {
            LockLowerBounds = true,
            LowerPercent = percent,
            LowerOffset = offset
        };
    }

    private static void B01_NonWritableSkips()
    {
        BaseAttributeComponent owner = Owner(false, true);
        owner.SetRawBase(A, 4f);
        owner.SetRawCurrent(A, 4f);
        owner.SetBaseValue(A, 9f);
        T.Eq("B01 writable calls", 1, owner.WritableCalls);
        T.F("B01 base unchanged", 4f, owner.GetRawBase(A));
        T.Eq("B01 dispatch calls", 0, owner.DispatchCalls.Count);
    }

    private static void B02_WritableClampZeroAndFloor()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetBaseValue(A, -3.7f);
        T.F("B02 negative clamped to zero", 0f, owner.GetRawBase(A));
        owner.SetBaseValue(A, 4.9f);
        T.F("B02 floor positive", 4f, owner.GetRawBase(A));
        T.Eq("B02 update calls", 1, owner.UpdateCalls);
        T.Eq("B02 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void B03_NegativeExemptFromZeroClamp()
    {
        CharacterAttributeTypes.attrsNotClampZero = new[] { A };
        BaseAttributeComponent owner = Owner();
        owner.SetBaseValue(A, -3.7f);
        T.F("B03 negative preserved then floored", -4f, owner.GetRawBase(A));
        T.Eq("B03 update calls", 1, owner.UpdateCalls);
        T.Eq("B03 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void B04_OptionalMaxAndLockerConflict()
    {
        BaseAttributeComponent owner = Owner();
        CharacterAttributeTypes.attributeIdsWithMax[A] = B;
        owner.SetRawCurrent(B, 100f);

        // Independent max-only, upper-only, lower-only and conflicting bounds.
        owner.SetBaseValue(A, 120f);
        T.F("B04 max-only cap", 100f, owner.GetRawBase(A));

        owner.SuppliedLockers.Clear();
        owner.SuppliedLockers.Add(Upper(0.5f, 5f));
        owner.SetBaseValue(A, 120f);
        T.F("B04 upper-only cap", 55f, owner.GetRawBase(A));

        owner.SuppliedLockers.Clear();
        owner.SuppliedLockers.Add(Lower(0.75f, 0f));
        owner.SetBaseValue(A, 20f);
        T.F("B04 lower-only floor", 75f, owner.GetRawBase(A));

        owner.SuppliedLockers.Clear();
        owner.SuppliedLockers.Add(Upper(0.5f, 5f));
        owner.SuppliedLockers.Add(Lower(0.75f, 0f));
        owner.SetBaseValue(A, 120f);
        T.F("B04 lower wins over upper", 75f, owner.GetRawBase(A));

        T.Eq("B04 collector calls", 4, owner.CollectorCalls);
        T.Eq("B04 update calls", 3, owner.UpdateCalls);
        T.Eq("B04 dispatch calls", 3, owner.DispatchCalls.Count);
    }

    private static void B05_UnchangedBaseSuppresses()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 5f);
        owner.SetRawCurrent(A, 7f);
        owner.SetBaseValue(A, 5f);
        T.F("B05 base unchanged", 5f, owner.GetRawBase(A));
        T.Eq("B05 update calls", 0, owner.UpdateCalls);
        T.Eq("B05 dispatch calls", 0, owner.DispatchCalls.Count);
    }

    private static void B06_ChangedBaseUpdatesThenDispatches()
    {
        BaseAttributeComponent owner = Owner(true, false);
        owner.SetRawBase(A, 2f);
        owner.SetRawCurrent(A, 3f);
        owner.UpdateObserver = (component, attrId) => component.SetRawCurrent(attrId, component.GetRawBase(attrId) + 1f);
        owner.SetBaseValue(A, 7.5f);
        T.F("B06 base floored", 7f, owner.GetRawBase(A));
        T.F("B06 current updated", 8f, owner.GetRawCurrent(A));
        T.True("B06 update before dispatch", owner.Trace.Count == 2 && owner.Trace[0] == "update" && owner.Trace[1] == "dispatch");
        T.Eq("B06 dispatch calls", 1, owner.DispatchCalls.Count);
        T.F("B06 dispatch new", 8f, owner.DispatchCalls[0].NewValue);
        T.F("B06 dispatch old", 3f, owner.DispatchCalls[0].OldValue);
    }

    private static void B07_UnchangedCurrentDispatchSuppressed()
    {
        BaseAttributeComponent owner = Owner(true, false);
        owner.SetRawBase(A, 3f);
        owner.SetRawCurrent(A, 0f);
        owner.SetBaseValue(A, 7f);
        T.Eq("B07 base change with unchanged current suppresses implement", 0, owner.DispatchImplementCalls);
        owner.DispatchCurrentValueEvent(A, 5f, 5f);
        T.Eq("B07 exact old==new suppresses", 0, owner.DispatchImplementCalls);
        owner.DispatchCurrentValueEvent(A, 6f, 5f);
        T.Eq("B07 exact old!=new dispatches", 1, owner.DispatchImplementCalls);
    }

    private static void B08_UpdateExceptionPersistsBaseMutation()
    {
        BaseAttributeComponent owner = Owner(true, false);
        owner.SetRawBase(A, 2f);
        owner.SetRawCurrent(A, 3f);
        owner.UpdateObserver = (component, attrId) =>
        {
            component.SetRawCurrent(attrId, 8f);
            throw new InvalidOperationException("update-fail");
        };
        T.Throws<InvalidOperationException>("B08 update throws", () => owner.SetBaseValue(A, 7.5f));
        T.F("B08 base mutation persisted", 7f, owner.GetRawBase(A));
        T.F("B08 observer mutation persisted", 8f, owner.GetRawCurrent(A));
        T.Eq("B08 dispatch suppressed", 0, owner.DispatchCalls.Count);
    }

    private static void B09_NotifyExceptionPersistsStorage()
    {
        BaseAttributeComponent owner = Owner(true, false);
        owner.SetRawBase(A, 2f);
        owner.SetRawCurrent(A, 3f);
        owner.UpdateObserver = (component, attrId) => component.SetRawCurrent(attrId, 8f);
        owner.DispatchObserver = (attrId, newValue, oldValue) => throw new InvalidOperationException("notify-fail");
        T.Throws<InvalidOperationException>("B09 notify throws", () => owner.SetBaseValue(A, 7.5f));
        T.F("B09 base persisted", 7f, owner.GetRawBase(A));
        T.F("B09 current persisted", 8f, owner.GetRawCurrent(A));
        T.Eq("B09 dispatch attempted once", 1, owner.DispatchCalls.Count);
    }

    private static void B10_ZeroAddNormalizesAndOldBaseDependency()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, -5f);
        owner.SetRawCurrent(A, -5f);
        owner.AddBaseValue(A, 0f);
        T.F("B10 zero add normalized old base", 0f, owner.GetRawBase(A));
        owner.SetRawBase(A, 4f);
        owner.SetRawCurrent(A, 4f);
        owner.AddBaseValue(A, 2.9f);
        T.F("B10 add reads old base", 6f, owner.GetRawBase(A));
        T.Eq("B10 update calls", 2, owner.UpdateCalls);
        T.Eq("B10 dispatch calls", 2, owner.DispatchCalls.Count);
        T.F("B10 second dispatch old", 4f, owner.DispatchCalls[1].OldValue);
        T.F("B10 second dispatch new", 6f, owner.DispatchCalls[1].NewValue);
    }

    private static void B11_InvalidIndexSetVsAdd()
    {
        BaseAttributeComponent owner = Owner(false, true);
        owner.SetBaseValue(INVALID, 1f);
        T.Eq("B11 nonwritable set permission calls", 1, owner.WritableCalls);
        T.Eq("B11 nonwritable set no collector", 0, owner.CollectorCalls);
        T.Throws<IndexOutOfRangeException>("B11 add reads array first", () => owner.AddBaseValue(INVALID, 1f));
        T.Eq("B11 add throw precedes permission", 1, owner.WritableCalls);
    }

    private static void B12_WritableInvalidThrowsAfterHooks()
    {
        BaseAttributeComponent owner = Owner();
        CharacterAttributeTypes.attributeIdsWithMax[INVALID] = B;
        owner.SetRawCurrent(B, 100f);
        owner.SuppliedLockers.Add(Upper(0.5f, 0f));
        T.Throws<IndexOutOfRangeException>("B12 writable invalid throws", () => owner.SetBaseValue(INVALID, 50f));
        T.Eq("B12 permission ran", 1, owner.WritableCalls);
        T.Eq("B12 bounds ran", 1, owner.CollectorCalls);
        T.Eq("B12 update not reached", 0, owner.UpdateCalls);
    }

    private static void A01_Policy0AddValue()
    {
        BaseAttributeComponent owner = Owner();
        State(A);
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 2.5f }, Array.Empty<float>(), new[] { 0 }), 1, 1f, 2);
        T.F("A01 additive", 5f, owner.GetRawBase(A));
        T.Eq("A01 update calls", 1, owner.UpdateCalls);
        T.Eq("A01 dispatch calls", 1, owner.DispatchCalls.Count);
        T.Eq("A01 log calls", 0, CombatLog.Instance.Count);
    }

    private static void A02_Policy3OverrideValue()
    {
        BaseAttributeComponent owner = Owner();
        State(A);
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 7.9f }, Array.Empty<float>(), new[] { 3 }), 1, 1f, 5);
        T.F("A02 override floors", 7f, owner.GetRawBase(A));
        T.Eq("A02 update calls", 1, owner.UpdateCalls);
        T.Eq("A02 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A03_Policy5AddTimeScaledValue()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 1f);
        owner.SetRawCurrent(A, 1f);
        State(A);
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 4f }, Array.Empty<float>(), new[] { 5 }), 1, 0.5f, 3);
        T.F("A03 time-scaled additive", 7f, owner.GetRawBase(A));
        T.Eq("A03 update calls", 1, owner.UpdateCalls);
        T.Eq("A03 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A04_Policy2SnapshotMinRatioFloorMax()
    {
        BaseAttributeComponent owner = Owner();
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.AddFromAttr,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue,
            0,
            20,
            2,
            30,
            1
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 2, 100f);
        T.F("A04 min/ratio/floor/max/add/stack", 60f, owner.GetRawBase(A));
        T.Eq("A04 update calls", 1, owner.UpdateCalls);
        T.Eq("A04 dispatch calls", 1, owner.DispatchCalls.Count);
        T.Eq("A04 log calls", 0, CombatLog.Instance.Count);
    }

    private static void A05_Policy4OverrideFromAttrIgnoresStack()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 5f);
        owner.SetRawCurrent(A, 5f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.OverrideFromAttr,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 3, 10f);
        T.F("A05 override from snapshot", 10f, owner.GetRawBase(A));
        T.Eq("A05 update calls", 1, owner.UpdateCalls);
        T.Eq("A05 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A06_Policy9AddFromAttrRate()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 10f);
        owner.SetRawCurrent(A, 10f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.AddFromAttrRate,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 1, 10000f);
        T.F("A06 derived rate additive", 20f, owner.GetRawBase(A));
        T.Eq("A06 update calls", 1, owner.UpdateCalls);
        T.Eq("A06 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A07_StackZeroAdditive()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 3f);
        owner.SetRawCurrent(A, 3f);
        State(A);
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 5f }, Array.Empty<float>(), new[] { 0 }), 1, 1f, 0);
        T.F("A07 stack zero additive no change", 3f, owner.GetRawBase(A));
        T.Eq("A07 update calls", 0, owner.UpdateCalls);
        T.Eq("A07 dispatch calls", 0, owner.DispatchCalls.Count);
    }

    private static void A08_StackZeroOverride()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 5f);
        owner.SetRawCurrent(A, 5f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.OverrideFromAttr,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 0, 10f);
        T.F("A08 stack zero override still applies", 10f, owner.GetRawBase(A));
        T.Eq("A08 update calls", 1, owner.UpdateCalls);
        T.Eq("A08 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A09_SnapshotZeroHonored()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(B, 20f);
        owner.SetRawCurrent(B, 20f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.AddFromAttr,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 5f }, policy), 1, 1f, 1, 0f);
        T.F("A09 snapshot zero uses value2 only", 5f, owner.GetRawBase(A));
        T.Eq("A09 update calls", 1, owner.UpdateCalls);
        T.Eq("A09 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A10_NullSnapshotLiveRead()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(B, 20f);
        owner.SetRawCurrent(B, 20f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.AddFromAttr,
            (int)B,
            0,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 1, null);
        T.F("A10 null snapshot reads live base", 20f, owner.GetRawBase(A));
        T.Eq("A10 update calls", 1, owner.UpdateCalls);
        T.Eq("A10 dispatch calls", 1, owner.DispatchCalls.Count);
    }

    private static void A11_MissingInstigatorGuardWithSnapshot()
    {
        BaseAttributeComponent owner = Owner();
        owner.SetRawBase(A, 3f);
        owner.SetRawCurrent(A, 3f);
        State(A);
        int[] policy =
        {
            (int)ECalculationPolicyType.AddFromAttr,
            (int)B,
            1,
            (int)EAttributeBasedFloatCalculationType.BaseValue
        };
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 10000f }, new[] { 0f }, policy), 1, 1f, 1, 10f);
        T.F("A11 storage unchanged", 3f, owner.GetRawBase(A));
        T.Eq("A11 dispatch calls", 0, owner.DispatchCalls.Count);
        T.Eq("A11 log calls", 1, CombatLog.Instance.Count);
        T.Eq("A11 log module", CombatLog.EDebugModule.Buff, CombatLog.Instance.Last!.Module);
        T.Eq("A11 log attr key", "attrId", CombatLog.Instance.Last!.Pairs[0].Key);
        T.Eq("A11 log attr value", A, (EAttributeType)CombatLog.Instance.Last!.Pairs[0].Value);
    }

    private static void A12_MalformedPolicyArrays()
    {
        BaseAttributeComponent owner = Owner();
        State(A);

        T.Throws<NullReferenceException>(
            "A12 null policy",
            () => ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 1f }, new[] { 0f }, null!), 1, 1f, 1));
        T.F("A12 null policy storage unchanged", 0f, owner.GetRawBase(A));

        T.Throws<IndexOutOfRangeException>(
            "A12 empty policy",
            () => ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 1f }, new[] { 0f }, Array.Empty<int>()), 1, 1f, 1));
        T.F("A12 empty policy storage unchanged", 0f, owner.GetRawBase(A));

        T.Throws<IndexOutOfRangeException>(
            "A12 short policy",
            () => ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 1f }, new[] { 0f }, new[] { 2 }), 1, 1f, 1));
        T.F("A12 short policy storage unchanged", 0f, owner.GetRawBase(A));
    }

    private static void A13_SequentialWriteSurvivesLaterThrow()
    {
        BaseAttributeComponent owner = Owner();
        State(A);
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 5f }, Array.Empty<float>(), new[] { 0 }), 1, 1f, 1);
        T.Throws<NullReferenceException>(
            "A13 later throw",
            () => ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(A, new[] { 1f }, new[] { 0f }, null!), 1, 1f, 1));
        T.F("A13 first write survives", 5f, owner.GetRawBase(A));
        T.Eq("A13 first dispatch survives", 1, owner.DispatchCalls.Count);
        T.Eq("A13 first update survives", 1, owner.UpdateCalls);
    }

    private static void A14_GetLevelValueCases()
    {
        float[]? none = null;
        float[] empty = Array.Empty<float>();
        float[] values = { 10f, 20f, 30f };
        T.F("A14 null default", -1f, AbilityUtils.GetLevelValue(none, 1, -1f));
        T.F("A14 empty default", -2f, AbilityUtils.GetLevelValue(empty, 1, -2f));
        T.F("A14 level zero first", 10f, AbilityUtils.GetLevelValue(values, 0, -1f));
        T.F("A14 level one first", 10f, AbilityUtils.GetLevelValue(values, 1, -1f));
        T.F("A14 level two second", 20f, AbilityUtils.GetLevelValue(values, 2, -1f));
        T.F("A14 level three third", 30f, AbilityUtils.GetLevelValue(values, 3, -1f));
        T.F("A14 out of range last", 30f, AbilityUtils.GetLevelValue(values, 99, -1f));
        T.F("A14 negative last", 30f, AbilityUtils.GetLevelValue(values, -1, -1f));
    }

    private static void A15_GetAttrValueCases()
    {
        BaseAttributeComponent owner = Owner(true, false);
        owner.SetRawBase(B, 10f);
        owner.SetRawCurrent(B, 13f);
        IAttributeSet? none = null;
        T.F("A15 null set zero", 0f, AbilityUtils.GetAttrValue(none, B, EAttributeBasedFloatCalculationType.BaseValue));
        T.F("A15 base", 10f, AbilityUtils.GetAttrValue(owner, B, EAttributeBasedFloatCalculationType.BaseValue));
        T.F("A15 current", 13f, AbilityUtils.GetAttrValue(owner, B, EAttributeBasedFloatCalculationType.CurrentValue));
        T.F("A15 bonus", 3f, AbilityUtils.GetAttrValue(owner, B, EAttributeBasedFloatCalculationType.BonusValue));
        T.F("A15 default policy base", 10f, AbilityUtils.GetAttrValue(owner, B, (EAttributeBasedFloatCalculationType)99));
    }

    private static void A16_StateAttributeMissEarlyReturn()
    {
        BaseAttributeComponent owner = Owner();
        // C is not in stateAttributeIds. The null policy would throw if the
        // exact early-return branch did not run first.
        ActiveBuffInternal.ModifyStateAttribute(null, owner, Mod(C, new[] { 1f }, new[] { 0f }, null!), 1, 1f, 1);
        T.F("A16 non-state storage unchanged", 0f, owner.GetRawBase(C));
        T.Eq("A16 no permission access", 0, owner.WritableCalls);
        T.Eq("A16 no log", 0, CombatLog.Instance.Count);
    }
}

internal static class T
{
    public static int Assertions;
    public static int Failures;

    public static void F(string label, float expected, float actual)
    {
        Assertions++;
        if (expected != actual)
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + expected.ToString("R") + " observed=" + actual.ToString("R"));
        }
    }

    public static void Eq<TV>(string label, TV expected, TV actual)
    {
        Assertions++;
        if (!EqualityComparer<TV>.Default.Equals(expected, actual))
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=<" + expected + "> observed=<" + actual + ">");
        }
    }

    public static void True(string label, bool condition)
    {
        Assertions++;
        if (!condition)
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
        catch (TEx)
        {
        }
        catch (Exception ex)
        {
            Failures++;
            Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + typeof(TEx).Name + " observed=" + ex.GetType().Name);
        }
    }
}
