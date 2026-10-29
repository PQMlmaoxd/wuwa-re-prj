/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This source-only overlay contains exactly eight selected whole ordinary
 * method bodies from the original Calculation type. It is not original
 * source, a whole-class recovery, a standalone compilation unit, or runtime
 * verification. The partial declaration is an overlay convenience and was
 * not observed in the original metadata.
 *
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * Assembly MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002eb2 (Calculation; raw types.jsonl row 29)
 * Canonical source: src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/source/_global/Calculation.cs
 * Canonical source bytes/SHA256: 24688 / daa9fff61d83566475f474ab84e2feea1b1b7de987a08a678cc0c87a32ea6e34
 * Raw methods: src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/evidence/raw/methods.jsonl
 * Raw methods bytes/SHA256: 17291952 / e5a3faded0efc4e051c58e303bf5aa0e38f55a0faa62a22785089d3ed6ff31c8
 * Raw types: src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/evidence/raw/types.jsonl
 * Raw types bytes/SHA256: 476812 / c09e0d9d00f85597c8055f09c3a0e294d53e3ac92b8b9b349f228c63213a78a3
 * Selected MethodDefs: 0x06018867, 0x06018865, 0x06018868, 0x06018869,
 * 0x0601886a, 0x0601886b, 0x0601886c, 0x06018873 (rows 288-295).
 * Existing accepted view src/reconstructed/managed/semantic/gameplay/Calculation.cs
 * is referenced only; it remains unchanged and retains ToughCalculation
 * (0x0601886e).
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;

public partial class Calculation : IStaticVariableResetter
{
	// R38 row 289; token 0x06018865; private static; raw methods.jsonl:642.
	// Canonical source Calculation.cs:20-28; RVA 7210036; code 56 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 28fb4ef1a2962b49c17b987a90b3990d62cbda76729772c53ad80faabbaf5abf.
	// Coverage: SOURCE_RAW_ONLY; invalid attribute returns before snapshot access.
	private static double GetAttrFromSnapshots(SnapshotPayload snap, EAttrOwnerType attrOwnerType, EAttributeType attrId)
	{
		if (attrId < EAttributeType.Lv || attrId >= EAttributeType.Max)
		{
			return 0.0;
		}
		float num = ((attrOwnerType != 0) ? snap.TargetSnapshot.GetCurrentValue(attrId) : snap.AttackerSnapshot.GetCurrentValue(attrId));
		return num;
	}

	// R38 row 288; token 0x06018867; public static; raw methods.jsonl:644.
	// Canonical source Calculation.cs:30-56; RVA 7210112; code 550 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 104494b32b927284efc860883c49a233ff5ea3bc13efc3060873bb11873dd32d.
	// Coverage: PRIMARY_FULL_CIL, whole untruncated body; local num/output is the
	// downstream consumer of this managed hurt result within CalculateFormula.
	public static double CalculateHurt(SnapshotPayload snapshots, EElementType elementType, EAttackType attackType, EAttributeType attrId, double attrRate, bool isCritical, double augment, double amplify, double gameplayRatio, double formula = 0.0)
	{
		AttributeSnapshot attackerSnapshot = snapshots.AttackerSnapshot;
		AttributeSnapshot? targetSnapshot = snapshots.TargetSnapshot;
		double num = attrRate * 9.9999997473787516E-05;
		double elementDamageBonus = GetElementDamageBonus(attackerSnapshot, elementType);
		double num2 = Math.Min(GetElementDamageReduce(targetSnapshot, elementType), 1.0);
		double elementResistant = GetElementResistant(targetSnapshot, elementType);
		double elementIgnoreResistance = GetElementIgnoreResistance(attackerSnapshot, elementType);
		double num3 = ((elementResistant - elementIgnoreResistance <= 0.0) ? (1.0 - (elementResistant - elementIgnoreResistance) / 2.0) : ((!(elementResistant - elementIgnoreResistance < 0.8)) ? (1.0 / (1.0 + (elementResistant - elementIgnoreResistance) * 5.0)) : (1.0 - (elementResistant - elementIgnoreResistance))));
		double attackTypeDamageBonus = GetAttackTypeDamageBonus(attackerSnapshot, attackType);
		double attrFromSnapshots = GetAttrFromSnapshots(snapshots, EAttrOwnerType.Attacker, attrId);
		float currentValue = targetSnapshot.GetCurrentValue(EAttributeType.Def);
		float num4 = attackerSnapshot.GetCurrentValue(EAttributeType.IgnoreDefRate) * 0.0001f;
		float currentValue2 = attackerSnapshot.GetCurrentValue(EAttributeType.Lv);
		float num5 = Math.Min(2f, 1f / (currentValue * (1f - num4) / (800f + currentValue2 * 8f) + 1f));
		float num6 = attackerSnapshot.GetCurrentValue(EAttributeType.DamageChange) * 0.0001f;
		double num7 = (double)(1f + num6) + elementDamageBonus + attackTypeDamageBonus;
		EElementType targetElementType = (EElementType)targetSnapshot.GetCurrentValue(EAttributeType.ElementPropertyType);
		double num8 = Math.Min(targetSnapshot.GetCurrentValue(EAttributeType.DamageReduce) * 0.0001f, 1.0);
		float num9 = 1f + attackerSnapshot.GetCurrentValue(EAttributeType.SpecialDamageChange) * 0.0001f;
		double num10 = CalculateElementMatchUpRate(elementType, targetElementType);
		float num11 = attackerSnapshot.GetCurrentValue(EAttributeType.CritDamage) * 0.0001f;
		double num12 = (isCritical ? ((double)num11) : 1.0);
		double val = (num * attrFromSnapshots + augment + formula) * num12 * (double)num5 * num7 * num3 * (1.0 - num8) * (1.0 - num2) * (double)num9 * num10 * gameplayRatio * Math.Max(1.0 + amplify * 9.9999997473787516E-05, 0.0);
		return Math.Max(0.0, val);
	}

	// R38 row 290; token 0x06018868; public static; raw methods.jsonl:645.
	// Canonical source Calculation.cs:58-71; RVA 7210676; code 164 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 5adc8a15f877b893644a335c6deb32b2d2ddaa6aca2261b5bc40946b94decec1.
	// Coverage: SOURCE_RAW_ONLY; unknown element values return 0f before attr access.
	public static double GetElementResistant(AttributeSnapshot attrSet, EElementType elementType)
	{
		return elementType switch
		{
			EElementType.Physical => attrSet.GetCurrentValue(EAttributeType.DamageResistancePhys) * 0.0001f, 
			EElementType.Ice => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement1) * 0.0001f, 
			EElementType.Fire => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement2) * 0.0001f, 
			EElementType.Thunder => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement3) * 0.0001f, 
			EElementType.Wind => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement4) * 0.0001f, 
			EElementType.Light => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement5) * 0.0001f, 
			EElementType.Dark => attrSet.GetCurrentValue(EAttributeType.DamageResistanceElement6) * 0.0001f, 
			_ => 0f, 
		};
	}

	// R38 row 291; token 0x06018869; public static; raw methods.jsonl:646.
	// Canonical source Calculation.cs:73-86; RVA 7210852; code 164 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 9526a07e8439130494b0f9a4677acd1be1a6e5bc5f37ed8467e98915de03921e.
	// Coverage: SOURCE_RAW_ONLY; unknown element values return 0f before attr access.
	public static double GetElementIgnoreResistance(AttributeSnapshot attrSet, EElementType elementType)
	{
		return elementType switch
		{
			EElementType.Physical => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistancePhys) * 0.0001f, 
			EElementType.Ice => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement1) * 0.0001f, 
			EElementType.Fire => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement2) * 0.0001f, 
			EElementType.Thunder => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement3) * 0.0001f, 
			EElementType.Wind => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement4) * 0.0001f, 
			EElementType.Light => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement5) * 0.0001f, 
			EElementType.Dark => attrSet.GetCurrentValue(EAttributeType.IgnoreDamageResistanceElement6) * 0.0001f, 
			_ => 0f, 
		};
	}

	// R38 row 292; token 0x0601886a; public static; raw methods.jsonl:647.
	// Canonical source Calculation.cs:88-101; RVA 7211028; code 164 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 73952f0a2705e784b36a6e41d72b989284934cbcebd6674eb29f41108af392b7.
	// Coverage: SOURCE_RAW_ONLY; unknown element values return 0f before attr access.
	public static double GetElementDamageReduce(AttributeSnapshot attrSet, EElementType elementType)
	{
		return elementType switch
		{
			EElementType.Physical => attrSet.GetCurrentValue(EAttributeType.DamageReducePhys) * 0.0001f, 
			EElementType.Ice => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement1) * 0.0001f, 
			EElementType.Fire => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement2) * 0.0001f, 
			EElementType.Thunder => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement3) * 0.0001f, 
			EElementType.Wind => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement4) * 0.0001f, 
			EElementType.Light => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement5) * 0.0001f, 
			EElementType.Dark => attrSet.GetCurrentValue(EAttributeType.DamageReduceElement6) * 0.0001f, 
			_ => 0f, 
		};
	}

	// R38 row 293; token 0x0601886b; public static; raw methods.jsonl:648.
	// Canonical source Calculation.cs:103-116; RVA 7211204; code 164 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 434c4478a076c29dd3440bb483d0ae5457fcb7cbf49ce246523c7703916f34a8.
	// Coverage: SOURCE_RAW_ONLY; unknown element values return 0f before attr access.
	public static double GetElementDamageBonus(AttributeSnapshot attrSet, EElementType elementType)
	{
		return elementType switch
		{
			EElementType.Physical => attrSet.GetCurrentValue(EAttributeType.DamageChangePhys) * 0.0001f, 
			EElementType.Ice => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement1) * 0.0001f, 
			EElementType.Fire => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement2) * 0.0001f, 
			EElementType.Thunder => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement3) * 0.0001f, 
			EElementType.Wind => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement4) * 0.0001f, 
			EElementType.Light => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement5) * 0.0001f, 
			EElementType.Dark => attrSet.GetCurrentValue(EAttributeType.DamageChangeElement6) * 0.0001f, 
			_ => 0f, 
		};
	}

	// R38 row 294; token 0x0601886c; public static; raw methods.jsonl:649.
	// Canonical source Calculation.cs:118-130; RVA 7211380; code 143 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 baccd9b795266d6461d1521b821ef64413e8c1aab9e3bb3d8494172c12bad27d.
	// Coverage: SOURCE_RAW_ONLY; unknown attack values return 0f before attr access.
	public static double GetAttackTypeDamageBonus(AttributeSnapshot attrSet, EAttackType attackType)
	{
		return attackType switch
		{
			EAttackType.Auto => attrSet.GetCurrentValue(EAttributeType.DamageChangeAuto) * 0.0001f, 
			EAttackType.Cast => attrSet.GetCurrentValue(EAttributeType.DamageChangeCast) * 0.0001f, 
			EAttackType.Ultra => attrSet.GetCurrentValue(EAttributeType.DamageChangeUltra) * 0.0001f, 
			EAttackType.QTE => attrSet.GetCurrentValue(EAttributeType.DamageChangeQte) * 0.0001f, 
			EAttackType.NormalSkill => attrSet.GetCurrentValue(EAttributeType.DamageChangeNormalSkill) * 0.0001f, 
			EAttackType.PhantomSkill => attrSet.GetCurrentValue(EAttributeType.DamageChangePhantom) * 0.0001f, 
			_ => 0f, 
		};
	}

	// R38 row 295; token 0x06018873; public static; raw methods.jsonl:656.
	// Canonical source Calculation.cs:231-276; RVA 7213016; code 617 B; EH0;
	// generic0; non-CG/nonvirtual/nonabstract/nonfinal/nonaccessor/nonctor/
	// nonasync/noniterator; source-slice SHA256 529466cc1e43d45854c03bb8a024e4f682163064023ed2572464432043506571.
	// Coverage: PRIMARY_FULL_CIL, whole untruncated body; calculated local num is
	// consumed by fluctuation, ceiling, sign, and the returned result.
	public static double CalculateFormula(TDamageParam inputParam, SnapshotPayload snapshots, bool isCritical, double augment, double amplify, double multiRatio, double gameplayRatio)
	{
		Damage damageData = inputParam.DamageData;
		int skillLevel = inputParam.SkillLevel;
		int formulaType = damageData.FormulaType;
		ECalculationType calculateType = (ECalculationType)damageData.CalculateType;
		double num = 0.0;
		if (formulaType == 0)
		{
			EAttributeType relatedProperty = (EAttributeType)damageData.RelatedProperty;
			int levelValue = AbilityUtils.GetLevelValue(damageData.RateLv(), skillLevel, 0);
			if (calculateType == ECalculationType.Hurt)
			{
				num = CalculateHurt(snapshots, inputParam.Element, (EAttackType)damageData.Type, relatedProperty, levelValue, isCritical, augment, amplify, gameplayRatio);
			}
			else
			{
				int levelValue2 = AbilityUtils.GetLevelValue(damageData.CureBaseValue(), skillLevel, 0);
				num = CalculateHeal(snapshots, levelValue2, relatedProperty, levelValue);
			}
		}
		else
		{
			if (!Formulas.TryGetValue(formulaType, out TFormula value))
			{
				Log instance = Singleton<Log>.Instance;
				global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("damageId", damageData.Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("formula type", formulaType);
				instance.Error((ELogModule)35, (ELogAuthor)20, "unexpected formula type", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
				return 0.0;
			}
			num = value(snapshots, calculateType, inputParam, skillLevel, isCritical, augment, amplify, inputParam.Accumulation, multiRatio, gameplayRatio, AbilityUtils.GetLevelValue(damageData.FormulaParam1(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam2(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam3(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam4(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam5(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam6(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam7(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam8(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam9(), skillLevel, 0), AbilityUtils.GetLevelValue(damageData.FormulaParam10(), skillLevel, 0));
		}
		int randomSeed = inputParam.RandomSeed;
		inputParam.RandomSeed = RandomSystem.GetNextRandomSeed(randomSeed, ERandomReason.DamageFluctuation);
		int levelValue3 = AbilityUtils.GetLevelValue(damageData.FluctuationUpper(), skillLevel, 10000);
		int levelValue4 = AbilityUtils.GetLevelValue(damageData.FluctuationLower(), skillLevel, 10000);
		float num2 = (float)((levelValue3 - levelValue4) * (randomSeed % 10000) + levelValue4) * 0.0001f;
		num = Math.Ceiling(num * (double)inputParam.ExtraRate * (double)num2);
		if (calculateType == ECalculationType.Heal)
		{
			num = 0.0 - num;
		}
		return num;
	}
}
