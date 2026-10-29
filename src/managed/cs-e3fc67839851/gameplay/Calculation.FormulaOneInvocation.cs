/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This source-like overlay is bounded to the formula-registry construction
 * and the one selected generated callback associated with registry key 1. It
 * is not original source, a whole-class recovery, a standalone compilation
 * unit, or runtime verification.
 *
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * Assembly MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original Calculation TypeDef: 0x02002eb2 (raw types.jsonl row 29)
 * Original nested callback TypeDef: 0x02009315 (Calculation/<>c; raw types.jsonl row 149)
 *
 * The original type is `public class`, not partial. `partial` is introduced
 * only as a source-overlay convenience. `_formulas`, `TFormula`, all
 * compiler-generated cache fields, and all other Calculation members remain
 * external observed members; they are deliberately not declared here.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;

public partial class Calculation : IStaticVariableResetter
{
	// R39 proposed ordinary row 296; MethodDef 0x06018874; raw methods.jsonl row 657.
	// public static, nonvirtual, non-generic, non-CG, non-accessor, non-ctor,
	// non-async, non-iterator, EH0; RVA 7213648; code 319 B.
	// The eight cache reads, delegate constructions, Dictionary.Add calls, and
	// final _formulas assignment retain the observed key/order and assignment
	// boundary. The seven callback bodies remain external and are not inflated
	// into this ordinary row.
	public static void CreateStaticDefaultValue()
	{
		_formulas = new Dictionary<int, TFormula>
		{
			{
				1,
				<>c.<>9__19_0 ?? (<>c.<>9__19_0 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_0))
			},
			{
				2,
				<>c.<>9__19_1 ?? (<>c.<>9__19_1 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_1))
			},
			{
				3,
				<>c.<>9__19_2 ?? (<>c.<>9__19_2 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_2))
			},
			{
				4,
				<>c.<>9__19_3 ?? (<>c.<>9__19_3 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_3))
			},
			{
				5,
				<>c.<>9__19_4 ?? (<>c.<>9__19_4 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_4))
			},
			{
				6,
				<>c.<>9__19_5 ?? (<>c.<>9__19_5 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_5))
			},
			{
				7,
				<>c.<>9__19_6 ?? (<>c.<>9__19_6 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_6))
			},
			{
				1001,
				<>c.<>9__19_7 ?? (<>c.<>9__19_7 = new TFormula(<>c.<>9.<CreateStaticDefaultValue>b__19_7))
			}
		};
	}

	// The nested type is retained as the observed private sealed display class
	// identity; partial is an overlay convenience, not an original metadata flag.
	private sealed partial class <>c
	{
		// Separate generated association for MethodDef 0x06049f93; raw
		// methods.jsonl row 2292; not ordinary row 297 and not an additional
		// ordinary-body count. Internal instance, nonvirtual, generic arity 0,
		// compiler-generated/display-class, EH0; RVA 21011444; code 265 B.
		// Raw parameter metadata is retained exactly; `parameters` is not `params`.
		internal double <CreateStaticDefaultValue>b__19_0(
			SnapshotPayload snap,
			ECalculationType calculateType,
			TDamageParam damageParam,
			int level,
			bool isCritical,
			double augment,
			double amplify,
			double accumulation,
			double multiRatio,
			double gameplayRatio,
			double[] parameters)
		{
			double num11 = GetAttrFromSnapshots(snap, (EAttrOwnerType)parameters[3], (EAttributeType)parameters[2]) * (parameters[4] * 9.9999997473787516E-05) + parameters[5];
			double num12 = GetAttrFromSnapshots(snap, (EAttrOwnerType)parameters[8], (EAttributeType)parameters[7]) * (parameters[6] * 9.9999997473787516E-05) + parameters[9] + GetAttrFromSnapshots(snap, (EAttrOwnerType)parameters[1], (EAttributeType)parameters[0]) * num11;
			if (calculateType == ECalculationType.Heal)
			{
				float num13 = snap.TargetSnapshot.GetCurrentValue(EAttributeType.HealedChange) * 0.0001f;
				float num14 = snap.AttackerSnapshot.GetCurrentValue(EAttributeType.HealChange) * 0.0001f;
				return Math.Max(num12 * (double)Math.Max(0f, 1f + num13 + num14), 0.0);
			}
			return Math.Max(CalculateHurt(snap, damageParam.Element, (EAttackType)damageParam.DamageData.Type, (EAttributeType)damageParam.DamageData.RelatedProperty, AbilityUtils.GetLevelValue(damageParam.DamageData.RateLv(), level, 0), isCritical, augment, amplify, gameplayRatio, num12), 0.0);
		}
	}
}
