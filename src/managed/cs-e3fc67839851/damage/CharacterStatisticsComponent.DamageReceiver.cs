/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * The original metadata declares CharacterStatisticsComponent as a public,
 * non-partial type. `partial` here is a nonoriginal source-overlay device;
 * this file is not original source and is not a standalone compilable unit.
 *
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002ed4; MethodDef: 0x060189b4.
 * Canonical source: CharacterStatisticsComponent.cs:181-202.
 * Original CIL: R42_ONCHARDAMAGEALL.il; RVA 0x6ed8a0; 217 bytes; 85
 * instructions; EH0. OnInit registration is retained only in
 * R42_ONINIT_REGISTRATION.il; its body is deliberately not reconstructed here.
 * No fields, helpers, constructors, getters, or other original members are
 * declared in this source-only overlay.
 */

using System;
using System.Collections.Generic;
using System.Text;
using Aki.Config;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game.Camera;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.RoleUi;
using CSharpScript.Game.World.Model;
using UnrealEngine;

public partial class CharacterStatisticsComponent : EntityComponent, IStaticVariableResetter
{
	private void OnCharDamageAll(Entity attacker, Entity victim, RequirementPayload requirements, DamageResult damageResult, FVectorDouble damagePosition)
	{
		int num = (int)damageResult.Damage;
		Damage damageData = damageResult.DamageData;
		switch ((ECalculationType)damageData.CalculateType)
		{
		case ECalculationType.Heal:
			if (StatisticsEnable)
			{
				OnCharHealStatistic(-num, attacker, victim, damageResult);
			}
			OnCharHealCombat(-num, attacker, victim, damageResult, requirements);
			break;
		case ECalculationType.Hurt:
			if (StatisticsEnable)
			{
				OnCharDamageStatistic(num, (int)damageResult.Element, damagePosition, attacker, victim, requirements.IsCritical.GetValueOrDefault(), damageData.DamageTextType, requirements.IsImmune.GetValueOrDefault(), damageData.Id, requirements.BulletId, requirements.BuffId);
			}
			OnCharDamageCombat(num, (int)damageResult.Element, damagePosition, attacker, victim, requirements.IsCritical.GetValueOrDefault(), damageData.DamageTextType, requirements.IsImmune.GetValueOrDefault(), damageData.Id, requirements.BulletId, requirements.BuffId, requirements.IsTargetKilled, requirements);
			break;
		}
	}
}
