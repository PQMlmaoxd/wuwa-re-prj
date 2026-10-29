/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This source-only view is not original source and is not a
 * standalone-compilation claim. It preserves both selected bodies in full;
 * unselected members and configuration/state producers remain external or
 * UNKNOWN.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200309f (global::InputFunctionVisionSkill2; raw type row 96)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/facbcb03e0ffa08e158f3fe2cfabd9298861b833f20b704713cad97584e84840/source/_global/InputFunctionVisionSkill2.cs
 * Raw methods root SHA256: 04bcff97962a8eaa046ba5643ba528442bdaf242c2a2cb2d022f26962d13da0f
 * Raw types root SHA256: d44da5d7ae4fe15a7eacf0b25fd0eed83303cc9178944fa5d940344de8b5982e
 * Scope: two selected R24 whole bodies, rows 232-233; the selector retains
 * the data-table override, specialized tag branches, and generic fallback.
 * Character binding, configuration rows, Blueprint graph behavior, and
 * successful activation/transformation are UNKNOWN.
 */

using System.Collections.Generic;
using Aki.Protocol.Summon;
using AkiClient.Game.Aki.Character.Input.Enum;
using AkiClient.Game.Aki.Character.Input.Structures;
using AkiClient.Game.Aki.Character.Vision;
using CSharpScript.Game;

public static class InputFunctionVisionSkill2
{
	// Token 0x06019a51; original private static; canonical source lines 10-110;
	// raw methods row 282; RVA 7814284; 696 bytes; EH0.
	private static SInputCommand? VisionSkill2Function(float time)
	{
		TsBaseCharacter baseCharacter = Global.BaseCharacter;
		if (baseCharacter == null)
		{
			return null;
		}
		Entity entity = baseCharacter.CharacterActorComponent?.Entity;
		if (entity == null)
		{
			return null;
		}
		BaseTagComponent component = entity.GetComponent<BaseTagComponent>();
		if (component == null || !component.Valid)
		{
			return null;
		}
		if (!InputFunctionCommon.CanResponseInput(entity))
		{
			return null;
		}
		SInputCommand sInputCommand = InputFunctionCommon.CreateInputCommandFromDataTable(entity.Id, EInputAction.幻象2, EInputState.Press);
		if (sInputCommand != null)
		{
			return sInputCommand;
		}
		if (component.HasTag(GameplayTagDefine.EGameplayTagId["Rogue.Vision.Common.Demo测试_可变身操控"]))
		{
			if (InputFunctionCommon.HasEnoughEnergy(entity))
			{
				return InputFunctionCommon.CreateSkillCommand(entity, 200010);
			}
			return null;
		}
		if (component.HasTag(GameplayTagDefine.EGameplayTagId["幻象.声骸角色.团子.团子声骸已装备"]) && !InputFunctionCommon.HasEnoughEnergy(entity))
		{
			return null;
		}
		if (component.HasTag(GameplayTagDefine.EGameplayTagId["系统.肉鸽.声骸BD.无妄者.解锁领域词条"]))
		{
			if (component.HasTag(GameplayTagDefine.EGameplayTagId["系统.肉鸽.声骸BD.无妄者.解锁领域词条.激活二段Q"]))
			{
				return InputFunctionCommon.CreateSkillCommand(entity, 23000201);
			}
			return InputFunctionCommon.CreateSkillCommand(entity, 230002);
		}
		if (component.HasTag(GameplayTagDefine.EGameplayTagId["系统.肉鸽.3_4肉鸽.鳞人"]))
		{
			if (component.HasTag(GameplayTagDefine.EGameplayTagId["系统.肉鸽.3_4肉鸽.鳞人.允许变身"]))
			{
				return InputFunctionCommon.CreateSkillCommand(entity, 280001116);
			}
			return InputFunctionCommon.CreateSkillCommand(entity, 280001110);
		}
		CharacterVisionComponent component2 = entity.GetComponent<CharacterVisionComponent>();
		if (component2 == null)
		{
			return null;
		}
		int visionId = component2.GetVisionId();
		if (visionId != 0)
		{
			SVisionData visionData = PhantomUtil.GetVisionData(visionId);
			if (visionData == null)
			{
				return null;
			}
			EntityHandle summonedEntity = PhantomUtil.GetSummonedEntity(entity, ESummonType.ConcomitantVision);
			if (summonedEntity == null || !summonedEntity.Valid)
			{
				return null;
			}
			if (summonedEntity.Entity.Active && visionData.类型 != EVisionType.驻场)
			{
				return null;
			}
			bool 空中能否释放 = visionData.空中能否释放;
			if (!空中能否释放 && component.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.位置状态.空中"]))
			{
				return null;
			}
			int entityVisionSkillId = PhantomUtil.GetEntityVisionSkillId(entity.Id, visionId);
			if (entityVisionSkillId == 0)
			{
				return null;
			}
			int visionLevel = component2.GetVisionLevel();
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(entity.Id, "VisionLevel", visionLevel);
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(entity.Id, "VisionID", visionId);
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(entity.Id, "VisionAirSkill", 空中能否释放 ? 1 : 0);
			ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(entity.Id, "VisionLink", component.HasAnyTag((IEnumerable<int>)new int[4]
			{
				GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.普攻"],
				GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.技能"],
				GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.蓄力"],
				GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.核心"]
			}) ? 1 : 0);
			return InputFunctionCommon.CreateSkillCommand(entity, entityVisionSkillId);
		}
		return null;
	}

	// Token 0x06019a52; original public static; canonical source lines 112-115;
	// raw methods row 283; RVA 7814992; 7 bytes; EH0.
	public static SInputCommand? VisionSkill2OnPress(float time)
	{
		return VisionSkill2Function(time);
	}
}
