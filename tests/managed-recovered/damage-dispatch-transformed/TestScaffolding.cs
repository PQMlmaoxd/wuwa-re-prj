// Authored test scaffolding for RE-DAMAGE-DISPATCH02 (Phase 1 candidate).
// This file contains NO recovered method body. The exact canonical method
// slices live only in the private generated fixtures under
// github_src/tools/re/local-recovery/damage-dispatch02/.
//
// Authored here only:
//   * minimal enum adapters whose referenced members are observed in the
//     canonical bodies and raw operands, plus HealChange=35 / HealedChange=36
//     bound from the canonical CalculateHeal raw constants (row 650);
//   * the SnapshotPayload attacker/target shape and the 143-float attribute
//     host shape preserved from the prior accepted fixture;
//   * the Aki.Config.Damage and TDamageParam host shapes with read hooks;
//   * authored Log/Singleton/RandomSystem adapters and the controlled
//     Dictionary<int, TFormula> registration helper.
//
// No helper is stubbed, no game formula is recomputed, and no whole-class,
// whole-enum, whole-config or registry-initialization coverage is claimed.
using System;
using System.Collections.Generic;
using Aki.Protocol;

namespace Aki.Protocol
{
	// Minimal authored adapter: exactly the member names referenced by the
	// selected bodies plus no inferred member.
	//
	// TYPE AUTHORITY: the enum consumed by the selected bodies is
	// Aki.Protocol.EAttributeType of Aki.Generated.dll, 64864768 bytes, SHA-256
	// 19f70a3cbe8a306424b860f2174152fd7986dd0133f7c6268180e8752d302ea1,
	// MVID 4ce2a693-a46e-45c4-bfb3-7a8bf9085389 (R53 addendum line 52). The retained IL
	// excerpt observes TypeDef 0x0200935E and int32 value__, so the underlying
	// type is int, not byte.
	//
	// VALUE BINDING: members 0..20 come from the retained packet; 21..34, 37..43,
	// 95, 97, 99, 100..106 and 114 come from R38 raw CIL operand associations.
	// HealChange=35 and HealedChange=36 are bound by the canonical CalculateHeal
	// raw method row 650 constants: offset 22 ldc.i4.s 36 (target), offset 42
	// ldc.i4.s 35 (attacker). Max=143 is an ASSUMPTION ALIAS for the observed
	// guard operand; no retained literal named Max exists in the excerpt.
	//
	// The byte-backed AkiClient.Game.Aki.Protocol.EAttributeType shard type is a
	// DIFFERENT type. It is not the authority for these bodies, it is not used to
	// prove any value here, and no source-fidelity comparison is claimed.
	public enum EAttributeType : int
	{
		Lv = 1,
		CritDamage = 9,
		Def = 10,
		DamageChangeNormalSkill = 14,
		DamageChange = 15,
		DamageReduce = 16,
		DamageChangeAuto = 17,
		DamageChangeCast = 18,
		DamageChangeUltra = 19,
		DamageChangeQte = 20,
		DamageChangePhys = 21,
		DamageChangeElement1 = 22,
		DamageChangeElement2 = 23,
		DamageChangeElement3 = 24,
		DamageChangeElement4 = 25,
		DamageChangeElement5 = 26,
		DamageChangeElement6 = 27,
		DamageResistancePhys = 28,
		DamageResistanceElement1 = 29,
		DamageResistanceElement2 = 30,
		DamageResistanceElement3 = 31,
		DamageResistanceElement4 = 32,
		DamageResistanceElement5 = 33,
		DamageResistanceElement6 = 34,
		HealChange = 35,
		HealedChange = 36,
		DamageReducePhys = 37,
		DamageReduceElement1 = 38,
		DamageReduceElement2 = 39,
		DamageReduceElement3 = 40,
		DamageReduceElement4 = 41,
		DamageReduceElement5 = 42,
		DamageReduceElement6 = 43,
		SpecialDamageChange = 95,
		ElementPropertyType = 97,
		IgnoreDefRate = 99,
		IgnoreDamageResistancePhys = 100,
		IgnoreDamageResistanceElement1 = 101,
		IgnoreDamageResistanceElement2 = 102,
		IgnoreDamageResistanceElement3 = 103,
		IgnoreDamageResistanceElement4 = 104,
		IgnoreDamageResistanceElement5 = 105,
		IgnoreDamageResistanceElement6 = 106,
		DamageChangePhantom = 114,
		Max = 143
	}
}

namespace CSharpScript.Game.NewWorld.Character.Common.Component.Abilities
{
	// Shard view EElementType.cs declares the type with no namespace; the
	// selected CalculateHurt metadata signature fully qualifies it here. Member
	// order and values follow the shard declaration. MODEL_ASSUMPTION (adapter).
	public enum EElementType
	{
		Physical,
		Ice,
		Fire,
		Thunder,
		Wind,
		Light,
		Dark,
		ElementTypeMax
	}
}

// Shard view EAttrOwnerType.cs (global type): Attacker, Target. The selected
// selector body only proves "0 selects attacker, any nonzero value selects
// target"; no broader enum-validity claim is made.
public enum EAttrOwnerType
{
	Attacker = 0,
	Target = 1
}

// Shard view EAttackType.cs (global type): Auto, Cast, Ultra, QTE, NormalSkill,
// PhantomSkill. Values follow that declaration order and the R38 raw
// attack-attribute map (17/18/19/20/14/114) used by the copied switch body.
public enum EAttackType
{
	Auto,
	Cast,
	Ultra,
	QTE,
	NormalSkill,
	PhantomSkill
}

// Authored shape only: the two canonical SnapshotPayload members consumed by the
// selected bodies. Attacker is non-nullable so the copied Hurt local assignment
// is warning-free; Target is nullable because the copied body names it
// `AttributeSnapshot?` and dereferences it without a guard.
public class SnapshotPayload
{
	public AttributeSnapshot AttackerSnapshot { get; set; } = null!;

	public AttributeSnapshot? TargetSnapshot { get; set; }
}

// Authored TEST_DOUBLE host shape: two public readonly float[143] arrays, no
// setter, copy, or selection algorithm. The exact recovered GetCurrentValue body
// in the private fixture reads CurrentValues. The 143 length is preserved from
// the prior accepted fixture and remains an authored assumption.
public partial class AttributeSnapshot
{
	public readonly float[] BaseValues = new float[143];

	public readonly float[] CurrentValues = new float[143];
}

// Authored adapter for the Aki.Config.Damage shape referenced by the canonical
// CalculateFormula and CalculateHeal bodies. Property/method names, the long Id
// type and the int[] return shapes are observed from raw operands; no config
// table content is modeled.
namespace Aki.Config
{
	public sealed class Damage
	{
		public int FormulaType { get; set; }

		public int CalculateType { get; set; }

		public int RelatedProperty { get; set; }

		public int Type { get; set; }

		private long _id;

		public int IdReads;

		public long Id
		{
			get
			{
				IdReads++;
				global::Probe.Add("id");
				return _id;
			}
			set
			{
				_id = value;
			}
		}

		public int RateLvReads;
		public int[]? RateLvArray;

		public int[]? RateLv()
		{
			RateLvReads++;
			return RateLvArray;
		}

		public int CureBaseValueReads;
		public int[]? CureBaseValueArray;

		public int[]? CureBaseValue()
		{
			CureBaseValueReads++;
			return CureBaseValueArray;
		}

		public int FormulaParamReads;
		public int[]? FormulaParam1Array;
		public int[]? FormulaParam2Array;
		public int[]? FormulaParam3Array;
		public int[]? FormulaParam4Array;
		public int[]? FormulaParam5Array;
		public int[]? FormulaParam6Array;
		public int[]? FormulaParam7Array;
		public int[]? FormulaParam8Array;
		public int[]? FormulaParam9Array;
		public int[]? FormulaParam10Array;

		public int[]? FormulaParam1() { FormulaParamReads++; return FormulaParam1Array; }
		public int[]? FormulaParam2() { FormulaParamReads++; return FormulaParam2Array; }
		public int[]? FormulaParam3() { FormulaParamReads++; return FormulaParam3Array; }
		public int[]? FormulaParam4() { FormulaParamReads++; return FormulaParam4Array; }
		public int[]? FormulaParam5() { FormulaParamReads++; return FormulaParam5Array; }
		public int[]? FormulaParam6() { FormulaParamReads++; return FormulaParam6Array; }
		public int[]? FormulaParam7() { FormulaParamReads++; return FormulaParam7Array; }
		public int[]? FormulaParam8() { FormulaParamReads++; return FormulaParam8Array; }
		public int[]? FormulaParam9() { FormulaParamReads++; return FormulaParam9Array; }
		public int[]? FormulaParam10() { FormulaParamReads++; return FormulaParam10Array; }

		public int UpperReads;
		public int LowerReads;
		public int[]? FluctuationUpperArray;
		public int[]? FluctuationLowerArray;
		public bool UpperThrows;
		public global::TDamageParam? Owner;

		public int[]? FluctuationUpper()
		{
			UpperReads++;
			global::Probe.Add("upper(seed=" + SeedLabel() + ")");
			if (UpperThrows)
			{
				throw new InvalidOperationException("authored upper-bound hook");
			}
			return FluctuationUpperArray;
		}

		public int[]? FluctuationLower()
		{
			LowerReads++;
			global::Probe.Add("lower(seed=" + SeedLabel() + ")");
			return FluctuationLowerArray;
		}

		private string SeedLabel()
		{
			return Owner == null ? "no-owner" : Owner.PeekSeed().ToString();
		}
	}
}

// Authored host for the TDamageParam shape read by the canonical body. The seed
// property is the real mutation target of the canonical expression; the getter
// and setter are traced and the backing field can be inspected without a getter.
public sealed class TDamageParam
{
	public Aki.Config.Damage DamageData { get; set; } = new Aki.Config.Damage();

	public int SkillLevel { get; set; }

	public CSharpScript.Game.NewWorld.Character.Common.Component.Abilities.EElementType Element { get; set; }

	private int _seed;

	public int SeedGets;
	public int SeedSets;

	public int RandomSeed
	{
		get
		{
			SeedGets++;
			global::Probe.Add("seed.get=" + _seed.ToString());
			return _seed;
		}
		set
		{
			SeedSets++;
			global::Probe.Add("seed.set=" + value.ToString());
			_seed = value;
		}
	}

	public int PeekSeed()
	{
		return _seed;
	}

	public void SetSeedSilently(int value)
	{
		_seed = value;
	}

	private float _extraRate;

	public int ExtraRateReads;

	public float ExtraRate
	{
		get
		{
			ExtraRateReads++;
			global::Probe.Add("extraRate(seed=" + _seed.ToString() + ")");
			return _extraRate;
		}
		set
		{
			_extraRate = value;
		}
	}

	public float Accumulation { get; set; }
}

public enum ECalculationType
{
	Hurt = 0,
	Heal = 1,
	Cost = 2
}

// Source-only adapter: the raw operand 1 at ERandomReason argument position is
// observed in CalculateFormula; the member name is authored.
public enum ERandomReason
{
	Unknown = 0,
	DamageFluctuation = 1
}

public enum ELogModule
{
	Unspecified = 0
}

public enum ELogAuthor
{
	Unspecified = 0
}

// Authored provider adapter. There is no default queue and no RNG emulation:
// every test configures the exact returned seeds it needs. The real XorShift
// implementation is NOT compiled, NOT executed and NOT claimed equivalent.
public static class RandomSystem
{
	public static Queue<int>? NextSeeds;
	public static bool ThrowOnCall;
	public static int Calls;
	public static readonly List<(int Seed, int Reason)> Inputs = new List<(int, int)>();

	public static int GetNextRandomSeed(int seed, ERandomReason reason)
	{
		Calls++;
		Inputs.Add((seed, (int)reason));
		Probe.Add("provider(" + seed.ToString() + "," + ((int)reason).ToString() + ")");
		if (ThrowOnCall)
		{
			throw new InvalidOperationException("authored provider hook");
		}
		if (NextSeeds == null || NextSeeds.Count == 0)
		{
			throw new InvalidOperationException("authored provider queue is empty");
		}
		return NextSeeds.Dequeue();
	}

	public static void Reset()
	{
		NextSeeds = new Queue<int>();
		ThrowOnCall = false;
		Calls = 0;
		Inputs.Clear();
	}
}

// Authored logger adapter. It records call count, module, author, message and
// payload copies in call order; optional throwing is a controlled test switch.
// The signature follows the observed Log.Error(ELogModule, ELogAuthor, string,
// ReadOnlySpan<(string, object)>) operand.
public sealed class Log
{
	public static int Calls;
	public static bool ThrowOnError;
	public static ELogModule Module;
	public static ELogAuthor Author;
	public static string Message = "";
	public static readonly List<(string Key, object Value)> Payload = new List<(string, object)>();

	public void Error(ELogModule module, ELogAuthor author, string message, ReadOnlySpan<(string, object)> data)
	{
		Calls++;
		Module = module;
		Author = author;
		Message = message;
		Payload.Clear();
		for (int i = 0; i < data.Length; i++)
		{
			Payload.Add((data[i].Item1, data[i].Item2));
		}
		Probe.Add("log");
		if (ThrowOnError)
		{
			throw new InvalidOperationException("authored logger hook");
		}
	}

	public static void Reset()
	{
		Calls = 0;
		ThrowOnError = false;
		Module = default;
		Author = default;
		Message = "";
		Payload.Clear();
	}
}

public static class Singleton<T> where T : new()
{
	public static T Instance { get; } = new T();
}

public static class Probe
{
	public static readonly List<string> Trace = new List<string>();

	public static void Add(string entry)
	{
		Trace.Add(entry);
	}

	public static void Reset()
	{
		Trace.Clear();
	}

	public static string Joined()
	{
		return string.Join(";", Trace);
	}
}

// Authored registry host. The canonical source declares a private
// Dictionary<int, TFormula> and a Formulas getter whose runtime initialization
// (CreateStaticDefaultValue / StaticVariableRegister) is NOT executed here. This
// host installs a real BCL Dictionary<int, TFormula> so CalculateFormula's own
// TryGetValue expression is the code under test.
public partial class Calculation
{
	private static Dictionary<int, TFormula>? _formulas;

	private static Dictionary<int, TFormula> Formulas => _formulas!;

	public static void RegisterFormulaForTest(
		int formulaType,
		Func<SnapshotPayload, ECalculationType, TDamageParam, int, bool, double, double, double, double, double, double[], double> formula)
	{
		if (formula == null)
		{
			throw new ArgumentNullException(nameof(formula));
		}
		_formulas ??= new Dictionary<int, TFormula>();
		_formulas[formulaType] =
			(snap, calculateType, inputParam, level, isCritical, augment, amplify, accumulation, multiRatio, gameplayRatio, parameters) =>
				formula(snap, calculateType, inputParam, level, isCritical, augment, amplify, accumulation, multiRatio, gameplayRatio, parameters);
	}

	public static void ClearFormulasForTest()
	{
		_formulas = new Dictionary<int, TFormula>();
	}
}
