/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This source-only partial view is not original source and is not a
 * standalone-compilation claim. It retains two selected command-consumer
 * bodies in full; the other component members and command helpers remain
 * external or UNKNOWN.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003055 (global::CharacterInputComponent; raw type row 175)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/637b99ed0cc50d209bcf625baa318a102beb758f85f8b716009e5d239de258a6/source/_global/CharacterInputComponent.cs
 * Raw methods root SHA256: 920c5cf907f476e694c4666f1a3e328bbaeb0dc4297883006fb5e85dbed884c2
 * Raw types root SHA256: ff1b0e90290ca5f684bf38434b703f75d58ea68a7806724001b20177070deb4f
 * Scope: two selected R24 whole bodies, rows 238-239. The empty Swallow
 * case is observed source behavior, not a stub. Async activation is a
 * fire-and-forget request; it is not a success or ownership guarantee.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Character.Input.Blueprints;
using AkiClient.Game.Aki.Character.Input.ControlMonster;
using AkiClient.Game.Aki.Character.Input.Enum;
using AkiClient.Game.Aki.Character.Input.Structures;
using AkiClient.Game.Aki.Character.Role.Common.Data.Structure;
using CSharpScript.Core.Common;
using CSharpScript.Game;
using CSharpScript.Game.Camera;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Input;
using CSharpScript.Game.Module.BattleUi;
using CSharpScript.Game.Module.Plot;
using CSharpScript.Game.Module.RoleUi;
using CSharpScript.Game.Module.SkillButtonUi;
using CSharpScript.Game.NewWorld.Character.Common.Component;
using CSharpScript.Game.NewWorld.Character.Common.Component.Move;
using CSharpScript.Game.Ui;
using Cysharp.Threading.Tasks;
using UnrealEngine;

public partial class CharacterInputComponent : EntityComponent, IInputHandler, IStaticVariableResetter
{
	// 0x0400c8ad; canonical source line 89. Other component fields remain
	// external to this selected command-consumer view.
	private CharacterAbilityComponent? AbilityComp;

	// 0x0400c8c2; canonical source line 131.
	private readonly Dictionary<ECommandType, double> CommandTimeMap = new Dictionary<ECommandType, double>();

	// Token 0x060195f1; original private instance; canonical source lines 1545-1595;
	// raw methods row 525; RVA 7665980; 269 bytes; EH0.
	private void ExecuteInputCommand(InputCommand inputCommand, string context)
	{
		SInputCommand command = inputCommand.Command;
		ECommandType eCommandType = command.CommandType;
		switch (eCommandType)
		{
		case ECommandType.Skill:
			if (Singleton<Info>.Instance.IsInGamepad() && inputCommand.Action == CSharpScript.Game.Input.EInputAction.攻击 && inputCommand.State == EInputState.Release && GetCommandInterval(ECommandType.Climb) < 0.25f)
			{
				RemoveCommandInterval(ECommandType.Climb);
			}
			else
			{
				ExecuteSkill(command.IntValue, context);
			}
			break;
		case ECommandType.Jump:
			ExecuteJump(command);
			break;
		case ECommandType.Climb:
			ExecuteClimb(command);
			CommandTimeMap[eCommandType] = Singleton<Time>.Instance.WorldTimeSeconds;
			break;
		case ECommandType.Sprint:
			ExecuteSprint(command);
			break;
		case ECommandType.FastSwim:
			ExecuteFastSwim(command);
			break;
		case ECommandType.FastClimb:
			ExecuteFastClimb(command);
			break;
		case ECommandType.SwitchCharacter:
			ExecuteSwitchCharacter(command.IntValue);
			break;
		case ECommandType.SwitchWalk:
			ExecuteSwitchWalk(command);
			break;
		case ECommandType.SendGameplayEvent:
			AbilityComp.SendGameplayEventToActor(command.TagValue);
			break;
		case ECommandType.XaBoost:
			ExecuteXaBoost(command);
			break;
		case ECommandType.Drop:
			ExecuteDrop(command);
			break;
		case ECommandType.Swallow:
			break;
		}
	}

	// Token 0x060195fb; original private instance; canonical source lines 1669-1676;
	// raw methods row 535; RVA 7666551; 45 bytes; EH0. The canonical IL
	// comment at IL_0022 is omitted; it records an ABI/decompiler uncertainty,
	// not additional body semantics.
	private void ExecuteSkill(int skillId, string context)
	{
		UniTaskExtensions.Forget<bool>(base.Entity.GetComponent<CharacterSkillComponent>().BeginSkillAsync(skillId, new SkillParam
		{
			Reason = "CharacterInputComponent.ExecuteSkill." + context
		}));
	}
}
