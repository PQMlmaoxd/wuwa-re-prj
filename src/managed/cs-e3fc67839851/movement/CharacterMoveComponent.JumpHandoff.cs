/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200305b (global::CharacterMoveComponent; raw type row 173)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/77fe430bc5bcbe53fa26959fddb3e477c149904e0368bf0843a29cbc6a7b6998/source/_global/CharacterMoveComponent.cs
 * Canonical source bytes/SHA256: 68749 / d2ee7fe77aa3a2fdf3853b8a7bad4833070b5ce852295d13c985fe8b4a9ab735
 * Raw methods root SHA256: eff0bfab8768545cc1af6d79123649a5abf11cff84be795bd16b3549a7698f58
 * Raw types root SHA256: 6bb4ec229cbf2e4573e7dc3a822f3f9230f78d5fa2f3d9c33b23fc8144388971
 * Scope: six selected ordinary whole method bodies and three observed
 * protected fields. Inherited members, omitted fields, helpers, callbacks,
 * and lifecycle behavior remain external. The source view is not a complete
 * class and does not promise compilation or runtime verification.
 *
 * Selected fields:
 *   0x0400c932 CharacterFloatingComponent? FloatingComp
 *   0x0400c93c System.Double TryGlideTime
 *   0x0400c941 System.Int32 GroundedFrame
 *
 * Accepted fields in CharacterMoveComponent.cs are intentionally not
 * redeclared: GlideComp, LastGlidingControlTime, TagComponent, and
 * AirInertiaHandler. AnimComp, JumpFrameCount, and UnifiedStateComponent are
 * inherited external members of BaseMoveComponent.
 */

using System;
using System.Collections.Generic;
using System.Diagnostics;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.LevelGamePlay;
using CSharpScript.Game.Module.AdventureGuide;
using CSharpScript.Game.Module.Teleport;
using CSharpScript.Game.NewWorld.Character.Common.Component.Move;
using Google.Protobuf;
using UnrealEngine;

public partial class CharacterMoveComponent : BaseMoveComponent, IComponentDependency, IStaticVariableResetter
{
	// Field 0x0400c932; original protected CharacterFloatingComponent?; canonical source line 59; no initializer.
	protected CharacterFloatingComponent? FloatingComp;

	// Field 0x0400c93c; original protected System.Double; canonical source line 79; no initializer.
	protected double TryGlideTime;

	// Field 0x0400c941; original protected System.Int32; canonical source line 89; no initializer.
	protected int GroundedFrame;

	// Token 0x060196ad; original public instance non-virtual; canonical source lines 948-1009; raw methods row 839; RVA 7691656; 419 bytes; EH0.
	public void JumpPress()
	{
		if (CheckInHit() || !CanJumpPress())
		{
			return;
		}
		CharacterFloatingComponent? floatingComp = FloatingComp;
		if (floatingComp != null && floatingComp.IsFloating)
		{
			base.Entity.GetComponent<CharacterFloatingComponent>()?.JumpPressInAir();
			TagComponent?.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.技能中"]);
			CharacterSkillComponent component = base.Entity.GetComponent<CharacterSkillComponent>();
			if (component != null && component.Valid && component.CurrentSkill != null)
			{
				component.StopGroup1Skill("跳跃打断技能");
			}
			OnJump();
			PlayerMotionRequest(MotionType.MotionJump);
			return;
		}
		ECharPositionState? eCharPositionState = UnifiedStateComponent?.PositionState;
		bool flag = eCharPositionState == ECharPositionState.Ground;
		bool flag2 = eCharPositionState.GetValueOrDefault() == ECharPositionState.Climb;
		if (flag || flag2)
		{
			TagComponent?.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.技能中"]);
			CharacterSkillComponent component2 = base.Entity.GetComponent<CharacterSkillComponent>();
			if (component2 != null && component2.Valid && component2.CurrentSkill != null)
			{
				component2.StopGroup1Skill("跳跃打断技能");
				LimitMaxSpeed();
			}
			OnJump();
			if (flag)
			{
				PlayerMotionRequest(MotionType.MotionJump);
			}
			return;
		}
		if (eCharPositionState.GetValueOrDefault() == ECharPositionState.Air)
		{
			if (JumpPressInAir())
			{
				return;
			}
		}
		else if (eCharPositionState.GetValueOrDefault() == ECharPositionState.Ski)
		{
			if (JumpPressInSki())
			{
				return;
			}
		}
		else if (eCharPositionState.GetValueOrDefault() == ECharPositionState.RailSlide && JumpPressInRailSlide())
		{
			return;
		}
		if (!TrySetGlide())
		{
			TryGlideTime = 500.0;
		}
	}

	// Token 0x060196ae; original protected instance non-virtual; canonical source lines 1011-1018; raw methods row 840; RVA 7692088; 68 bytes; EH0.
	protected bool CheckInHit()
	{
		if (TagComponent != null && (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.动作状态.受击"]) || TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.被击硬直时间"])))
		{
			return true;
		}
		return false;
	}

	// Token 0x060196a7; original public override instance virtual; canonical source lines 862-895; raw methods row 833; RVA 7691020; 249 bytes; EH0.
	public override bool CanJumpPress()
	{
		if (GroundedFrame > Singleton<Time>.Instance.Frame - 2)
		{
			return false;
		}
		CharacterSkillComponent component = base.Entity.GetComponent<CharacterSkillComponent>();
		ECharPositionState? eCharPositionState = UnifiedStateComponent?.PositionState;
		ECharMoveState? eCharMoveState = UnifiedStateComponent?.MoveState;
		switch (eCharPositionState)
		{
		case ECharPositionState.Ground:
		case ECharPositionState.Climb:
		case ECharPositionState.Ski:
			if (component == null || !component.Valid || component.CheckJumpCanInterrupt())
			{
				return JumpCheck();
			}
			return false;
		case ECharPositionState.Air:
			if (eCharMoveState.GetValueOrDefault() == ECharMoveState.Glide)
			{
				return Singleton<Time>.Instance.WorldTime - LastGlidingControlTime > 0.30000001192092896;
			}
			eCharMoveState.GetValueOrDefault();
			_ = 22;
			return true;
		case ECharPositionState.RailSlide:
		case ECharPositionState.Floating:
			return true;
		default:
			return false;
		}
	}

	// Token 0x060196a6; original protected instance non-virtual; canonical source lines 843-860; raw methods row 832; RVA 7690928; 79 bytes; EH0.
	protected bool JumpCheck()
	{
		if (!CanResponseInput() || ContainsTag(GameplayTagDefine.EGameplayTagId["战斗状态.行为限制.禁止跳跃"]))
		{
			return false;
		}
		BaseUnifiedStateComponent? unifiedStateComponent = UnifiedStateComponent;
		if (unifiedStateComponent == null || unifiedStateComponent.PositionState != 0)
		{
			BaseUnifiedStateComponent? unifiedStateComponent2 = UnifiedStateComponent;
			if (unifiedStateComponent2 == null)
			{
				return false;
			}
			return unifiedStateComponent2.PositionState == ECharPositionState.Ski;
		}
		return true;
	}

	// Token 0x060196aa; original protected instance non-virtual; canonical source lines 919-928; raw methods row 836; RVA 7691484; 68 bytes; EH0.
	protected void OnJump()
	{
		CharacterAnimationComponent component = base.Entity.GetComponent<CharacterAnimationComponent>();
		if (component != null && component.Valid && component.MainAnimInstance != null)
		{
			component.MainAnimInstance.Montage_Stop(0f);
		}
		JumpFrameCount = 3;
		AnimComp.OnJump();
	}

	// Token 0x060196c6; original public instance non-virtual; canonical source lines 1504-1509; raw methods row 864; RVA 7696020; 32 bytes; EH0. Parameter MotionType type is required; no optional default.
	public void PlayerMotionRequest(MotionType type)
	{
		PlayerMotionRequest playerMotionRequest = Aki.Protocol.PlayerMotionRequest.Create();
		playerMotionRequest.Motion = type;
		Singleton<Net>.Instance.Call<PlayerMotionResponse>(ERequestMessageId.PlayerMotionRequest, (IMessage)(object)playerMotionRequest, null);
	}
}
