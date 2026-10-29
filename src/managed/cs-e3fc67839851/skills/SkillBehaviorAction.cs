/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200312e (global::SkillBehaviorAction)
 * Scope: three selected whole source-level method bodies only.
 * Missing members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using Aki.Protocol.Summon;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game;
using CSharpScript.Game.Camera;
using CSharpScript.Game.NewWorld.SceneItem.RefCompController;
using UnrealEngine;

public partial class SkillBehaviorAction
{
    // Token 0x0601a118; original public static, non-virtual; canonical source lines 46-55.
    public static void BeginGroup(TArray<SSkillBehaviorAction> skillBehaviorActionGroup, IBeginSkillBehaviorActionParam param)
    {
        if (param.Entity.GetComponent<CharacterActorComponent>().IsAutonomousProxy)
        {
            for (int i = 0; i < skillBehaviorActionGroup.Num(); i++)
            {
                Begin(skillBehaviorActionGroup.Get(i), param);
            }
        }
    }

    // Token 0x0601a119; original public static, non-virtual; canonical source lines 57-125.
    // EH: catch System.Exception (try 141..386; handler 386..501).
    public static void Begin(SSkillBehaviorAction skillBehaviorAction, IBeginSkillBehaviorActionParam param)
    {
        Entity entity = param.Entity;
        ReadOnlySpan<(string, object)> details =
        [
            ("技能Id", param.Skill.SkillId),
            ("技能名", param.Skill.SkillName),
            ("技能行为", skillBehaviorAction.ActionType)
        ];
        SkillUtils.Log(CombatLog.ELogType.Debug, ESkillLogType.SkillBehavior, entity, "SkillBehaviorAction.Begin", details);
        try
        {
            switch (skillBehaviorAction.ActionType)
            {
            case 0L:
                SetLocation(skillBehaviorAction, param);
                break;
            case 1L:
                SetRotation(skillBehaviorAction, param);
                break;
            case 2L:
                PlayEffect(skillBehaviorAction, param);
                break;
            case 3L:
                CreateBullet(skillBehaviorAction, param);
                break;
            case 4L:
                CameraModify(skillBehaviorAction, param);
                break;
            case 5L:
                CameraSequence(skillBehaviorAction, param);
                break;
            case 6L:
                SetMovementMode(skillBehaviorAction, param);
                break;
            case 7L:
                SetCollision(skillBehaviorAction, param);
                break;
            case 8L:
                UseSummonSkill(skillBehaviorAction, param);
                break;
            case 9L:
                Buff(skillBehaviorAction, param);
                break;
            case 10L:
                Tag(skillBehaviorAction, param);
                break;
            case 11L:
                ExitWeakTime(skillBehaviorAction, param);
                break;
            case 12L:
                PlaySkillMontage(skillBehaviorAction, param);
                break;
            case 13L:
                UpdateCustomValue(skillBehaviorAction, param);
                break;
            case 14L:
                BatchCreate(param.Entity, skillBehaviorAction.CommonConf, param.Skill.SkillId);
                break;
            }
        }
        catch (Exception e)
        {
            CombatLog instance = Singleton<CombatLog>.Instance;
            Entity entity2 = param.Entity;
            ReadOnlySpan<(string, object)> details2 =
            [
                ("技能Id", param.Skill.SkillId),
                ("技能名", param.Skill.SkillName)
            ];
            instance.ErrorWithStack(CombatLog.EDebugModule.Skill, entity2, "SkillBehaviorAction.Begin异常", e, details2);
        }
    }

    // Token 0x0601a121; original private static, non-virtual; canonical source lines 755-793.
    private static void CreateBullet(SSkillBehaviorAction action, IBeginSkillBehaviorActionParam param)
    {
        for (int i = 0; i < action.Bullets.Num(); i++)
        {
            SSkillBehaviorBullet sSkillBehaviorBullet = action.Bullets.Get(i);
            for (int j = 0; j < sSkillBehaviorBullet.bulletCount; j++)
            {
                int value = -1;
                if (param.Skill.SkillBehaviorAnimNotifyMessageId.HasValue)
                {
                    TsBaseCharacter actor = param.Entity.GetComponent<CharacterActorComponent>().Actor;
                    if (actor != null)
                    {
                        TsBaseCharacter owner = actor;
                        value = BulletUtil.CreateBulletFromAN(owner, sSkillBehaviorBullet.bulletRowName, param.Entity.GetComponent<CharacterActorComponent>().ActorTransform, param.Skill.SkillId, needSync: true, param.Skill.SkillBehaviorAnimNotifyMessageId.Value, param.Skill.ExtraTargetLocation);
                    }
                }
                else
                {
                    BulletEntity bulletEntity = ControllerBase<BulletController>.Instance.CreateBulletCustomTarget(param.Entity, sSkillBehaviorBullet.bulletRowName, param.Entity.GetComponent<CharacterActorComponent>().ActorTransform, new BulletController.BulletCreateParams
                    {
                        SkillId = param.Skill.SkillId,
                        SkillContextId = param.Skill.CombatMessageId.GetValueOrDefault(),
                        SyncType = EBulletSyncType.SyncCreate,
                        BattleContext = param.Skill.BattleContext,
                        InitTargetLocation = param.Skill.ExtraTargetLocation
                    }, param.Skill.CombatMessageId);
                    if (bulletEntity != null)
                    {
                        value = bulletEntity.Id;
                    }
                }
                if (!string.IsNullOrEmpty(sSkillBehaviorBullet.BlackboardKey))
                {
                    ControllerBase<BlackboardController>.Instance.SetIntValueByEntity(param.Entity.Id, sSkillBehaviorBullet.BlackboardKey, value);
                }
            }
        }
    }

    // R21 (2026-10-12): two appended whole source-level method bodies.

    // Token 0x0601a120; original private static, non-virtual; canonical source lines 733-753.
    private static void PlayEffect(SSkillBehaviorAction action, IBeginSkillBehaviorActionParam param)
    {
        CharacterGameplayCueComponent component = param.Entity.GetComponent<CharacterGameplayCueComponent>();
        for (int i = 0; i < action.Cues.Num(); i++)
        {
            SSkillBehaviorCue sSkillBehaviorCue = action.Cues.Get(i);
            int num = component.AddCue(sSkillBehaviorCue.CueId, new GameplayCueParam
            {
                Sync = true
            });
            if (sSkillBehaviorCue.Stop)
            {
                SkillBehaviorMisc.GetEndSkillBehaviorParamList(param.Skill).Add(new EndSkillBehaviorParam
                {
                    Entity = param.Entity,
                    ActionType = action.ActionType,
                    GameplayCue = num
                });
            }
        }
    }

    // The ReadOnlySpan collection expressions below represent lowered InlineArray3
    // logging scaffolding; they preserve observed tuple evaluation/order and are
    // not claims about original source syntax or ABI.
    // Token 0x0601a11b; original public static, non-virtual; canonical source lines 133-206.
    // EH: catch System.Exception (try 173..507; handler 507..650) and enumerator-only finally (try 26..664; handler 664..678).
    public static void End(Skill skill)
    {
        if (!SkillBehaviorMisc.ParamMap.TryGetValue(skill, out List<EndSkillBehaviorParam> value) || value == null)
        {
            return;
        }
        foreach (EndSkillBehaviorParam item in value)
        {
            Entity entity = item.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("技能Id", skill.SkillId),
                ("技能名", skill.SkillName),
                ("技能行为", item.ActionType)
            ];
            SkillUtils.Log(CombatLog.ELogType.Debug, ESkillLogType.SkillBehavior, entity, "SkillBehaviorAction.End", details);
            try
            {
                switch (item.ActionType)
                {
                case ESkillBehaviorActionType.播放特效:
                {
                    CharacterGameplayCueComponent component = item.Entity.GetComponent<CharacterGameplayCueComponent>();
                    if (item.GameplayCue.HasValue)
                    {
                        component.RemoveCueByHandle(item.GameplayCue.Value);
                    }
                    break;
                }
                case ESkillBehaviorActionType.设置移动状态:
                {
                    CharacterActorComponent component2 = item.Entity.GetComponent<CharacterActorComponent>();
                    if (item.MovementMode.HasValue)
                    {
                        component2.Actor.KuroSetMovementMode(new SetMovementModeInfo
                        {
                            Mode = item.MovementMode.Value,
                            Context = "[SkillBehaviorAction.End]"
                        });
                    }
                    break;
                }
                case ESkillBehaviorActionType.设置碰撞:
                {
                    UCapsuleComponent capsuleComponent = item.Entity.GetComponent<CharacterActorComponent>().Actor.CapsuleComponent;
                    if (capsuleComponent != null && item.CollisionChannel.HasValue && item.CollisionResponse.HasValue)
                    {
                        capsuleComponent.SetCollisionResponseToChannel(item.CollisionChannel.Value, item.CollisionResponse.Value);
                    }
                    break;
                }
                case ESkillBehaviorActionType.伴生物使用技能:
                    if (item.SummonSkillComponent != null && item.SummonSkillId.HasValue)
                    {
                        item.SummonSkillComponent.EndSkill(item.SummonSkillId.Value, "SkillBehaviorAction.End");
                    }
                    break;
                case ESkillBehaviorActionType.创建子弹:
                case ESkillBehaviorActionType.镜头效果:
                case ESkillBehaviorActionType.特写镜头:
                    break;
                }
            }
            catch (Exception e)
            {
                CombatLog instance = Singleton<CombatLog>.Instance;
                Entity entity2 = item.Entity;
                ReadOnlySpan<(string, object)> details2 =
                [
                    ("技能Id", skill.SkillId),
                    ("技能名", skill.SkillName),
                    ("技能行为", item.ActionType)
                ];
                instance.ErrorWithStack(CombatLog.EDebugModule.Skill, entity2, "SkillBehaviorAction.End异常", e, details2);
            }
        }
        SkillBehaviorMisc.ParamMap.Remove(skill);
    }
}
