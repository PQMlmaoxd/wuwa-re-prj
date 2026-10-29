// Authored self-runner for RE-GAMEPLAY-DAMAGE-FORMULA01.
//
// Status of this file: PHASE 1 PREPARE ONLY. It is not compiled or executed
// until the primary resumes the task; the runner's --prepare-only mode only
// writes the private fixture, manifest, receipts and provenance.
//
// What this runner exercises: exactly the nine recovered bodies generated into
// the private scratch fixture -- CalculateHurt, GetAttrFromSnapshots,
// GetElementResistant, GetElementIgnoreResistance, GetElementDamageReduce,
// GetElementDamageBonus, GetAttackTypeDamageBonus,
// CalculateElementMatchUpRate and AttributeSnapshot.GetCurrentValue.
//
// No test framework, reflection, file, process, network, native, thread, clock
// or RNG API is used. Every input below is a synthetic authored vector; no game
// configuration, Buff value or extracted asset is read. No expected value is
// produced by calling the code under test: each one is derived from the saved
// source expression in README.md and asserted by IEEE-754 bit pattern.
using System;
using System.Collections.Generic;
using Aki.Protocol;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;

internal static class Program
{
	private static int _passedGroups;
	private static int _failedGroups;

	private static readonly Dictionary<string, int> GroupAssertions =
		new Dictionary<string, int>(StringComparer.Ordinal);

	private static readonly (string Sid, Action Body)[] Tests =
	{
		("D01", D01_RateScalingAndConditionalCritical),
		("D02", D02_DefenseFactor),
		("D03", D03_OffenseBonusesAreAdditive),
		("D04", D04_ResistanceRateBranches),
		("D05", D05_ReductionAmplifyGameplaySpecial),
		("D06", D06_AttributeSelectionAndNoEarlyReturn),
		("D07", D07_RawDoubleSemantics),
		("D08", D08_DeterminismAndReadDependencies)
	};

	private static int Main()
	{
		Console.WriteLine("RE-GAMEPLAY-DAMAGE-FORMULA01 SELF-RUNNER");
		Console.WriteLine("EXECUTION_PATH=DIRECT_CSC_ASSEMBLY");
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
		const int MinAssertions = 30;
		const int MaxAssertions = 60;
		bool shapeOk = Tests.Length == ExpectedGroups
			&& seen.Count == ExpectedGroups
			&& groupsWithAssertions == ExpectedGroups;
		bool budgetOk = T.Assertions >= MinAssertions && T.Assertions <= MaxAssertions;

		Console.WriteLine("GROUPS_WITH_ASSERTIONS=" + groupsWithAssertions.ToString());
		Console.WriteLine("GROUPS_PASSED=" + _passedGroups.ToString());
		Console.WriteLine("GROUPS_FAILED=" + _failedGroups.ToString());
		Console.WriteLine("ASSERTIONS_TOTAL=" + T.Assertions.ToString());
		Console.WriteLine("ASSERTIONS_PASSED=" + T.Passed.ToString());
		Console.WriteLine("ASSERTIONS_FAILED=" + T.Failures.ToString());
		Console.WriteLine("ASSERTION_BUDGET=" + MinAssertions.ToString() + ".." + MaxAssertions.ToString());
		Console.WriteLine("ASSERTION_BUDGET_OK=" + (budgetOk ? "true" : "false"));
		Console.WriteLine("SYNTHETIC_INPUTS_ONLY=true");
		Console.WriteLine("RECOMPUTED_FORMULA_ASSERTIONS=0");
		Console.WriteLine("ROUNDING_OR_SEED_ASSERTIONS=0");

		bool ok = shapeOk && budgetOk && _failedGroups == 0 && T.Failures == 0 && T.Assertions > 0;
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
			Console.WriteLine("RESULT " + sid + " PASS assertions=" + assertions.ToString());
		}
		else
		{
			_failedGroups++;
			Console.WriteLine("RESULT " + sid + " FAIL assertions=" + assertions.ToString()
				+ " passed=" + passed.ToString() + " failed=" + failed.ToString());
		}
	}

	private static SnapshotPayload Payload(AttributeSnapshot attacker, AttributeSnapshot target)
	{
		return new SnapshotPayload { AttackerSnapshot = attacker, TargetSnapshot = target };
	}

	private static AttributeSnapshot Snap(params (int Index, float Value)[] entries)
	{
		AttributeSnapshot snapshot = new AttributeSnapshot();
		foreach ((int index, float value) in entries)
		{
			snapshot.CurrentValues[index] = value;
		}
		return snapshot;
	}

	private static bool SameArray(float[] left, float[] right)
	{
		if (left.Length != right.Length)
		{
			return false;
		}
		for (int i = 0; i < left.Length; i++)
		{
			if (left[i] != right[i])
			{
				return false;
			}
		}
		return true;
	}

	// Neutral defaults: Ice element, Auto attack, related attribute 7 (= Atk,
	// externally retained at R53_R46_RETAINED_EVIDENCE.txt:42 and deliberately
	// outside the 43-name adapter set), neutral target, no amplifier,
	// gameplayRatio 1, formula 0.
	private static double Hurt(SnapshotPayload payload, double attrRate, bool isCritical, double augment,
		double amplify = 0.0, double gameplayRatio = 1.0,
		EElementType element = EElementType.Ice, EAttackType attackType = EAttackType.Auto,
		double formula = 0.0)
	{
		return Calculation.CalculateHurt(payload, element, attackType, (EAttributeType)7,
			attrRate, isCritical, augment, amplify, gameplayRatio, formula);
	}

	private static double HurtAttr(SnapshotPayload payload, EAttributeType attrId, double attrRate,
		bool isCritical, double augment)
	{
		return Calculation.CalculateHurt(payload, EElementType.Ice, EAttackType.Auto, attrId,
			attrRate, isCritical, augment, 0.0, 1.0, 0.0);
	}

	// D01: attrRate scaling through the exact binary constant, and the
	// unconditional read / conditional use of CritDamage.
	private static void D01_RateScalingAndConditionalCritical()
	{
		// K = 9.9999997473787516E-05 = (double)(float)1e-4 = 0x3F1A36E2E0000000.
		// 10000*K = 0.9999999747378752 (0x3FEFFFFFF2700000); * 128 is an exact
		// exponent shift with no rounding, so every other Hurt factor is 1.0 and
		// the expected result is 127.99999676644802 (0x405FFFFFF2700000).
		// Full derivation: README.md "D01 derivation".
		SnapshotPayload neutral = Payload(Snap((7, 128f)), Snap());
		T.Bits("D01a",
			"element=Ice attack=Auto attrRate=10000 attrId=7(Atk) attackerAtk=128 crit=false augment=0 amplify=0 gameplayRatio=1 target=neutral",
			0x405FFFFFF2700000UL,
			Hurt(neutral, 10000.0, false, 0.0));

		// CritDamage raw 5000 -> float 0.5 exactly; crit=true multiplies by 0.5
		// (exact power-of-two scaling, no rounding).
		SnapshotPayload half = Payload(Snap((7, 128f), (9, 5000f)), Snap());
		T.Bits("D01b",
			"attrRate=10000 attackerAtk=128 attackerCritDamage=5000 crit=true augment=0 target=neutral",
			0x404FFFFFF2700000UL,
			Hurt(half, 10000.0, true, 0.0));

		// Raw 0 reaches num12 as 0.0f; the product is zero, not a 1.0 fallback.
		T.Bits("D01c",
			"attrRate=10000 attackerAtk=128 attackerCritDamage=0 crit=true augment=0",
			0x0000000000000000UL,
			Hurt(Payload(Snap((7, 128f)), Snap()), 10000.0, true, 0.0));

		// CritDamage is read at line 52 for both critical settings but only used
		// by the line-53 selector. NaN with crit=false leaves the result exact.
		AttributeSnapshot nanCrit = Snap((9, float.NaN));
		T.Bits("D01d",
			"attrRate=0 attackerCritDamage=NaN crit=false augment=64 target=neutral",
			0x4050000000000000UL,
			Hurt(Payload(nanCrit, Snap()), 0.0, false, 64.0));

		// The same NaN is retained when the critical arm selects it.
		T.NaN("D01e",
			"attrRate=0 attackerCritDamage=NaN crit=true augment=64 target=neutral",
			Hurt(Payload(nanCrit, Snap()), 0.0, true, 64.0));
	}

	// D02: identified defense factor, upper cap only, and the final nonnegative
	// floor when the factor itself is negative.
	private static void D02_DefenseFactor()
	{
		// def=800, level=0, ignore=0: 800f/800f = 1f; +1f = 2f; 1f/2f = 0.5f.
		T.Bits("D02a",
			"attackerAtk=0 attrRate=0 augment=64 targetDef=800 attackerLv=0 attackerIgnoreDefRate=0",
			0x4040000000000000UL,
			Hurt(Payload(Snap(), Snap((10, 800f))), 0.0, false, 64.0));

		// IgnoreDefRate raw 10000 -> float 1.0f exactly -> def*(1-1) = 0 -> 1f.
		T.Bits("D02b",
			"augment=64 targetDef=800 attackerIgnoreDefRate=10000",
			0x4050000000000000UL,
			Hurt(Payload(Snap((99, 10000f)), Snap((10, 800f))), 0.0, false, 64.0));

		// Negative defense: -800f/800f = -1f; +1f = 0f; 1f/0f = float +Infinity;
		// Math.Min(2f, +Inf) = 2f. Upper cap only, no zero-division guard.
		T.Bits("D02c",
			"augment=64 targetDef=-800 attackerLv=0 attackerIgnoreDefRate=0",
			0x4060000000000000UL,
			Hurt(Payload(Snap(), Snap((10, -800f))), 0.0, false, 64.0));

		// -1600f/800f = -2f; +1f = -1f; 1f/-1f = -1f. The factor has no lower
		// clamp; only the returned Math.Max(0.0, val) floors it to +0.0.
		T.Bits("D02d",
			"augment=64 targetDef=-1600 attackerLv=0 attackerIgnoreDefRate=0",
			0x0000000000000000UL,
			Hurt(Payload(Snap(), Snap((10, -1600f))), 0.0, false, 64.0));
	}

	// D03: offense terms are additive, attacker-only, and the unknown attack
	// selector falls back to 0f.
	private static void D03_OffenseBonusesAreAdditive()
	{
		// num7 = (double)(1f + 5000*0.0001f) + 0.25 + 0.25 = 1.5 + 0.25 + 0.25.
		// The element/attack bonuses read AttackerSnapshot only; the target's
		// DamageChangeElement1=10000 decoy must not enter the product.
		SnapshotPayload bonuses = Payload(
			Snap((15, 5000f), (22, 2500f), (17, 2500f)),
			Snap((22, 10000f)));
		T.Bits("D03a",
			"attacker DamageChange=5000 DamageChangeElement1=2500 DamageChangeAuto=2500 targetDamageChangeElement1=10000 element=Ice attack=Auto augment=64",
			0x4060000000000000UL,
			Hurt(bonuses, 0.0, false, 64.0));

		// Attack selector 99 hits the switch default arm -> 0f, so 1.5 + 0.25.
		T.Bits("D03b",
			"same bonuses attack=(EAttackType)99 augment=64",
			0x405C000000000000UL,
			Hurt(bonuses, 0.0, false, 64.0, attackType: (EAttackType)99));
	}

	// D04: the three resistance arms and the ignore-resistance subtraction.
	private static void D04_ResistanceRateBranches()
	{
		// Ice reads resistance attribute 29 and ignore attribute 101.
		// raw -5000 -> -0.5f; R <= 0 -> 1 - R/2 = 1.25 -> 64*1.25 = 80.
		T.Bits("D04a",
			"targetResistanceElement1=-5000 attackerIgnoreResistanceElement1=0 augment=64 element=Ice",
			0x4054000000000000UL,
			Hurt(Payload(Snap(), Snap((29, -5000f))), 0.0, false, 64.0));

		// raw 5000 -> 0.5f; 0 < R < 0.8 -> 1 - R = 0.5 -> 32.
		T.Bits("D04b",
			"targetResistanceElement1=5000 attackerIgnoreResistanceElement1=0 augment=64 element=Ice",
			0x4040000000000000UL,
			Hurt(Payload(Snap(), Snap((29, 5000f))), 0.0, false, 64.0));

		// raw 10000 -> 1.0f; R >= 0.8 -> 1/(1+R*5) = 1/6 (single double rounding).
		// 64*(1/6) = 10.666666666666666 asserted by bits, not by a decimal
		// approximation. Raw 8000 is NOT a boundary case here: 8000*0.0001f
		// rounds to 0.7999999523162842f and stays in the middle arm.
		T.Bits("D04c",
			"targetResistanceElement1=10000 attackerIgnoreResistanceElement1=0 augment=64 element=Ice",
			0x4025555555555555UL,
			Hurt(Payload(Snap(), Snap((29, 10000f))), 0.0, false, 64.0));

		// Ignore raw 10000 -> 1.0f; R = -0.5 - 1.0 = -1.5 -> 1 - (-1.5)/2 = 1.75.
		T.Bits("D04d",
			"targetResistanceElement1=-5000 attackerIgnoreResistanceElement1=10000 augment=64",
			0x405C000000000000UL,
			Hurt(Payload(Snap((101, 10000f)), Snap((29, -5000f))), 0.0, false, 64.0));
	}

	// D05: mitigation caps/floors, the ordered amplify factor, gameplayRatio and
	// SpecialDamageChange as plain multiplicative terms.
	private static void D05_ReductionAmplifyGameplaySpecial()
	{
		// raw 20000 -> 2.0f; Math.Min(2.0f, 1.0) = 1.0 -> (1 - 1) = 0.
		T.Bits("D05a",
			"targetDamageReduce=20000 augment=64 target=neutral",
			0x0000000000000000UL,
			Hurt(Payload(Snap(), Snap((16, 20000f))), 0.0, false, 64.0));

		// No lower clamp: raw -5000 -> -0.5 -> (1 - (-0.5)) = 1.5 -> 96.
		T.Bits("D05b",
			"targetDamageReduce=-5000 augment=64 target=neutral",
			0x4058000000000000UL,
			Hurt(Payload(Snap(), Snap((16, -5000f))), 0.0, false, 64.0));

		// amplify raw 10000: 1.0 + 10000*K = 1.9999999747378752; 64*that.
		T.Bits("D05c",
			"augment=64 amplify=10000",
			0x405FFFFFF9380000UL,
			Hurt(Payload(Snap(), Snap()), 0.0, false, 64.0, amplify: 10000.0));

		// amplify raw -20000: 1.0 - 1.9999999494757503 < 0 -> Math.Max(...,0)=0.
		T.Bits("D05d",
			"augment=64 amplify=-20000",
			0x0000000000000000UL,
			Hurt(Payload(Snap(), Snap()), 0.0, false, 64.0, amplify: -20000.0));

		// gameplayRatio participates as an ordinary double factor.
		T.Bits("D05e",
			"augment=64 gameplayRatio=0.5",
			0x4040000000000000UL,
			Hurt(Payload(Snap(), Snap()), 0.0, false, 64.0, gameplayRatio: 0.5));

		// num9 = 1f + 5000*0.0001f = 1.5f (no clamp in Hurt) -> 96.
		T.Bits("D05f",
			"attackerSpecialDamageChange=5000 augment=64 target=neutral",
			0x4058000000000000UL,
			Hurt(Payload(Snap((95, 5000f)), Snap()), 0.0, false, 64.0));
	}

	// D06: attacker-attribute selection, target-defense selection, and proof
	// that an invalid related id returns 0 from the selector without turning
	// CalculateHurt into an early return.
	private static void D06_AttributeSelectionAndNoEarlyReturn()
	{
		// Owner is hard-coded Attacker at line 41; num = 10000*K, so the base
		// term is num*5000 with no other factor.
		T.Bits("D06a",
			"attrRate=10000 attrId=7(Atk) attackerAtk=5000 targetAtk=9999 augment=0",
			0x40B387FFF7B8DC00UL,
			HurtAttr(Payload(Snap((7, 5000f)), Snap((7, 9999f))),
				(EAttributeType)7, 10000.0, false, 0.0));

		// attrId 0 < Lv -> selector returns 0.0 before snapshot access, but the
		// augment and the defense factor still apply: 64*0.5 = 32.
		T.Bits("D06b",
			"attrId=0(below Lv) augment=64 targetDef=800",
			0x4040000000000000UL,
			HurtAttr(Payload(Snap(), Snap((10, 800f))), (EAttributeType)0, 0.0, false, 64.0));

		// attrId 143 = Max -> same guard, same still-applied product.
		T.Bits("D06c",
			"attrId=143(Max) augment=64 targetDef=800",
			0x4040000000000000UL,
			HurtAttr(Payload(Snap(), Snap((10, 800f))), (EAttributeType)143, 0.0, false, 64.0));

		// The guard precedes both property reads, so a null payload with an
		// out-of-range id returns 0.0 instead of throwing.
		T.DEq("D06d", "snap=null owner=Attacker attrId=0", 0.0,
			Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Attacker, (EAttributeType)0));
		T.DEq("D06e", "snap=null owner=Target attrId=255", 0.0,
			Calculation.GetAttrFromSnapshotsForTest(null!, EAttrOwnerType.Target, (EAttributeType)255));

		// Def is read from the TARGET snapshot (line 42). The attacker's 1600
		// would give 1f/(1600f/800f + 1f) = 1f/3f, not the target's exact 0.5f.
		T.Bits("D06f",
			"attackerDef=1600 targetDef=800 augment=64",
			0x4040000000000000UL,
			Hurt(Payload(Snap((10, 1600f)), Snap((10, 800f))), 0.0, false, 64.0));

		// The optional formula input is added beside augment before multipliers.
		T.Bits("D06g",
			"formula=32 augment=32 targetDef=800",
			0x4040000000000000UL,
			Calculation.CalculateHurt(Payload(Snap(), Snap((10, 800f))), EElementType.Ice,
				EAttackType.Auto, (EAttributeType)7, 0.0, false, 32.0, 0.0, 1.0, 32.0));
	}

	// D07: raw double semantics of the RETURNED amount -- no rounding, no
	// finite/nonnegative validation, no null shield. (The body does contain one
	// integer conversion: the line-48 float->EElementType cast for the inert
	// element value; it does not touch the returned amount.)
	private static void D07_RawDoubleSemantics()
	{
		// 0.25 survives exactly in the returned double: neither the result nor
		// any intermediate of the final product is Ceiling/Floor/int-converted.
		T.Bits("D07a",
			"augment=0.25 attrRate=0 target=neutral",
			0x3FD0000000000000UL,
			Hurt(Payload(Snap(), Snap()), 0.0, false, 0.25));

		// Negative result is replaced by +0.0 at the returned Math.Max(0.0, val).
		T.Bits("D07b",
			"augment=-1 attrRate=0 target=neutral",
			0x0000000000000000UL,
			Hurt(Payload(Snap(), Snap()), 0.0, false, -1.0));

		// Math.Max(0.0, NaN) is NaN; the value is not sanitized to 0.
		T.NaN("D07c",
			"gameplayRatio=NaN augment=64 target=neutral",
			Hurt(Payload(Snap(), Snap()), 0.0, false, 64.0, gameplayRatio: double.NaN));

		// Infinity propagates through the whole ordered product.
		T.Inf("D07d",
			"attrRate=+Infinity attrId=7(Atk) attackerAtk=128 augment=0 target=neutral",
			Hurt(Payload(Snap((7, 128f)), Snap()), double.PositiveInfinity, false, 0.0));

		// The nullable target is dereferenced with no immunity/shield branch; the
		// exact thrown type is required, so a wrapped/typed replacement fails.
		T.Throws<NullReferenceException>("D07e", "targetSnapshot=null element=Ice", () =>
		{
			Calculation.CalculateHurt(
				new SnapshotPayload { AttackerSnapshot = Snap(), TargetSnapshot = null },
				EElementType.Ice, EAttackType.Auto, (EAttributeType)7,
				0.0, false, 64.0, 0.0, 1.0, 0.0);
		});
	}

	// D08: repeated-call bit identity, input arrays untouched, and the actual
	// read dependency of each mutated slot.
	private static void D08_DeterminismAndReadDependencies()
	{
		AttributeSnapshot attacker = Snap((7, 128f));
		AttributeSnapshot target = Snap((10, 800f));
		SnapshotPayload payload = Payload(attacker, target);
		float[] attackerBefore = (float[])attacker.CurrentValues.Clone();
		float[] targetBefore = (float[])target.CurrentValues.Clone();

		double first = Hurt(payload, 0.0, false, 64.0);
		double second = Hurt(payload, 0.0, false, 64.0);

		T.Bits("D08a", "call 1 of 2 attrRate=0 augment=64 targetDef=800", 0x4040000000000000UL, first);
		T.True("D08b", "second call returns identical 64-bit pattern",
			BitConverter.DoubleToInt64Bits(first) == BitConverter.DoubleToInt64Bits(second));
		T.True("D08c", "attacker CurrentValues unchanged by the two calls",
			SameArray(attackerBefore, attacker.CurrentValues));
		T.True("D08d", "target CurrentValues unchanged by the two calls",
			SameArray(targetBefore, target.CurrentValues));

		// Index 97 is read at line 48 but CalculateElementMatchUpRate returns 1.0
		// in every arm, so this write must not change the result.
		target.CurrentValues[97] = 10000f;
		T.Bits("D08e", "mutate target[97] ElementPropertyType after the two calls",
			0x4040000000000000UL, Hurt(payload, 0.0, false, 64.0));

		// Index 10 is a real read dependency: 400/800 = 0.5f; +1f = 1.5f;
		// 1f/1.5f = 0.6666666865348816f; 64*that = 42.66666793823242.
		target.CurrentValues[10] = 400f;
		T.Bits("D08f", "mutate target[10] Def 800->400", 0x4045555560000000UL,
			Hurt(payload, 0.0, false, 64.0));
	}
}

internal static class T
{
	public static int Assertions;
	public static int Passed;
	public static int Failures;

	public static void Bits(string label, string vector, ulong expectedBits, double actual)
	{
		Assertions++;
		ulong actualBits = unchecked((ulong)BitConverter.DoubleToInt64Bits(actual));
		bool ok = actualBits == expectedBits;
		Console.WriteLine("  CASE " + label + " " + vector);
		Console.WriteLine("    expected=" + FromBits(expectedBits).ToString("R")
			+ " bits=0x" + expectedBits.ToString("X16")
			+ " observed=" + actual.ToString("R")
			+ " bits=0x" + actualBits.ToString("X16")
			+ (ok ? " MATCH" : " MISMATCH"));
		if (ok)
		{
			Passed++;
		}
		else
		{
			Failures++;
			Console.WriteLine("  ASSERT FAIL " + label + " bit pattern");
		}
	}

	public static void NaN(string label, string vector, double actual)
	{
		Assertions++;
		bool ok = double.IsNaN(actual);
		Console.WriteLine("  CASE " + label + " " + vector);
		Console.WriteLine("    expected=NaN observed=" + actual.ToString("R")
			+ " bits=0x" + unchecked((ulong)BitConverter.DoubleToInt64Bits(actual)).ToString("X16")
			+ (ok ? " MATCH" : " MISMATCH"));
		if (ok)
		{
			Passed++;
		}
		else
		{
			Failures++;
			Console.WriteLine("  ASSERT FAIL " + label + " expected NaN");
		}
	}

	public static void Inf(string label, string vector, double actual)
	{
		Assertions++;
		bool ok = double.IsPositiveInfinity(actual);
		Console.WriteLine("  CASE " + label + " " + vector);
		Console.WriteLine("    expected=+Infinity observed=" + actual.ToString("R")
			+ " bits=0x" + unchecked((ulong)BitConverter.DoubleToInt64Bits(actual)).ToString("X16")
			+ (ok ? " MATCH" : " MISMATCH"));
		if (ok)
		{
			Passed++;
		}
		else
		{
			Failures++;
			Console.WriteLine("  ASSERT FAIL " + label + " expected +Infinity");
		}
	}

	public static void DEq(string label, string vector, double expected, double actual)
	{
		Assertions++;
		bool ok = BitConverter.DoubleToInt64Bits(expected) == BitConverter.DoubleToInt64Bits(actual);
		Console.WriteLine("  CASE " + label + " " + vector);
		Console.WriteLine("    expected=" + expected.ToString("R")
			+ " bits=0x" + unchecked((ulong)BitConverter.DoubleToInt64Bits(expected)).ToString("X16")
			+ " observed=" + actual.ToString("R")
			+ " bits=0x" + unchecked((ulong)BitConverter.DoubleToInt64Bits(actual)).ToString("X16")
			+ (ok ? " MATCH" : " MISMATCH"));
		if (ok)
		{
			Passed++;
		}
		else
		{
			Failures++;
			Console.WriteLine("  ASSERT FAIL " + label + " expected 0.0");
		}
	}

	public static void True(string label, string vector, bool condition)
	{
		Assertions++;
		Console.WriteLine("  CASE " + label + " " + vector);
		Console.WriteLine("    expected=true observed=" + (condition ? "true" : "false")
			+ (condition ? " MATCH" : " MISMATCH"));
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

	public static void Throws<TEx>(string label, string vector, Action action) where TEx : Exception
	{
		Assertions++;
		Console.WriteLine("  CASE " + label + " " + vector);
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
				Console.WriteLine("    observed=" + ex.GetType().Name + " MATCH");
			}
			else
			{
				Failures++;
				Console.WriteLine("  ASSERT FAIL " + label + ": expected=" + typeof(TEx).Name
					+ " observed=" + ex.GetType().Name);
			}
		}
	}

	public static void Fail(string label)
	{
		Assertions++;
		Failures++;
		Console.WriteLine("  ASSERT FAIL " + label);
	}

	private static double FromBits(ulong bits)
	{
		return BitConverter.Int64BitsToDouble(unchecked((long)bits));
	}
}
