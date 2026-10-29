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
 * Scope: two selected ordinary whole method bodies. Existing accepted
 * source views already declare AirInertiaHandler and GroundedFrame; no new
 * CharacterMoveComponent fields are declared here. Inherited members, omitted
 * fields, properties, helpers, callbacks, and lifecycle behavior remain
 * external. The source view is not a complete class and does not promise
 * compilation or runtime verification.
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
	// Token 0x0601968f; original protected instance non-virtual; canonical source lines 228-236; raw methods row 809; RVA 7684620; 89 bytes; EH0.
	protected void OnLand()
	{
		GroundedTimeUe = UGameplayStatics.GetGamePlayTimeSeconds(ActorComp.Actor);
		GroundedFrame = Singleton<Time>.Instance.Frame;
		if ((EKuroNetMode)AActor.GetKuroNetMode() == EKuroNetMode.KNM_Net && ActorComp.IsAutonomousProxy)
		{
			Singleton<EventSystem>.Instance.Emit(EEventName.EntityOnLandedPush, base.Entity);
		}
	}

	// Token 0x06019690; original protected instance non-virtual; canonical source lines 238-282; raw methods row 810; RVA 7684724; 402 bytes; EH0.
	protected void OnPositionStateChanged(ECharPositionState oldPositionState, ECharPositionState newPositionState)
	{
		if (oldPositionState == ECharPositionState.Air)
		{
			IsFallingIntoWater = false;
			StopAddMove(AirInertiaHandler);
			AirInertiaHandler = 0;
			if (ActorComp.IsRoleAndCtrlByMe && newPositionState == ECharPositionState.Ground)
			{
				PlayerMotionRequest(MotionType.BeLand);
			}
		}
		switch (newPositionState)
		{
		case ECharPositionState.Water:
		{
			VelocityVector.FromUeVector(ActorComp.ActorVelocityProxy);
			double num = VelocityVector.Size();
			if (num > 800.0)
			{
				VelocityVector.MultiplyEqual(800.0 / num);
				ActorComp?.SetActorVelocity(VelocityVector);
			}
			break;
		}
		case ECharPositionState.Ground:
			if (base.Entity.Active && ActorComp.IsAutonomousProxy)
			{
				base.Entity.GetComponent<CharacterDamageComponent>()?.FallInjure();
			}
			break;
		case ECharPositionState.Air:
			if (HasBaseMovement && !ActorComp.Actor.BasedMovement.bRelativeRotation && DeltaBaseMovementSpeed.HasValue)
			{
				AirInertiaHandler = SetAddMoveWorld(DeltaBaseMovementSpeed, 1.5f, BaseMoveComponent.BaseMoveInheritCurve, AirInertiaHandler, null);
			}
			if (DeltaConveyBeltSpeed.HasValue)
			{
				AirInertiaHandler = SetAddMoveWorld(DeltaConveyBeltSpeed, 1.5f, BaseMoveComponent.BaseMoveInheritCurve, AirInertiaHandler, null);
			}
			break;
		case ECharPositionState.Climb:
			break;
		}
	}

}
