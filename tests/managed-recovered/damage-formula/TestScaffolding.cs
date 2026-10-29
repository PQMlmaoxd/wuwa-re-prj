// Authored test scaffolding for RE-GAMEPLAY-DAMAGE-FORMULA01 (preflight-02 candidate).
// This file contains NO recovered method body. The nine exact recovered bodies
// are generated into the private scratch fixture by run_damage_formula.py.
//
// Authored here only:
//   * minimal enum adapters bound to observed metadata/raw operands;
//   * the SnapshotPayload attacker/target shape;
//   * the 143-float attribute host shape (plain arrays, no take/copy logic);
//   * one public wrapper around the exact private recovered selector.
//
// No helper is stubbed, no damage formula is recomputed, and no whole-class or
// whole-enum compatibility is claimed. Adapters are MODEL_ASSUMPTION hosts.
using Aki.Protocol;

namespace Aki.Protocol
{
	// Minimal authored adapter: exactly the 43 member names referenced by the
	// nine selected bodies plus no inferred member.
	//
	// TYPE AUTHORITY (corrected in preflight-02): the enum consumed by the
	// selected bodies is `Aki.Protocol.EAttributeType` of Aki.Generated.dll,
	// 64864768 bytes, SHA-256
	// 19f70a3cbe8a306424b860f2174152fd7986dd0133f7c6268180e8752d302ea1,
	// MVID 4ce2a693-a46e-45bc-bfb3-7a8bf9085389 (R53 addendum line 52).
	// The retained IL excerpt observes TypeDef 0x0200935E,
	// `.class ... Aki.Protocol.EAttributeType`, and `int32 value__`
	// (R53_R46_RETAINED_EVIDENCE.txt lines 21-24, IL lines 13-16). The
	// underlying type is therefore `int`, not `byte`.
	//
	// The byte-backed type of the same short name at shard
	// 896b8405.../source/AkiClient/Game/Aki/Protocol/EAttributeType.cs
	// (`: byte`, `AkiClient.Game.Aki.Protocol`) is a DIFFERENT type. It is not
	// the authority for these bodies, it is not used to prove any value here,
	// and no source-fidelity comparison between the two is claimed.
	//
	// VALUE BINDING: the retained excerpt covers only the first 21 literals
	// (None=0 through DamageChangeQte=20, retained packet lines 17-76). Members
	// 21..143 below are adapter members whose numbers come from the R38 raw CIL
	// operand associations (addendum lines 186-219): bonus 21..27, resistance
	// 28..34, reduction 37..43, SpecialDamageChange 95, ElementPropertyType 97,
	// IgnoreDefRate 99, ignore-resistance 100..106, DamageChangePhantom 114.
	// That is a qualified numeric association, not a retained enum-table proof.
	//
	// `Max = 143` is an ASSUMPTION ALIAS: the guard operand 143 is observed, but
	// no retained literal named `Max` exists in the excerpt. The member exists
	// only because the copied body spells it.
	//
	// `DamageChangeQte` is the externally retained spelling and value 20
	// (retained packet line 76); the copied body matches it exactly.
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
	// Shard view EElementType.cs (520f9521...) declares the type with no
	// namespace; the selected CalculateHurt metadata signature (R38 row 288)
	// fully qualifies it in this namespace. The adapter follows the metadata
	// binding so the copied body compiles against one unambiguous type; member
	// order/values follow the shard declaration. MODEL_ASSUMPTION (adapter).
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

// Shard view EAttrOwnerType.cs (global type): Attacker, Target.
// The selected selector body only proves "0 selects attacker, any nonzero value
// selects target"; no broader enum-validity claim is made.
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
// is warning-free; Target is nullable because the copied Hurt body names it
// `AttributeSnapshot?` and dereferences it without a guard.
public class SnapshotPayload
{
	public AttributeSnapshot AttackerSnapshot { get; set; } = null!;

	public AttributeSnapshot? TargetSnapshot { get; set; }
}

// Authored TEST_DOUBLE host shape: two public readonly float[143] arrays, no
// setter, copy, or selection algorithm. The exact recovered GetCurrentValue body
// in the private fixture reads CurrentValues.
public partial class AttributeSnapshot
{
	public readonly float[] BaseValues = new float[143];

	public readonly float[] CurrentValues = new float[143];
}

// Authored adapter only. The declaration and exact body of GetAttrFromSnapshots
// remain unchanged in the private fixture; this wrapper exposes it because the
// canonical declaration is `private static`.
public partial class Calculation
{
	public static double GetAttrFromSnapshotsForTest(SnapshotPayload snap, EAttrOwnerType attrOwnerType, EAttributeType attrId)
	{
		return GetAttrFromSnapshots(snap, attrOwnerType, attrId);
	}
}
