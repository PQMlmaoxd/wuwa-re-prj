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
 * Raw methods root SHA256: eff0bfab8768545cc1af6d79123649a5abf11cff84be795bd16b3549a7698f58
 * Raw types root SHA256: 6bb4ec229cbf2e4573e7dc3a822f3f9230f78d5fa2f3d9c33b23fc8144388971
 * Scope: two selected ordinary whole method bodies and four observed
 * protected fields. Inherited members, omitted fields, helpers, callbacks,
 * and lifecycle behavior remain external. The source view is not a complete
 * class and does not promise compilation or runtime verification.
 *
 * Selected fields:
 *   0x0400c92f CharacterGlideComponent? GlideComp
 *   0x0400c935 System.Double LastGlidingControlTime
 *   0x0400c937 BaseTagComponent? TagComponent
 *   0x0400c939 System.Int32 AirInertiaHandler
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
	// Field 0x0400c92f; original protected CharacterGlideComponent?; no initializer.
	protected CharacterGlideComponent? GlideComp;

	// Field 0x0400c935; original protected System.Double; no initializer.
	protected double LastGlidingControlTime;

	// Field 0x0400c937; original protected BaseTagComponent?; no initializer.
	protected BaseTagComponent? TagComponent;

	// Field 0x0400c939; original protected System.Int32; no initializer.
	protected int AirInertiaHandler;

	// Token 0x060196a9; original protected instance non-virtual; canonical source lines 907-917; raw methods row 835; RVA 7691304; 168 bytes; EH0.
	protected void LimitMaxSpeed()
	{
		VelocityVector.FromUeVector(ActorComp.ActorVelocityProxy);
		double num = VelocityVector.Size();
		float? num2 = base.CurrentMovementSettings?.SprintSpeed;
		if (num > (double?)num2)
		{
			VelocityVector.MultiplyEqual((double)num2.Value / num);
			ActorComp?.SetActorVelocity(VelocityVector);
		}
	}

	// Token 0x060196b2; original public instance non-virtual; canonical source lines 1135-1188; raw methods row 844; RVA 7693356; 331 bytes; EH0.
	public bool TrySetGlide()
	{
		BaseUnifiedStateComponent? unifiedStateComponent = UnifiedStateComponent;
		if (unifiedStateComponent == null || unifiedStateComponent.PositionState != ECharPositionState.Air)
		{
			return false;
		}
		ECharMoveState moveState = UnifiedStateComponent.MoveState;
		if (moveState == ECharMoveState.Glide || moveState == ECharMoveState.Slide)
		{
			return false;
		}
		BaseTagComponent? tagComponent = TagComponent;
		if (tagComponent != null && tagComponent.HasTag(GameplayTagDefine.EGameplayTagId["战斗状态.行为限制.滑翔禁止"]))
		{
			return false;
		}
		if (Singleton<Time>.Instance.WorldTime - LastGlidingControlTime > 0.30000001192092896)
		{
			if (!(GetHeightAboveGround() > 250f))
			{
				BaseTagComponent? tagComponent2 = TagComponent;
				if (tagComponent2 == null || !tagComponent2.HasTag(GameplayTagDefine.EGameplayTagId["战斗状态.行为改变.跳跃进滑翔"]))
				{
					goto IL_0149;
				}
			}
			if (ControllerBase<FormationAttributeController>.Instance.GetValue(EFormationAttributeId.Strength) > 10f)
			{
				CharacterSkillComponent component = base.Entity.GetComponent<CharacterSkillComponent>();
				if (component != null && component.Valid && component.CurrentSkill != null)
				{
					if (!component.CheckGlideCanInterrupt())
					{
						return false;
					}
					component.StopGroup1Skill("滑翔打断技能");
					LimitMaxSpeed();
				}
				CharacterGlideComponent? glideComp = GlideComp;
				if (glideComp != null && glideComp.Valid)
				{
					GlideComp.EnterGlideState("MoveComp");
					StopAddMove(AirInertiaHandler);
					AirInertiaHandler = 0;
					LastGlidingControlTime = Singleton<Time>.Instance.WorldTime;
				}
				return true;
			}
		}
		goto IL_0149;
		IL_0149:
		return false;
	}
}
