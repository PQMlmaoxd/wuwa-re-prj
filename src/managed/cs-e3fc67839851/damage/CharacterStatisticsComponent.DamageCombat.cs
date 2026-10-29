/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * The original metadata declares CharacterStatisticsComponent as a public,
 * non-partial type. `partial` here is a nonoriginal source-overlay device;
 * this file is not original source and is not a standalone compilable unit.
 *
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002ed4; MethodDef: 0x060189ef.
 * Canonical source: CharacterStatisticsComponent.cs:1020-1043.
 * Original CIL: R43_ONCHARDAMAGECOMBAT.il; RVA 0x6ef508; 165 bytes; 56
 * instructions; EH0. No fields, helpers, constructors, getters, or other
 * original members are declared in this source-only overlay.
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
	private void OnCharDamageCombat(int damage, int elementId, FVectorDouble damagePosition, Entity attacker, Entity victim, bool bCritical, int damageTextType, bool bImmune, long damageId, long? bulletId = null, long? buffId = null, bool? isTargetKilled = null, RequirementPayload? requirements = null)
	{
		if (!IsCombatStarted)
		{
			return;
		}
		CombatDataDamage combatDataDamage = new CombatDataDamage(attacker.Id, damageId, damage, (requirements?.SkillId).GetValueOrDefault(), victim.Id);
		combatDataDamage.ToString();
		ListCombatDataAll.Add(combatDataDamage);
		if (ListViewFilter(combatDataDamage))
		{
			ListCombatData.Add(combatDataDamage);
		}
		if (isTargetKilled.GetValueOrDefault())
		{
			CombatDataKilled combatDataKilled = new CombatDataKilled(attacker.Id, victim.Id);
			combatDataKilled.ToString();
			ListCombatDataAll.Add(combatDataKilled);
			if (ListViewFilter(combatDataKilled))
			{
				ListCombatData.Add(combatDataKilled);
			}
		}
	}
}
