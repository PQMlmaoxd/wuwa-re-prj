/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020030b6 (global::CharacterLockOnComponent; raw type row 30)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/bb09b591a2c22d2d502625deca8d1feff8234907868222898b6703b239c6df7b/source/_global/CharacterLockOnComponent.cs
 * Canonical source bytes/SHA256: 27877 / 6ab5d7a684a9b19886d689c66dc05be5897b13dc6a3422fc0f52c452478f905d
 * Raw methods root SHA256: 407f910cbc23b732df2eeb2b1bbd22611c6729b32a5a86b1488da9d72c8983c3
 * Raw types root SHA256: cc388b9dd215dc483c22c79d99e7e8594a4652c19772712beb8334d037a238f3
 * Scope: two selected ordinary whole method bodies. CharacterLockOnComponent
 * own fields, omitted inherited fields, properties, helpers, callbacks, and
 * lifecycle behavior remain external. The source view is not a complete class
 * and does not promise compilation or runtime verification.
 *
 * No CharacterLockOnComponent own field is declared in this selected view.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game.Camera;
using CSharpScript.Game.Common.Event;
using Google.Protobuf;

public partial class CharacterLockOnComponent : BaseLockOnComponent, IComponentDependency
{
	// Token 0x06019b45; original public instance non-virtual; canonical source lines 716-772; raw methods row 864; RVA 7840124; 674 bytes; EH0.
	public bool ChangeShowTarget(Vector2D inputDirect, float angleCoefficient, float distCoefficient)
	{
		if (IsLookAt || !base.IsHardLock || !HardLockConfig.HasValue || base.GetCurrentInfo == null)
		{
			return false;
		}
		List<LockOnInfo> list = DetectAlternativeTargets(HardLockConfig.Value, isHardLock: true);
		Vector actorLocationProxy = base.ActorLocationProxy;
		GetSkillBoneLocation(base.GetCurrentInfo.EntityHandle, base.GetCurrentInfo.SocketName, TmpVector1);
		TmpVector2.DeepCopy(actorLocationProxy);
		TmpVector2.Z = TmpVector1.Z;
		TmpVector1.SubtractionEqual(actorLocationProxy);
		Singleton<MathUtils>.Instance.LookRotationUpFirst(TmpVector1, Vector.UpVectorProxy, InverseQuat);
		InverseQuat.Inverse(InverseQuat);
		double num = inputDirect.SizeSquared();
		LockOnInfo lockOnInfo = null;
		double num2 = 1E+50;
		for (int i = 0; i < list.Count; i++)
		{
			LockOnInfo lockOnInfo2 = list[i];
			if (!LockOnUtils.IsValidLockOnTarget(lockOnInfo2.EntityHandle) || lockOnInfo2.Equal(base.GetCurrentInfo))
			{
				continue;
			}
			GetSkillBoneLocation(lockOnInfo2.EntityHandle, lockOnInfo2.SocketName, TmpVector1);
			TmpVector1.SubtractionEqual(TmpVector2);
			InverseQuat.RotateVector(TmpVector1, TmpVector1);
			if (Math.Abs(TmpVector1.X) < 1E-08 && Math.Abs(TmpVector1.Y) < 1E-08)
			{
				continue;
			}
			double num3 = Math.Atan2(TmpVector1.Y, TmpVector1.X) * 57.295780181884766;
			double num4 = Math.Asin(TmpVector1.Z / TmpVector1.Size()) * 57.295780181884766;
			double num5 = Math.Sqrt(num3 * num3 + num4 * num4);
			TmpVector2D.X = num3;
			TmpVector2D.Y = num4;
			double num6 = TmpVector2D.DotProduct(inputDirect);
			if (!(num6 < 0.0))
			{
				double num7 = Math.Acos(num6 / Math.Sqrt(TmpVector2D.SizeSquared() * num)) * 57.295780181884766;
				double num8 = (double)angleCoefficient * num7 / 180.0 + (double)distCoefficient * num5;
				if (num8 < num2)
				{
					num2 = num8;
					lockOnInfo = lockOnInfo2;
				}
			}
		}
		if (lockOnInfo != null)
		{
			SetCurrentInfo(lockOnInfo);
			SetShowTarget(lockOnInfo.EntityHandle, lockOnInfo.SocketName, isHardLock: true);
			LockOnDebug.SetDebugArrow(lockOnInfo, EColorType.Black);
			return true;
		}
		return false;
	}

	// Token 0x06019b2f; original public override instance virtual; canonical source lines 164-204; raw methods row 842; RVA 7835972; 406 bytes; EH0. Raw is_virtual=true; no raw is_newslot claim.
	public override bool SetShowTarget(EntityHandle? target, string socketName = "", bool isHardLock = false)
	{
		CharacterActorComponent characterActorComponent = target?.Entity?.GetComponent<CharacterActorComponent>();
		string text = socketName;
		if (target != null && target.Valid)
		{
			if ((characterActorComponent?.CameraLockOnConfig?.IsEnabled).GetValueOrDefault() && !IsLookAt)
			{
				text = characterActorComponent.CameraLockOnConfig.CameraLockOnBoneName;
			}
			if (!base.HasShowTarget)
			{
				TagComponent?.AddTag(GameplayTagDefine.EGameplayTagId["角色.Common.拥有锁定目标"]);
			}
			ControllerBase<SceneTeamController>.Instance.EmitEvent(base.Entity, EEventName.CharSetShowTarget, target.Id, text, isHardLock);
		}
		else if (base.HasShowTarget)
		{
			TagComponent?.RemoveTag(GameplayTagDefine.EGameplayTagId["角色.Common.拥有锁定目标"]);
			ControllerBase<SceneTeamController>.Instance.EmitEvent(base.Entity, EEventName.CharEndShowTarget, base.ShowTarget?.Id ?? (-1), base.ShowTargetSocket, base.IsHardLock);
		}
		ShowTargetSetTime = Singleton<Time>.Instance.WorldTime;
		if (base.ShowTarget == target && base.ShowTargetSocket == text)
		{
			return true;
		}
		if (target == null)
		{
			ShowTargetInternal = null;
			ShowTargetSocketInternal = "";
			GlobalData.BpEventManager.小队技能目标改变时.Broadcast(null);
			return false;
		}
		ShowTargetInternal = target;
		ShowTargetSocketInternal = text;
		if (characterActorComponent != null)
		{
			GlobalData.BpEventManager.小队技能目标改变时.Broadcast(characterActorComponent.Actor);
		}
		return true;
	}
}
