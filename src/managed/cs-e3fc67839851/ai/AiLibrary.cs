/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This partial source-only view is not original source and is not a
 * standalone-compilation claim. The original type is a public, non-static
 * class; `partial` is an overlay convenience, not observed metadata.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02000cea (global::AiLibrary; raw type row 8)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/5162edb5b734f611d2dbb0a01440bd0b33b4a9217e1ac4357443cf77098362cc/source/_global/AiLibrary.cs
 * Raw methods root SHA256: a79e64a26ef6140d8009c9e83e24723ea557bffc0d1f6767a65de4f6b8b472f2
 * Raw types root SHA256: 271f790da97347a3f363ba686a657058bbc6cbc69d962a24ab3a3d8e10d4361c
 * Scope: three selected ordinary whole method bodies; constructor and all
 * dependencies remain external. Generated InlineArray/PrivateImplementationDetails
 * calls in the canonical body are retained unchanged; this view does not
 * reconstruct their compiler-generated structure.
 */

using System;
using Aki.Config;
using CSharpScript.Core.Common;
using UnrealEngine;

public partial class AiLibrary
{
	// Token 0x060041cb; original public static non-virtual; canonical source lines 8-73;
	// raw methods row 51; RVA 453632; 789 bytes; EH0; optional debugLog=false.
	public static bool IsSkillAvailable(AiController aiController, int skillInfoId, CharacterSkillComponent skillComp, BaseTagComponent tagComp, int ignoreSkillType, double targetAngle, double height, double distance, double angle, bool checkDistanceAndAngle, bool debugLog = false)
	{
		if (!aiController.AiSkill.SkillInfos.TryGetValue(skillInfoId, out var value))
		{
			return false;
		}
		if (!aiController.AiSkill.SkillPreconditionMap.TryGetValue(value.SkillPreconditionId, out var value2))
		{
			return false;
		}
		if (!value2.NeedTarget)
		{
			return false;
		}
		bool flag = debugLog && !Singleton<Info>.Instance.IsBuildShipping;
		if (flag)
		{
			Log instance = Singleton<Log>.Instance;
			(string, object) reference = ("SkillInfoId", skillInfoId);
			instance.Info((ELogModule)94, (ELogAuthor)6, "Detect Skill", new ReadOnlySpan<(string, object)>(ref reference));
		}
		if (ignoreSkillType >= 0 && value.SkillType != ignoreSkillType)
		{
			if (flag)
			{
				Log instance2 = Singleton<Log>.Instance;
				(string, object) reference2 = ("Type", ignoreSkillType);
				instance2.Info((ELogModule)94, (ELogAuthor)6, "FailType", new ReadOnlySpan<(string, object)>(ref reference2));
			}
			return false;
		}
		int result;
		int skillId = (int.TryParse(value.SkillId, out result) ? result : 0);
		if (!aiController.AiSkill.CanActivate(skillInfoId) || !skillComp.IsCanUseSkill(skillId) || !aiController.AiSkill.CanActivate(skillInfoId))
		{
			if (flag)
			{
				Singleton<Log>.Instance.Info((ELogModule)94, (ELogAuthor)6, "FailCD", default(ReadOnlySpan<(string, object)>));
			}
			return false;
		}
		aiController.AiSkill.PreconditionTagMap.TryGetValue(value.SkillPreconditionId, out var _);
		if (value2.NeedTag.Length > 0 && (!tagComp.Valid || (aiController.AiSkill.PreconditionTagMap.TryGetValue(value.SkillPreconditionId, out var value4) && !tagComp.HasTag(value4.TagId()))))
		{
			if (flag)
			{
				Singleton<Log>.Instance.Info((ELogModule)94, (ELogAuthor)6, "FailTag", default(ReadOnlySpan<(string, object)>));
			}
			return false;
		}
		if (!value2.TargetAngleRange.HasValue || !value2.HeightRange.HasValue || !value2.DistanceRange.HasValue || !value2.AngleRange.HasValue || !Singleton<MathUtils>.Instance.InRangeAngle(targetAngle, value2.TargetAngleRange.Value) || !Singleton<MathUtils>.Instance.InRange(height, value2.HeightRange.Value) || (checkDistanceAndAngle && (!Singleton<MathUtils>.Instance.InRange(distance, value2.DistanceRange.Value) || !Singleton<MathUtils>.Instance.InRangeAngle(angle, value2.AngleRange.Value))))
		{
			if (flag)
			{
				Log instance3 = Singleton<Log>.Instance;
				global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("TargetAngle", targetAngle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("Distance", distance);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("Angle", angle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("Height", height);
				instance3.Info((ELogModule)94, (ELogAuthor)6, "FailLocation", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
			}
			return false;
		}
		return true;
	}

	// Token 0x060041cc; original public static non-virtual; canonical source lines 75-132;
	// raw methods row 52; RVA 454436; 637 bytes; finally EH try 228..506, handler 506..520;
	// optional debug=false.
	public static bool SelectSkillWithTarget(AiController aiController, CharacterSkillComponent skillComponent, CharacterActorComponent targetActorComp, int skillType, bool debug = false)
	{
		CharacterActorComponent charActorComp = aiController.CharActorComp;
		BaseTagComponent component = charActorComp.Entity.GetComponent<BaseTagComponent>();
		Vector vector = Vector.Create();
		Singleton<MathUtils>.Instance.InverseTransformPositionNoScale(targetActorComp.FloorLocation, targetActorComp.ActorRotationProxy, charActorComp.FloorLocation, vector);
		double angleByVector2D = Vector.GetAngleByVector2D(vector);
		Singleton<MathUtils>.Instance.InverseTransformPositionNoScale(charActorComp.FloorLocation, charActorComp.ActorRotationProxy, targetActorComp.FloorLocation, vector);
		double z = vector.Z;
		double distance = Math.Max(vector.Size2D() - (double)charActorComp.ScaledRadius - (double)targetActorComp.ScaledRadius, 1E-08);
		double angleByVector2D2 = Vector.GetAngleByVector2D(vector);
		int num = 0;
		int value = 0;
		int num2 = 0;
		if (debug)
		{
			Log instance = Singleton<Log>.Instance;
			(string, object) reference = ("Owner", aiController.CharActorComp.Actor.GetName());
			instance.Info((ELogModule)94, (ELogAuthor)6, "SelectSkillWithTarget", new ReadOnlySpan<(string, object)>(ref reference));
		}
		foreach (int item in aiController.AiSkill.ActiveSkillGroup)
		{
			Span<int> arrayIntBytes = aiController.AiSkill.BaseSkill.Value.RandomSkills()[item].GetArrayIntBytes();
			for (int i = 0; i < arrayIntBytes.Length; i++)
			{
				int num3 = arrayIntBytes[i];
				if (!aiController.AiSkill.SkillInfos.TryGetValue(num3, out var value2))
				{
					Log instance2 = Singleton<Log>.Instance;
					(string, object) reference2 = ("Id", num3);
					instance2.Error((ELogModule)27, (ELogAuthor)6, "没有配置技能库", new ReadOnlySpan<(string, object)>(ref reference2));
				}
				else if (value2.SkillWeight > 0 && IsSkillAvailable(aiController, num3, skillComponent, component, skillType, angleByVector2D, z, distance, angleByVector2D2, checkDistanceAndAngle: true, debug))
				{
					int skillWeight = value2.SkillWeight;
					num = unchecked(num + skillWeight);
					if (Singleton<MathUtils>.Instance.GetRandomRange(0.0, num) < (double)skillWeight)
					{
						value = num3;
						num2 = (int.TryParse(value2.SkillId, out var result) ? result : 0);
					}
				}
			}
		}
		if (debug)
		{
			Log instance3 = Singleton<Log>.Instance;
			(string, object) reference3 = ("SkillId", num2);
			instance3.Info((ELogModule)94, (ELogAuthor)6, "SelectSkillWithTarget Success", new ReadOnlySpan<(string, object)>(ref reference3));
		}
		if (num2 != 0)
		{
			ControllerBase<BlackboardController>.Instance.SetStringValueByEntity(charActorComp.Entity.Id, "SkillId", num2.ToString());
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(charActorComp.Entity.Id, "SkillInfoId", value);
			return true;
		}
		return false;
	}

	// Token 0x060041cd; original public static non-virtual; canonical source lines 134-187;
	// raw methods row 53; RVA 455116; 578 bytes; finally EH try 44..496, handler 496..510.
	public static bool SelectSkillWithoutTarget(AiController aiController, CharacterSkillComponent skillComponent, int skillType)
	{
		CharacterActorComponent charActorComp = aiController.CharActorComp;
		BaseTagComponent component = charActorComp.Entity.GetComponent<BaseTagComponent>();
		int num = 0;
		int value = 0;
		int num2 = 0;
		foreach (int item in aiController.AiSkill.ActiveSkillGroup)
		{
			int[] array = aiController.AiSkill.BaseSkill.Value.RandomSkills()[item].ArrayInt();
			foreach (int num3 in array)
			{
				AiSkillPrecondition value3;
				if (!aiController.AiSkill.SkillInfos.TryGetValue(num3, out var value2))
				{
					Log instance = Singleton<Log>.Instance;
					(string, object) reference = ("Id", num3);
					instance.Error((ELogModule)27, (ELogAuthor)6, "没有配置技能库", new ReadOnlySpan<(string, object)>(ref reference));
				}
				else if (!aiController.AiSkill.SkillPreconditionMap.TryGetValue(value2.SkillPreconditionId, out value3))
				{
					Log instance2 = Singleton<Log>.Instance;
					(string, object) reference2 = ("Id", value2.SkillPreconditionId);
					instance2.Error((ELogModule)27, (ELogAuthor)6, "没有配置技能前置条件", new ReadOnlySpan<(string, object)>(ref reference2));
				}
				else
				{
					if (value3.NeedTarget || (skillType >= 0 && value2.SkillType != skillType))
					{
						continue;
					}
					int result;
					int num4 = (int.TryParse(value2.SkillId, out result) ? result : 0);
					if (skillComponent.IsCanUseSkill(num4) && aiController.AiSkill.CanActivate(num3) && (value3.NeedTag.Length <= 0 || !aiController.AiSkill.PreconditionTagMap.TryGetValue(value2.SkillPreconditionId, out var value4) || value4.TagId() == 0 || (component.Valid && component.HasTag(value4.TagId()))))
					{
						int skillWeight = value2.SkillWeight;
						num = unchecked(num + skillWeight);
						if (Singleton<MathUtils>.Instance.GetRandomRange(0.0, num) < (double)skillWeight)
						{
							value = num3;
							num2 = num4;
						}
					}
				}
			}
		}
		if (num2 != 0)
		{
			ControllerBase<BlackboardController>.Instance.SetStringValueByEntity(charActorComp.Entity.Id, "SkillId", num2.ToString());
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(charActorComp.Entity.Id, "SkillInfoId", value);
			return true;
		}
		return false;
	}
}
