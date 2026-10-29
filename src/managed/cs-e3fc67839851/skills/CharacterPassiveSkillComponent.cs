/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002ec6 (global::CharacterPassiveSkillComponent)
 * Scope: nine selected whole method bodies only.
 * Unknown dependencies and unselected lifecycle members remain external;
 * see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using CSharpScript.Utils;
using UnrealEngine;

public partial class CharacterPassiveSkillComponent : EntityComponent
{
    // Observed members used by the selected view.
    private CharacterTriggerComponent triggerComp;
    private CharacterBuffComponent buffComp;
    private BaseSkillCdComponent? cdComp;
    private CharacterActorComponent actorComp;
    private readonly Dictionary<long, PassiveSkillData> SkillMap = new Dictionary<long, PassiveSkillData>();
    private ContextParam Context;
    private readonly HashSet<long> LockMap = new HashSet<long>();

    // The original external OnTick clears LockMap; it is not selected here.
    // OnPassiveSkillAdded and GetPassiveSkillConfig are original external
    // members, deliberately not restated in this source view.

    // Token 0x0601896b; original private, non-virtual.
    private bool IsPassiveSkillInCd(long skillId, int cdEntityId)
    {
        return cdComp?.IsPassiveSkillInCd(skillId, cdEntityId) ?? false;
    }

    // Token 0x0601896c; original private, non-virtual.
    private void StartPassiveCd(long skillId, int cdEntityId)
    {
        cdComp?.StartPassiveCd(skillId, cdEntityId);
    }

    // Token 0x0601896e; original private, non-virtual.
    private Entity? GetEntityByKey(long skillId, string targetKey, Dictionary<string, TFormulaValue>? parameters, Dictionary<string, TFormulaValue>? extraParams)
    {
        object target = base.Entity;
        if (!string.IsNullOrEmpty(targetKey))
        {
            TFormulaValue extraValue;
            if (parameters != null && parameters.TryGetValue(targetKey, out TFormulaValue parameterValue))
            {
                target = parameterValue.TryGetEntity(out Entity parameterEntity) ? parameterEntity : null;
            }
            else if (extraParams != null && extraParams.TryGetValue(targetKey, out extraValue))
            {
                target = extraValue.TryGetEntity(out Entity extraEntity) ? extraEntity : null;
            }
        }

        if (target is Entity entity)
        {
            return entity;
        }

        CombatLog logger = Singleton<CombatLog>.Instance;
        Entity owner = base.Entity;
        ReadOnlySpan<(string, object)> details =
        [
            ("targetKey", targetKey),
            ("skillId", skillId)
        ];
        logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能Entity非法", details);
        return null;
    }

    // Token 0x0601896f; original private, non-virtual.
    private int GetCdEntityId(PassiveSkill config, Entity cdEntity)
    {
        if (!config.IsShareAllCdSkill || config.CDType != "Owner")
        {
            return cdEntity.Id;
        }
        return 0;
    }

    // Token 0x06018972; original public, non-virtual.
    public bool CheckSkillHasAuthority(PassiveSkill config)
    {
        if (buffComp.HasBuffAuthority())
        {
            return true;
        }
        if (Convert.ToInt32(config.ActionExecuteFlag) == 2 && actorComp.IsAutonomousProxy)
        {
            return true;
        }
        return false;
    }

    // Token 0x06018973; original public, non-virtual.
    public bool LearnPassiveSkill(long skillId, AddPassiveSkillParam param)
    {
        PassiveSkill? config = GetPassiveSkillConfig(skillId);
        if (!config.HasValue)
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details = [("skillId", skillId)];
            logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能配置不存在", details);
            return false;
        }

        if (HasSkill(skillId) || !CheckAddSkillHasAuthority(config.Value))
        {
            return false;
        }

        if (Convert.ToInt32(config.Value.ActionExecuteFlag) == 1)
        {
            if (param.NeedBroadcast.GetValueOrDefault())
            {
                ControllerBase<SkillMessageController>.Instance.PassiveSkillAddRequest(base.Entity, skillId, param.PreMessageId);
            }
            return true;
        }

        if (string.IsNullOrEmpty(config.Value.TriggerType) || !Enum.TryParse<ETriggerEvent>(config.Value.TriggerType, out ETriggerEvent triggerType))
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details = [("skillId", skillId)];
            logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能配置错误，缺少触发类型", details);
            return false;
        }

        if (triggerType == ETriggerEvent.GlobalDamageTrigger && !IsWhitelistSkill(skillId))
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details = [("skillId", skillId)];
            logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "禁止白名单之外的被动使用全局伤害监听", details);
            return false;
        }

        Log infoLogger = Singleton<Log>.Instance;
        ReadOnlySpan<(string, object)> infoDetails =
        [
            ("owner", base.Entity.Id),
            ("skillId", skillId)
        ];
        infoLogger.Info((ELogModule)35, (ELogAuthor)20, "角色添加被动技能", infoDetails);

        // The two inline delegates are the readable association for nested
        // TypeDef 0x0200931f methods 0x06049fb0 and 0x06049fb1. The closure
        // fields capture this component, skillId, and config; they are not
        // counted as standalone selected methods.
        int triggerHandle = triggerComp.AddTrigger(new TriggerConfigData
        {
            Type = config.Value.TriggerType,
            Preset = config.Value.TriggerPreset(),
            Params = config.Value.TriggerParams,
            Formula = config.Value.TriggerFormula,
            ExecuteType = (EExecuteType)Convert.ToInt32(config.Value.ExecuteType)
        }, delegate(Dictionary<string, TFormulaValue>? parameters, Dictionary<string, TFormulaValue>? extraParams)
        {
            Entity instigatorEntity = GetEntityByKey(skillId, config.Value.InstigatorType, parameters, extraParams);
            if (instigatorEntity != null)
            {
                Entity cooldownEntity = GetEntityByKey(skillId, config.Value.CDType, parameters, extraParams);
                if (cooldownEntity != null)
                {
                    int cooldownEntityId = GetCdEntityId(config.Value, cooldownEntity);
                    if (CheckSkillHasAuthority(config.Value) && !IsPassiveSkillInCd(skillId, cooldownEntityId))
                    {
                        ExecuteAction(skillId, instigatorEntity, cooldownEntityId, extraParams);
                    }
                }
            }
        }, delegate
        {
            if (config.Value.CDType == "Owner")
            {
                int cooldownEntityId = GetCdEntityId(config.Value, base.Entity);
                if (IsPassiveSkillInCd(skillId, cooldownEntityId))
                {
                    return false;
                }
            }
            return CheckSkillHasAuthority(config.Value);
        });

        // AddTrigger's -1 result is stored without a pre-check, as observed.
        SkillMap[skillId] = new PassiveSkillData
        {
            SkillId = skillId,
            TriggerHandle = triggerHandle,
            Actions = ParseActions(config.Value, skillId),
            TargetKey = config.Value.InstigatorType,
            CombatMessageId = param.CombatMessageId
        };
        // External original member; its activation/reset behavior is not part
        // of this selected source view.
        OnPassiveSkillAdded(skillId, triggerHandle, config.Value, param);
        return true;
    }

    // Token 0x06018976; original protected, non-virtual.
    protected List<SkillActionData> ParseActions(PassiveSkill config, long skillId)
    {
        List<SkillActionData> actions = new List<SkillActionData>();
        SkillActionData firstAction = ParseAction(config, skillId);
        if (firstAction != null)
        {
            actions.Add(firstAction);
        }
        foreach (long subSkillId in config.SubSkillActionIter())
        {
            PassiveSkill? subSkillConfig = GetPassiveSkillConfig((int)subSkillId);
            if (subSkillConfig.HasValue)
            {
                SkillActionData subAction = ParseAction(subSkillConfig.Value, skillId);
                if (subAction != null)
                {
                    actions.Add(subAction);
                }
            }
        }
        return actions;
    }

    // Token 0x06018977; original protected, non-virtual.
    protected SkillActionData? ParseAction(PassiveSkill config, long skillId)
    {
        if (!Enum.TryParse<ESkillAction>(Convert.ToString(config.SkillAction), out ESkillAction action))
        {
            return null;
        }

        string[] parameters = config.SkillActionParams() ?? Array.Empty<string>();
        SkillActionData actionData = new SkillActionData
        {
            Action = action
        };
        switch (action)
        {
            case ESkillAction.AddBullet:
            {
                actionData.BulletRowNames = new string[parameters.Length];
                for (int i = 0; i < parameters.Length; i++)
                {
                    actionData.BulletRowNames[i] = parameters[i].Trim();
                }
                return actionData;
            }

            case ESkillAction.RemoveBullet:
            {
                actionData.BulletRowNames = new string[parameters.Length];
                actionData.SummonChild = new bool[parameters.Length];
                for (int i = 0; i < parameters.Length; i++)
                {
                    string[] split = parameters[i].Split('#');
                    actionData.BulletRowNames[i] = split.Length != 0 ? split[0].Trim() : string.Empty;
                    actionData.SummonChild[i] = split.Length > 1 && split[1].Trim() == "1";
                }
                return actionData;
            }

            case ESkillAction.AddBuff:
            {
                actionData.BuffId = new long[parameters.Length];
                for (int i = 0; i < parameters.Length; i++)
                {
                    actionData.BuffId[i] = Convert.ToInt64(parameters[i]);
                }
                return actionData;
            }

            case ESkillAction.RemoveBuff:
            {
                actionData.BuffId = new long[parameters.Length];
                actionData.StackCount = new int[parameters.Length];
                for (int i = 0; i < parameters.Length; i++)
                {
                    string[] split = parameters[i].Split('#');
                    actionData.BuffId[i] = split.Length != 0 ? Convert.ToInt64(split[0]) : 0;
                    actionData.StackCount[i] = split.Length > 1 ? Convert.ToInt32(split[1]) : -1;
                }
                return actionData;
            }

            case ESkillAction.StartSkill:
                actionData.SkillId = parameters.Length != 0 ? Convert.ToInt32(parameters[0]) : 0;
                return actionData;

            case ESkillAction.LockOn:
                actionData.IsHardLock = parameters.Length != 0 && parameters[0] == "1";
                actionData.LockOnConfigId = parameters.Length > 1 ? Convert.ToInt32(parameters[1]) : 0;
                actionData.SkillTargetPriority = parameters.Length > 2
                    ? (ESkillTargetPriority)Convert.ToInt32(parameters[2])
                    : ESkillTargetPriority.系统设置;
                actionData.ShowTarget = parameters.Length <= 3 || parameters[3] == "1";
                actionData.GlobalTarget = parameters.Length > 4 && parameters[4] == "1";
                return actionData;

            case ESkillAction.Customize:
                actionData.Formula = CharacterPassiveSkillCustomAction.AddCustomAction(config, base.Entity, triggerComp.TriggerFormulaFunc);
                return actionData;

            case ESkillAction.ReduceSkillCd:
            {
                actionData.ReduceSkillId = new long[parameters.Length];
                actionData.CdResetType = new ESkillCdReduceType[parameters.Length];
                actionData.DecreaseMagnitude = new float[parameters.Length];
                actionData.DecreaseRatio = new float[parameters.Length];
                for (int i = 0; i < parameters.Length; i++)
                {
                    string[] split = parameters[i].Split('#');
                    actionData.ReduceSkillId[i] = split.Length != 0 ? Convert.ToInt64(split[0]) : 0;
                    actionData.CdResetType[i] = split.Length > 1
                        ? (ESkillCdReduceType)Convert.ToInt32(split[1])
                        : ESkillCdReduceType.SpecifiedSkillId;
                    actionData.DecreaseMagnitude[i] = split.Length > 2 ? Convert.ToSingle(split[2]) : 0f;
                    actionData.DecreaseRatio[i] = split.Length > 3 ? Convert.ToSingle(split[3]) : 0f;
                }
                return actionData;
            }

            default:
                return null;
        }
    }

    // Token 0x06018978; original protected, non-virtual.
    protected void ExecuteAction(long skillId, Entity targetEntity, int passiveCdEntityId, Dictionary<string, TFormulaValue>? extraParams)
    {
        // LockMap is method-wide bookkeeping. There is no method-wide cleanup
        // finally in the selected body; OnTick is an external reset path.
        if (LockMap.Contains(skillId))
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("skillId", skillId),
                ("desc", GetPassiveSkillConfig(skillId)?.SkillDesc ?? string.Empty),
                ("current executing skill ids", new List<long>(LockMap))
            ];
            logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能在同一次调用栈中重复触发，需要检查技能配置", details);
        }
        else
        {
            if (!SkillMap.TryGetValue(skillId, out PassiveSkillData skillData))
            {
                return;
            }

            // Cooldown starts before action dispatch; lock insertion follows it.
            StartPassiveCd(skillId, passiveCdEntityId);
            LockMap.Add(skillId);
            for (int i = 0; i < skillData.Actions.Count; i++)
            {
                SkillActionData action = skillData.Actions[i];
                switch (action.Action)
                {
                    case ESkillAction.AddBullet:
                    {
                        FTransformDouble? initialTransform = targetEntity.GetComponent<BaseActorComponent>()?.ActorTransform;
                        if (!initialTransform.HasValue)
                        {
                            CombatLog logger = Singleton<CombatLog>.Instance;
                            Entity owner = base.Entity;
                            ReadOnlySpan<(string, object)> details =
                            [
                                ("skillId", skillId),
                                ("targetEntity", targetEntity.Id)
                            ];
                            logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能目标没有ActorTransform", details);
                        }
                        else
                        {
                            for (int iBullet = 0; iBullet < action.BulletRowNames.Length; iBullet++)
                            {
                                ControllerBase<BulletController>.Instance.CreateBulletCustomTarget(base.Entity, action.BulletRowNames[iBullet], initialTransform, null, skillData.CombatMessageId);
                            }
                        }
                        break;
                    }

                    case ESkillAction.RemoveBullet:
                    {
                        IReadOnlyCollection<BulletEntity> bullets = ModelBase<BulletModel>.Instance.GetBulletSetByAttacker(base.Entity.Id);
                        if (bullets == null)
                        {
                            break;
                        }
                        List<(long, bool)> toDestroy = new List<(long, bool)>();
                        foreach (BulletEntity bullet in bullets)
                        {
                            BulletInfo bulletInfo = bullet.GetBulletInfo();
                            for (int iBullet = 0; iBullet < action.BulletRowNames.Length; iBullet++)
                            {
                                if (action.BulletRowNames[iBullet] == bulletInfo.BulletRowName)
                                {
                                    bool summonChild = iBullet < action.SummonChild.Length && action.SummonChild[iBullet];
                                    toDestroy.Add((bulletInfo.BulletEntityId, summonChild));
                                    break;
                                }
                            }
                        }
                        for (int iBullet = 0; iBullet < toDestroy.Count; iBullet++)
                        {
                            ControllerBase<BulletController>.Instance.DestroyBullet(Convert.ToInt32(toDestroy[iBullet].Item1), toDestroy[iBullet].Item2, EBulletDestroyReason.FromPassiveSkill);
                        }
                        break;
                    }

                    case ESkillAction.AddBuff:
                    {
                        CharacterBuffComponent targetBuffComponent = targetEntity.GetComponent<CharacterBuffComponent>();
                        string reason = $"被动技能{skillId}添加";
                        for (int iBuff = 0; iBuff < action.BuffId.Length; iBuff++)
                        {
                            targetBuffComponent?.AddBuff(action.BuffId[iBuff], new AddBuffParam
                            {
                                InstigatorId = buffComp.CreatureDataId,
                                PreMessageId = skillData.CombatMessageId,
                                Reason = reason
                            });
                        }
                        break;
                    }

                    case ESkillAction.RemoveBuff:
                    {
                        CharacterBuffComponent targetBuffComponent = targetEntity.GetComponent<CharacterBuffComponent>();
                        string reason = $"被动技能{skillId}移除";
                        for (int iBuff = 0; iBuff < action.BuffId.Length; iBuff++)
                        {
                            int stackCount = iBuff < action.StackCount.Length ? action.StackCount[iBuff] : -1;
                            targetBuffComponent?.RemoveBuff(action.BuffId[iBuff], stackCount, reason);
                        }
                        break;
                    }

                    case ESkillAction.StartSkill:
                        targetEntity.GetComponent<CharacterSkillComponent>()?.BeginSkillAsync(action.SkillId, new SkillParam
                        {
                            ContextId = skillData.CombatMessageId,
                            Reason = "PassiveSkillComponent.ExecuteAction"
                        });
                        break;

                    case ESkillAction.LockOn:
                    {
                        CharacterLockOnComponent lockOnComponent = targetEntity.GetComponent<CharacterLockOnComponent>();
                        CharacterSkillComponent skillComponent = targetEntity.GetComponent<CharacterSkillComponent>();
                        if (action.IsHardLock)
                        {
                            lockOnComponent?.EnterLockDirection();
                            break;
                        }
                        skillComponent?.LockOnTargetAndSetShow(new SSkillTarget
                        {
                            LockOnConfigId = action.LockOnConfigId,
                            SkillTargetPriority = action.SkillTargetPriority,
                            ShowTarget = action.ShowTarget,
                            GlobalTarget = action.GlobalTarget
                        });
                        break;
                    }

                    case ESkillAction.Customize:
                        Context.SkillId = skillId;
                        action.Formula?.Evaluate(extraParams, Context);
                        break;

                    case ESkillAction.ReduceSkillCd:
                    {
                        CharacterSkillCdComponent skillCdComponent = targetEntity.GetComponent<CharacterSkillCdComponent>();
                        if (skillCdComponent == null)
                        {
                            break;
                        }
                        for (int iCd = 0; iCd < action.ReduceSkillId.Length; iCd++)
                        {
                            float magnitude = iCd < action.DecreaseMagnitude.Length ? action.DecreaseMagnitude[iCd] : 0f;
                            float changeTimePercentage = (0f - (iCd < action.DecreaseRatio.Length ? action.DecreaseRatio[iCd] : 0f)) * 0.0001f;
                            float modifyTime = 0f - magnitude;
                            if (iCd < action.CdResetType.Length && action.CdResetType[iCd] == ESkillCdReduceType.SpecifiedSkillId)
                            {
                                skillCdComponent.ModifyCdTime(new long[1] { action.ReduceSkillId[iCd] }, modifyTime, changeTimePercentage);
                            }
                            else if (iCd < action.CdResetType.Length && action.CdResetType[iCd] == ESkillCdReduceType.SpecifiedSkillGenre)
                            {
                                skillCdComponent.ModifyCdTimeBySkillGenres(new int[1] { Convert.ToInt32(action.ReduceSkillId[iCd]) }, modifyTime, changeTimePercentage);
                            }
                        }
                        break;
                    }
                }
            }
            LockMap.Remove(skillId);
        }
    }
}
