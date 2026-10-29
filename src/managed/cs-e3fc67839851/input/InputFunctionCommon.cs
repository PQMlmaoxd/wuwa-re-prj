/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This source-only partial view is not original source and is not a
 * standalone-compilation claim. It retains four selected R24 bodies; the
 * unselected TempSkill getter, type initializer, and reset methods remain
 * external and are not stubbed.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003093 (global::InputFunctionCommon; raw type row 90)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/62f6a6f02cbfd03bdd2802093285f964db8059f9b4fcf6b53e007efb8d92ad58/source/_global/InputFunctionCommon.cs
 * Raw methods root SHA256: 707147e2516e5bbc35921d0ee2ffd2fe1e7b04fdbd6619c7e9cbe1af163bd3a7
 * Raw types root SHA256: 9714f50e9fc215ce669d3977f12807680d6611733a030fc7b9ed9ec3bfc5dc45
 * The observed _tempSkill field 0x0400cc35 is declared without an invented
 * initializer. The TempSkill getter is an external unselected member.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Character.Input.Enum;
using AkiClient.Game.Aki.Character.Input.Structures;
using AkiClient.Game.Aki.Data.Fight.Struct;
using CSharpScript.Game;
using CSharpScript.Game.Input;
using UnrealEngine;

public partial class InputFunctionCommon : IStaticVariableResetter
{
	// 0x0400cc35; observed field; initializer belongs to an unselected type
	// initializer and is intentionally not reconstructed here.
	private static Skill? _tempSkill;

	// Token 0x06019a20; original public static; canonical source lines 23-43;
	// raw methods row 759; RVA 7808808; 127 bytes; EH0.
	public static SInputCommand? CreateSkillCommand(Entity entity, int skillId)
	{
		if (skillId == 0)
		{
			return null;
		}
		BaseSkillComponent component = entity.GetComponent<BaseSkillComponent>();
		if (component == null || !component.Valid)
		{
			return null;
		}
		SSkillInfo skillInfo = component.GetSkillInfo(skillId);
		int priority = component.GetPriority(skillId);
		int skillIdWithGroupId = component.GetSkillIdWithGroupId(1);
		int activePriority = component.GetActivePriority(skillIdWithGroupId);
		if (priority > activePriority || component.IsMainSkillReadyEnd || (priority == activePriority && component.SkillAcceptInput) || (skillInfo != null && skillInfo.GroupId != 1))
		{
			return new SInputCommand(ECommandType.Skill, skillId, default(FGameplayTag));
		}
		return null;
	}

	// Token 0x06019a21; original public static; canonical source lines 45-70;
	// raw methods row 760; RVA 7808948; 221 bytes; EH0.
	public static bool CanResponseInput(Entity entity)
	{
		BaseMoveComponent? component = entity.GetComponent<BaseMoveComponent>();
		if (component == null || !component.CanResponseInput())
		{
			return false;
		}
		BaseTagComponent component2 = entity.GetComponent<BaseTagComponent>();
		if (component2 != null && component2.HasAnyTag((IEnumerable<int>)new int[6]
		{
			GameplayTagDefine.EGameplayTagId["功能.功能制作.被击硬直时间"],
			GameplayTagDefine.EGameplayTagId["行为状态.位置状态.水中"],
			GameplayTagDefine.EGameplayTagId["行为状态.位置状态.攀爬"],
			GameplayTagDefine.EGameplayTagId["行为状态.动作状态.滑坡"],
			GameplayTagDefine.EGameplayTagId["行为状态.动作状态.受击.被抓取"],
			GameplayTagDefine.EGameplayTagId["角色.BaseRole.状态通用标识.输入硬直"]
		}))
		{
			return false;
		}
		if (component2 != null && component2.HasTag(GameplayTagDefine.EGameplayTagId["角色.BaseRole.技能通用标识.幻象变身中"]) && (component2 == null || !component2.HasTag(GameplayTagDefine.EGameplayTagId["角色.BaseRole.技能通用标识.幻象变身中.可输入"])))
		{
			return false;
		}
		return true;
	}

	// Token 0x06019a24; original public static; canonical source lines 101-150;
	// raw methods row 763; RVA 7809320; 293 bytes; EH0.
	public static SInputCommand? CreateInputCommandFromDataTable(int entityId, AkiClient.Game.Aki.Character.Input.Enum.EInputAction action, EInputState state)
	{
		Entity entity = Singleton<EntitySystem>.Instance.Get(entityId);
		if (entity == null)
		{
			return null;
		}
		BaseTagComponent component = Singleton<EntitySystem>.Instance.GetComponent<BaseTagComponent>(entityId);
		BaseSkillComponent component2 = Singleton<EntitySystem>.Instance.GetComponent<BaseSkillComponent>(entityId);
		if (component == null || component2 == null)
		{
			return null;
		}
		SInputCommandTransform[] inputCommandTransformData = ModelBase<InputModel>.Instance.GetInputCommandTransformData(action, state);
		if (inputCommandTransformData == null)
		{
			return null;
		}
		SInputCommandTransform[] array = inputCommandTransformData;
		foreach (SInputCommandTransform sInputCommandTransform in array)
		{
			if (!component.HasTag(sInputCommandTransform.Tag.TagId()))
			{
				continue;
			}
			bool flag = false;
			if (sInputCommandTransform.BehaviorConditionGroup.Num() == 0)
			{
				flag = true;
			}
			else if (SkillBehaviorCondition.SatisfyGroup(sInputCommandTransform.BehaviorConditionGroup, sInputCommandTransform.BehaviorConditionFormula, new BeginSkillBehaviorConditionParam
			{
				Entity = entity,
				SkillComponent = component2,
				Skill = TempSkill
			}))
			{
				flag = true;
			}
			if (flag)
			{
				if (sInputCommandTransform.Command.CommandType == ECommandType.Skill)
				{
					return CreateSkillCommand(entity, sInputCommandTransform.Command.IntValue);
				}
				return new SInputCommand(sInputCommandTransform.Command.CommandType, sInputCommandTransform.Command.IntValue, sInputCommandTransform.Command.TagValue);
			}
		}
		return null;
	}

	// Token 0x06019a25; original public static; canonical source lines 152-160;
	// raw methods row 764; RVA 7809628; 29 bytes; EH0.
	public static bool HasEnoughEnergy(Entity entity)
	{
		BaseAttributeComponent component = entity.GetComponent<BaseAttributeComponent>();
		if (component.GetCurrentValue(EAttributeType.SpecialEnergy4) >= component.GetCurrentValue(EAttributeType.SpecialEnergy4Max))
		{
			return true;
		}
		return false;
	}
}
