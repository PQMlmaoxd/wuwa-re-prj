/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003206 (global::RoleTeamComponent)
 * Scope: sixteen selected whole source-level method bodies only.
 * Missing members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Character.Role.Common;
using AkiClient.Game.Aki.Render.RuntimeBP.Character.MaterialController;
using CSharpScript.Game;
using CSharpScript.Game.Camera;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Effect;
using CSharpScript.Game.Module.Plot;
using CSharpScript.Game.Module.RoleUi;
using CSharpScript.Game.NewWorld.Character.Common.Component;
using CSharpScript.Game.NewWorld.Character.Common.Component.Explore;
using CSharpScript.Game.NewWorld.Character.Common.Component.Move;
using CSharpScript.Game.NewWorld.Character.Role.Component;
using CSharpScript.Game.NewWorld.Pawn.Component;
using CSharpScript.Game.Utils;
using UnrealEngine;

public partial class RoleTeamComponent : EntityComponent
{
    private CharacterActorComponent? ActorComp;

    private CharacterBuffComponent? BuffComponent;

    private BaseTagComponent? TagComponent;

    private CharacterUnifiedStateComponent? StateComp;

    private CharacterFloatingComponent? FloatingComp;

    private RoleInheritComponent? InheritComp;

    private CharacterSkillComponent? SkillComp;

    private RoleQteComponent? QteComp;

    private CharacterMoveComponent? MoveComp;

    private CharacterInputComponent? InputComp;

    private CharacterLockOnComponent? LockOnComp;

    private CharacterManipulateComponent? ManipulateComp;

    private CharacterManipulateInteractComponent? ManipulateInteractComp;

    private CharacterExploreComponent? CharacterExploreComp;

    private RoleSceneInteractComponent? RoleSceneInteractComp;

    private PD_CharacterControllerData_C? GoDownMaterial;

    private double ChangeRoleCdEndTime = -1.0;

    private ETeamState? TeamState;

    public bool GoBattleSkill;

    private ITagTask? NotHideTagTask;

    private ITagTask? IsStiffTagTask;

    private int QuitEffectHandle;

    private TimerHandle? WaitQuitEffectTimer;

    // Token 0x0601a8fb; original private, non-virtual; canonical source lines 143-154.
    private void ClearWaiters()
    {
        NotHideTagTask?.EndTask();
        NotHideTagTask = null;
        IsStiffTagTask?.EndTask();
        IsStiffTagTask = null;
        if (WaitQuitEffectTimer != null)
        {
            TimerSystem.Instance.Remove(WaitQuitEffectTimer);
            WaitQuitEffectTimer = null;
        }
    }

    // Token 0x0601a8fd; original public static, non-virtual; canonical source lines 161-232.
    public static void OnChangeRole(EntityHandle? lastRole, EntityHandle newRole, bool useGoBattleSkill, double coolDownTime, bool goDownWaitSkillEnd, bool allowRefreshTransform, bool forceInheritTransform)
    {
        Log instance = Singleton<Log>.Instance;
        ReadOnlySpan<(string, object)> details =
        [
            ("Last", lastRole?.Id),
            ("New", newRole.Id)
        ];
        instance.Info((ELogModule)43, (ELogAuthor)49, "执行战斗换人", details);
        RoleTeamComponent roleTeamComponent = lastRole?.Entity?.GetComponent<RoleTeamComponent>();
        RoleTeamComponent component = newRole.Entity.GetComponent<RoleTeamComponent>();
        ETeamGroupType valueOrDefault = ModelBase<SceneTeamModel>.Instance.CurrentGroupType.GetValueOrDefault();
        bool flag = SceneTeamDefine.needInheritTypeSet.Contains(valueOrDefault);
        CharacterSkillComponent characterSkillComponent = roleTeamComponent?.SkillComp;
        if (goDownWaitSkillEnd && characterSkillComponent != null && characterSkillComponent.CurrentSkill != null && !characterSkillComponent.IsMainSkillReadyEnd)
        {
            EntityHandle entityHandle = characterSkillComponent.SkillTarget;
            if (entityHandle == null && ControllerBase<FormationDataController>.Instance.GlobalIsInFight)
            {
                CharacterLockOnComponent? lockOnComp = roleTeamComponent.LockOnComp;
                lockOnComp?.DetectSoftLockTarget(new SkillTargetImpl());
                entityHandle = lockOnComp?.GetCurrentTarget();
            }
            CharacterBuffComponent buffComponent = roleTeamComponent.BuffComponent;
            if (entityHandle != null && buffComponent != null && buffComponent.HasBuffAuthority())
            {
                buffComponent.AddBuff(1101006002L, new AddBuffParam
                {
                    InstigatorId = buffComponent.CreatureDataId,
                    Reason = "战斗换人"
                });
            }
        }
        bool specialRollInAir = false;
        bool flag2 = false;
        BaseTagComponent baseTagComponent = roleTeamComponent?.TagComponent;
        if (baseTagComponent != null)
        {
            flag2 = baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.位置状态.攀爬"]) || baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.位置状态.水中"]);
            specialRollInAir = baseTagComponent.HasAllTag(new int[2]
            {
                GameplayTagDefine.EGameplayTagId["行为状态.位置状态.空中"],
                GameplayTagDefine.EGameplayTagId["行为状态.动作状态.特殊滚动"]
            });
        }
        component.PerformPossess();
        if (roleTeamComponent != null && roleTeamComponent != component)
        {
            Log instance2 = Singleton<Log>.Instance;
            (string, object) reference = ("Entity", roleTeamComponent.Entity?.Id);
            instance2.Info((ELogModule)43, (ELogAuthor)49, "角色下场", new ReadOnlySpan<(string, object)>(ref reference));
            if (GlobalData.GameInstance != null)
            {
                GlobalData.BpEventManager.当换人完成时.Broadcast();
            }
            component.ManipulateComp?.SetDataFromOldRole(lastRole);
            component.ManipulateInteractComp?.SetDataFromOldRole(lastRole);
            component.CharacterExploreComp?.SetDataFromOldRole(lastRole);
            component.RoleSceneInteractComp?.SetDataFromOldRole(lastRole);
            roleTeamComponent.ManipulateInteractComp?.ClearTarget();
            roleTeamComponent.ClearMovePlatformAttach();
            if (flag || !roleTeamComponent.TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.切人不隐藏"]))
            {
                roleTeamComponent.SkillComp.StopGroup1Skill("RoleTeamComponent.OnChangeRole");
            }
            bool skill = roleTeamComponent.TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.技能中"]) && !roleTeamComponent.SkillComp.IsMainSkillReadyEnd;
            RoleInheritComponent.StateInherit(roleTeamComponent.InheritComp, component.InheritComp, component.QteComp.IsInQte ? ERoleInheritType.QTE : ERoleInheritType.NORMAL, skill);
            roleTeamComponent.GoDown(flag);
            roleTeamComponent.CommitChangeRoleCoolDown(coolDownTime);
        }
        bool forceInherit = flag2 || forceInheritTransform || flag;
        component.RefreshPosition(lastRole, specialRollInAir, allowRefreshTransform, forceInherit);
        component.GoBattle(!flag2 && useGoBattleSkill);
    }

    // Token 0x0601a8fe; original private, non-virtual; canonical source lines 234-249.
    private void PerformPossess()
    {
        AController characterController = Global.CharacterController;
        TsBaseCharacter actor = ActorComp.Actor;
        if (characterController.Pawn != actor)
        {
            actor.Mesh.AddTickPrerequisiteActor(characterController);
            characterController.Possess(actor);
            MoveComp.StopMove(isStop: false, "RoleTeamComponent.PerformPossess");
            CharacterInputComponent? inputComp = InputComp;
            if (inputComp == null || !inputComp.Active)
            {
                InputComp?.SetActive(value: true);
            }
        }
    }

    // Token 0x0601a902; original private, non-virtual; canonical source lines 390-443.
    private void GoBattle(bool useGoBattleSkill)
    {
        if (QteComp.IsInQte)
        {
            GoBattleSkill = false;
        }
        else
        {
            bool flag = TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.BUFF通用标识.无QTE则强制出场技"]);
            LockOnComp.DetectSoftLockTarget(new SkillTargetImpl());
            EntityHandle currentTarget = LockOnComp.GetCurrentTarget();
            bool flag2 = TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]);
            GoBattleSkill = flag || (useGoBattleSkill && flag2 && currentTarget != null);
        }
        ClearWaiters();
        int[] outGameRoleTags = CharacterUnifiedStateComponent.outGameRoleTags;
        foreach (int value in outGameRoleTags)
        {
            TagComponent.RemoveTag(value);
        }
        SetTeamTag(ETeamState.OnStage);
        if (!ModelBase<PlotModel>.Instance.InSeamlessFormation)
        {
            base.Entity.EnableByKey(EEntityDisableKey.GoDown);
        }
        else
        {
            base.Entity.DisableByKey(EEntityDisableKey.GoDown);
        }
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnRoleGoUpEnable);
        Singleton<TickProcessSystem>.Instance.RegisterOnceTickProcess(ETickingGroup.TG_PostUpdateWork, tickEvenPaused: true, NextSpawnGoBattleMaterial);
        ActorComp.KuroMoveAlongFloor(Vector.ZeroVector, 0f, "GoBattle");
        if (!Singleton<UiCameraAnimationManager>.Instance.IsActivate())
        {
            ControllerBase<CameraController>.Instance.ExitCameraMode(ECustomCameraMode.Widget);
        }
        if (GoBattleSkill)
        {
            bool flag3 = StateComp.PositionState == ECharPositionState.Air;
            int num;
            if (StateComp.PositionState == ECharPositionState.Floating)
            {
                CharacterFloatingComponent? floatingComp = FloatingComp;
                num = ((floatingComp != null && !floatingComp.IsOnGround) ? 1 : 0);
            }
            else
            {
                num = 0;
            }
            bool flag4 = (byte)num != 0;
            ActorComp.Actor.FightCommand(flag3 || flag4);
        }
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnRoleGoUp);
    }

    // Token 0x0601a905; original private, non-virtual; canonical source lines 486-528.
    private void GoDown(bool forceGoDown)
    {
        bool flag = TagComponent?.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.切人不隐藏"]) ?? false;
        bool flag2 = TagComponent?.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.被击硬直时间"]) ?? false;
        if (forceGoDown || (!flag && !flag2))
        {
            Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，立即隐藏", default(ReadOnlySpan<(string, object)>));
            SetTeamTag(ETeamState.UnderStage);
            base.Entity.DisableByKey(EEntityDisableKey.GoDown);
            ActorComp.RestoreDefaultController();
            MoveComp?.StopAllAddMove();
            Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnRoleGoDownFinish);
            return;
        }
        Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，等待切人不隐藏Tag、硬直时间Tag移除再隐藏角色", default(ReadOnlySpan<(string, object)>));
        if (flag && NotHideTagTask == null)
        {
            NotHideTagTask = TagComponent.ListenForTagAddOrRemove(GameplayTagDefine.EGameplayTagId["角色.Common.切人不隐藏"], delegate(int tagId, bool tagExist)
            {
                if (!tagExist)
                {
                    NotHideTagTask?.EndTask();
                    NotHideTagTask = null;
                    TryGoDown();
                }
            });
        }
        if (flag2 && IsStiffTagTask == null)
        {
            IsStiffTagTask = TagComponent.ListenForTagAddOrRemove(GameplayTagDefine.EGameplayTagId["功能.功能制作.被击硬直时间"], delegate(int tagId, bool tagExist)
            {
                if (!tagExist)
                {
                    IsStiffTagTask?.EndTask();
                    IsStiffTagTask = null;
                    TryGoDown();
                }
            });
        }
        SetTeamTag(ETeamState.OnStageWithoutControl);
        ActorComp.RestoreDefaultController();
        MoveComp?.StopAllAddMove();
    }

    // Token 0x0601a906; original private, non-virtual; canonical source lines 530-540.
    private void TryGoDown()
    {
        bool flag = NotHideTagTask == null && IsStiffTagTask == null;
        Log instance = Singleton<Log>.Instance;
        (string, object) reference = ("CanGoDown", flag);
        instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，尝试隐藏角色", new ReadOnlySpan<(string, object)>(ref reference));
        if (flag && ModelBase<SceneTeamModel>.Instance.GetCurrentEntity?.Id != base.Entity.Id)
        {
            DisableRoleWithEffect();
        }
    }

    // Token 0x0601a907; original public, non-virtual; canonical source lines 542-555.
    public void InterruptDisableWithEffect()
    {
        if (WaitQuitEffectTimer != null)
        {
            TimerSystem.Instance.Remove(WaitQuitEffectTimer);
            WaitQuitEffectTimer = null;
        }
        int quitEffectHandle = QuitEffectHandle;
        if (quitEffectHandle != 0)
        {
            ActorComp.Actor.CharRenderingComponent.RemoveMaterialControllerData(quitEffectHandle);
            QuitEffectHandle = 0;
        }
    }

    // Token 0x0601a908; original public, non-virtual; canonical source lines 557-578.
    // Generated association: parent ldftn 0x0071; MethodDef 0x0601a920, raw row 927.
    public void DisableRoleWithEffect()
    {
        if (WaitQuitEffectTimer != null)
        {
            Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，正在播放特效等待隐藏", default(ReadOnlySpan<(string, object)>));
            return;
        }
        Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，播放特效后再隐藏", default(ReadOnlySpan<(string, object)>));
        if (GoDownMaterial != null)
        {
            QuitEffectHandle = ActorComp.Actor.CharRenderingComponent.AddMaterialControllerData(GoDownMaterial);
        }
        WaitQuitEffectTimer = TimerSystem.Instance.Delay(delegate
        {
            WaitQuitEffectTimer = null;
            DisableRole();
        }, 300f, null, null, needCheckMaxInterval: true, ModelBase<CharacterModel>.Instance.InverseSelfCenteredTimeDilation);
        if (ModelBase<GameModeModel>.Instance.IsMulti)
        {
            ControllerBase<SceneTeamController>.Instance.RoleGoDownPush(base.Entity);
        }
    }

    // Token 0x0601a909; original public, non-virtual; canonical source lines 580-593.
    public void DisableRoleWithoutEffect()
    {
        Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "角色下场，立刻隐藏", default(ReadOnlySpan<(string, object)>));
        if (WaitQuitEffectTimer != null)
        {
            TimerSystem.Instance.Remove(WaitQuitEffectTimer);
            WaitQuitEffectTimer = null;
        }
        DisableRole();
        if (ModelBase<GameModeModel>.Instance.IsMulti)
        {
            ControllerBase<SceneTeamController>.Instance.RoleGoDownPush(base.Entity);
        }
    }

    // Token 0x0601a90a; original private, non-virtual; canonical source lines 595-609.
    private void DisableRole()
    {
        int quitEffectHandle = QuitEffectHandle;
        if (quitEffectHandle != 0)
        {
            ActorComp.Actor.CharRenderingComponent.RemoveMaterialControllerData(quitEffectHandle);
            QuitEffectHandle = 0;
        }
        SkillComp.StopAllSkills("RoleTeamComponent.DisableRole");
        UpdateStealthTag(onStageWithoutControl: false);
        SetTeamTag(ETeamState.UnderStage);
        MoveComp?.CharacterMovement?.SetDefaultMovementMode();
        base.Entity.DisableByKey(EEntityDisableKey.GoDown);
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnRoleGoDownFinish);
    }

    // Token 0x0601a90b; original private, non-virtual; canonical source lines 611-616.
    private void CommitChangeRoleCoolDown(double coolDown)
    {
        double num = coolDown * 1000.0;
        ChangeRoleCdEndTime = num + Singleton<Time>.Instance.PlayerWorldTime;
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnChangeRoleCoolDownChanged, coolDown);
    }

    // Token 0x0601a90c; original public, non-virtual; canonical source lines 618-626.
    public bool IsChangeRoleCoolDown()
    {
        if (GetChangeRoleCoolDown() > 0.0)
        {
            return true;
        }
        ChangeRoleCdEndTime = -1.0;
        return false;
    }

    // Token 0x0601a90d; original public, non-virtual; canonical source lines 628-635.
    public double GetChangeRoleCoolDown()
    {
        if (!(ChangeRoleCdEndTime > 0.0))
        {
            return -1.0;
        }
        return ChangeRoleCdEndTime - Singleton<Time>.Instance.PlayerWorldTime;
    }

    // Token 0x0601a90e; original public, non-virtual; canonical source lines 637-644.
    public void SetTeamTag(ETeamState state)
    {
        TeamState = state;
        if (base.Entity.IsInit)
        {
            RefreshTeamTag();
        }
    }

    // Token 0x0601a90f; original private, non-virtual; canonical source lines 646-698.
    private void RefreshTeamTag()
    {
        if (!TeamState.HasValue)
        {
            return;
        }
        ETeamState? teamState = TeamState;
        if (!teamState.HasValue)
        {
            return;
        }
        switch (teamState.GetValueOrDefault())
        {
        case ETeamState.OnStage:
            TagComponent.AddTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台"]);
            UpdateStealthTag(onStageWithoutControl: false);
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]);
            }
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台不受控制"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台不受控制"]);
            }
            break;
        case ETeamState.OnStageWithoutControl:
            TagComponent.AddTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台不受控制"]);
            UpdateStealthTag(onStageWithoutControl: true);
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]);
            }
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台"]);
            }
            break;
        case ETeamState.UnderStage:
            if (ActorComp.IsAutonomousProxy)
            {
                TagComponent.AddTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.后台"]);
            }
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台"]);
            }
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台不受控制"]))
            {
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.前台不受控制"]);
            }
            break;
        }
    }

    // Token 0x0601a911; original private, non-virtual; canonical source lines 705-715.
    private void UpdateStealthTag(bool onStageWithoutControl)
    {
        if (onStageWithoutControl)
        {
            TagComponent?.AddTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.不被敌方子弹命中.前台不受控制"]);
        }
        else
        {
            TagComponent?.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.不被敌方子弹命中.前台不受控制"]);
        }
    }
}
