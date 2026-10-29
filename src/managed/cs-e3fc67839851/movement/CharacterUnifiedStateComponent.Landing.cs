/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002edd (global::CharacterUnifiedStateComponent; raw type row 12)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/b8a1b0a963278e911df32576816acf778956864e5242424d007d1f05a082f96e/source/_global/CharacterUnifiedStateComponent.cs
 * Canonical source bytes/SHA256: 45196 / 9187c39662dcd892e512058eea950b5d1aaafacac6095de094198faa80dbb4bd
 * Raw methods root SHA256: c956421ce2a714f5f1d455528985f8e6193a222dd0ff6f9c00b767eee638d655
 * Raw types root SHA256: 7849707d9544c18560d3f2a0ed71be7154142f943d86231027ff5d935dd22df4
 * Scope: five selected ordinary whole method bodies and two observed fields.
 * Inherited members, omitted fields, properties, helpers, callbacks, and
 * lifecycle behavior remain external. The source view is not a complete class
 * and does not promise compilation or runtime verification.
 *
 * Selected fields:
 *   0x0400bec0 CharacterAbilityComponent? AbilityComponent
 *   0x0400bed6 System.Collections.Generic.Dictionary<ECharPositionState, System.Int32> PositionEnumToTag
 */

using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component.Move;
using UnrealEngine;

public partial class CharacterUnifiedStateComponent : BaseUnifiedStateComponent, IComponentDependency
{
	// Field 0x0400bec0; original private CharacterAbilityComponent?; canonical source line 19; no initializer.
	private CharacterAbilityComponent? AbilityComponent;

	// Field 0x0400bed6; original protected static System.Collections.Generic.Dictionary<ECharPositionState, System.Int32>; canonical source line 59; no initializer.
	protected static Dictionary<ECharPositionState, int> PositionEnumToTag;

	// Token 0x06018a3b; original private instance non-virtual; canonical source lines 276-368; raw methods row 451; RVA 7280692; 423 bytes; EH0.
	private void OnCharacterMovementModeChanged(int charId, EMovementMode prevMovementMode, EMovementMode newMovementMode, byte prevCustomMode, byte newCustomMode)
	{
		switch (newMovementMode)
		{
		case EMovementMode.MOVE_None:
			SetPositionState(ECharPositionState.Ground);
			SetMoveState(ECharMoveState.Other);
			break;
		case EMovementMode.MOVE_Walking:
		case EMovementMode.MOVE_NavWalking:
			SetPositionState(ECharPositionState.Ground);
			break;
		case EMovementMode.MOVE_Falling:
			SetPositionState(ECharPositionState.Air);
			if (MoveState != ECharMoveState.KnockUp && MoveState != ECharMoveState.Captured)
			{
				SetMoveState(ECharMoveState.Other);
			}
			break;
		case EMovementMode.MOVE_Flying:
			SetPositionState(ECharPositionState.Air);
			if (MoveState != ECharMoveState.Captured)
			{
				SetMoveState(ECharMoveState.Flying);
			}
			break;
		case EMovementMode.MOVE_WalkingOnAir:
			SetPositionState(ECharPositionState.Air);
			SetMoveState(ECharMoveState.WalkOnAir);
			break;
		case EMovementMode.MOVE_Custom:
			switch (newCustomMode)
			{
			case 0:
				SetPositionState(ECharPositionState.Climb);
				if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.冲刺保持"]))
				{
					SwitchFastClimb(fastClimb: true);
				}
				break;
			case 1:
				SetPositionState(ECharPositionState.Water);
				break;
			case 2:
				SetPositionState(ECharPositionState.Air);
				SetMoveState(ECharMoveState.Glide);
				break;
			case 4:
				SetPositionState(ECharPositionState.Air);
				break;
			case 5:
				SetPositionState(ECharPositionState.Ground);
				break;
			case 6:
				SetPositionState(ECharPositionState.Air);
				break;
			case 7:
				SetPositionState(ECharPositionState.Air);
				SetMoveState(ECharMoveState.Soar);
				break;
			case 8:
				SetPositionState(ECharPositionState.Ski);
				break;
			case 9:
				SetPositionState(ECharPositionState.Air);
				SetMoveState(ECharMoveState.Roll);
				break;
			case 10:
				SetPositionState(ECharPositionState.Air);
				SetMoveState(ECharMoveState.Kite);
				break;
			case 11:
				SetPositionState(ECharPositionState.Ride);
				break;
			case 12:
				SetPositionState(ECharPositionState.RailSlide);
				break;
			case 13:
				SetPositionState(ECharPositionState.Climb);
				break;
			case 15:
				SetPositionState(ECharPositionState.Floating);
				SetMoveState(ECharMoveState.Floating);
				break;
			default:
				SetPositionState(ECharPositionState.Air);
				break;
			}
			break;
		case EMovementMode.MOVE_Swimming:
			break;
		}
	}

	// Token 0x06018a3e; original public override instance virtual; canonical source lines 370-386; raw methods row 454; RVA 7281144; 151 bytes; EH0.
	public override void SetPositionState(ECharPositionState newPositionState)
	{
		ECharPositionState positionState = PositionState;
		if (!ActorComponent.IsMoveAutonomousProxy)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("old", positionState.ToString());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("new", newPositionState.ToString());
			instance.Warn(CombatLog.EDebugModule.UnifiedState, entity, "非主控端尝试修改位置状态", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
		}
		else if (SetPositionStateInternal(newPositionState))
		{
			OnPositionStateChange(positionState, newPositionState);
		}
	}

	// Token 0x06018a40; original private instance non-virtual; canonical source lines 396-418; raw methods row 456; RVA 7281332; 226 bytes; EH0.
	private bool SetPositionStateInternal(ECharPositionState newPositionState)
	{
		ECharPositionState positionState = PositionState;
		if (positionState == newPositionState)
		{
			return false;
		}
		TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.位置状态"]);
		TagComponent.AddTag(PositionEnumToTag[newPositionState]);
		SetPositionSubState(ECharPositionSubState.None, forceAddTag: true);
		CachedPositionState = newPositionState;
		CombatLog instance = Singleton<CombatLog>.Instance;
		Entity entity = base.Entity;
		global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("old", positionState.ToString());
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("new", newPositionState.ToString());
		instance.Info(CombatLog.EDebugModule.UnifiedState, entity, "设置位置状态", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
		if (base.Entity != null)
		{
			UKuroJsModelFunctionLibrary.SetCharacterUnifiedStateComponentPositionState(base.Entity.Id, (byte)newPositionState);
		}
		return true;
	}

	// Token 0x06018a42; original protected override instance virtual; canonical source lines 420-433; raw methods row 458; RVA 7281580; 118 bytes; EH0.
	protected override void OnLand()
	{
		FGameplayTag? gameplayTagById = GameplayTagUtils.GetGameplayTagById(GameplayTagDefine.EGameplayTagId["行为状态.动作状态.落地"]);
		AbilityComponent.SendGameplayEventToActor(gameplayTagById.Value);
		Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.CharOnLand);
		if (MoveState == ECharMoveState.KnockUp)
		{
			SetMoveState(ECharMoveState.StandUp);
		}
		else if (!TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.动作状态.受击"]))
		{
			SetMoveState(ECharMoveState.Other);
		}
	}

	// Token 0x06018a43; original protected override instance virtual; canonical source lines 435-442; raw methods row 459; RVA 7281710; 34 bytes; EH0.
	protected override void OnPositionStateChange(ECharPositionState oldState, ECharPositionState newState)
	{
		if (newState == ECharPositionState.Ground)
		{
			OnLand();
		}
		Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.CharOnPositionStateChanged, oldState, newState);
	}

}
