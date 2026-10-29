/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * The original metadata declares CharacterStatisticsComponent as a public,
 * non-partial type. `partial` here is a nonoriginal source-overlay device;
 * this file is not original source and is not a standalone compilable unit.
 *
 * Assembly: private/r5b1/materialized/CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002ed4; MethodDef: 0x060189f4.
 * Canonical source: CharacterStatisticsComponent.cs:1103-1138.
 * Original CIL: R44_LISTVIEWFILTER.il; RVA 0x6ef758; 194 bytes; 77
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
	private static bool ListViewFilter(CombatDataBase data)
	{
		if (CurrentAttackerId > 0 && CurrentAttackerId != data.AttackerId)
		{
			return false;
		}
		if (CurrentTargetId > 0 && CurrentTargetId != data.TargetId)
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.Damage) && data is CombatDataDamage)
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.Heal) && data is CombatDataHeal)
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.SkillUsed) && data is CombatDataSkill)
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.State) && (data is CombatDataBuffRemoved || data is CombatDataBuffAdded))
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.Kill) && data is CombatDataKilled)
		{
			return false;
		}
		if (!MapCombatDataType.ContainsKey(ECombatDataType.Revive) && data is CombatDataRevive)
		{
			return false;
		}
		return true;
	}
}
