/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * The original type was not partial; this partial declaration is a source-only
 * overlay and is not a standalone compilation unit.
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e65; MethodDef: 0x06018655.
 * Canonical body: src/reconstructed/managed/full/CSharpScript/shards/03be93c6e9f3b6cc4f774490240d058232d76eebe5eefff276bffc0da2c82d16/source/_global/BaseDamageComponent.cs:683-700.
 * Original CIL: R41_EXECDAMAGEEVENT.il; RVA 0x6d4094; 216 bytes; 91 instructions; EH0.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using Aki.Protocol.Summon;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;
using CSharpScript.Game.World.Model;
using CSharpScript.Utils;
using Google.Protobuf;
using UnrealEngine;

public partial class BaseDamageComponent : EntityComponent
{
	private void ExecDamageEvent(DamageResult damageResult, DamageEventInfo inputParam, RequirementPayload requirements)
	{
		Entity entity = inputParam.Attacker.Entity;
		Entity entity2 = base.Entity;
		if (damageResult.DamageData.CalculateType == 1)
		{
			Singleton<EventSystem>.Instance.Emit(EEventName.FormationPanelUIShowRoleHeal, entity2);
		}
		ControllerBase<SceneTeamController>.Instance.EmitEvent(entity2, EEventName.CharBeDamage, entity, entity2, requirements, damageResult, inputParam.HitPosition);
		ControllerBase<SceneTeamController>.Instance.EmitEvent(entity, EEventName.CharDamage, entity, entity2, requirements, damageResult, inputParam.HitPosition);
		if (damageResult.IsTargetKilled)
		{
			Entity entity3 = ((damageResult.KillerEntityId == 0L) ? null : ModelBase<CreatureModel>.Instance.GetEntity(damageResult.KillerEntityId)?.Entity) ?? entity;
			ControllerBase<SceneTeamController>.Instance.EmitEvent(entity3, EEventName.CharKillTarget, entity3, entity2, requirements, damageResult, inputParam.HitPosition);
			ControllerBase<SceneTeamController>.Instance.EmitEvent(entity2, EEventName.CharBeKilled, entity3, entity2, requirements, damageResult, inputParam.HitPosition);
		}
		Singleton<EventSystem>.Instance.Emit(EEventName.GlobalCharDamage, entity, entity2, requirements, damageResult, inputParam.HitPosition);
	}
}
