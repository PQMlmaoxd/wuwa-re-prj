/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e65 (global::BaseDamageComponent; raw type row 27)
 * Scope: four selected whole source-level method bodies only.
 * The response delegate inside GetServerDamage is an associated generated
 * MethodDef, not an additional ordinary source-body row. Missing members and
 * downstream damage effects remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using Aki.Protocol.Summon;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;
using CSharpScript.Game.World.Model;
using CSharpScript.Utils;
using Google.Protobuf;
using UnrealEngine;

public partial class BaseDamageComponent : EntityComponent
{
    // Observed canonical field 0x0400bb3e; other members remain external.
    protected BaseTagComponent? TagComponent;

    // Token 0x06018648; original public, non-virtual; canonical source lines 113-195.
    public BulletDamageResult ExecuteBulletDamage(int bulletEntityId, IBulletDamageParam damageParam, long contextId)
    {
        DamageSnapshot damageSnapshot = ModelBase<DamageModel>.Instance?.GetDamageSnapshotById(damageParam.DamageDataId);
        BulletDamageResult bulletDamageResult = new BulletDamageResult();
        if (damageSnapshot == null)
        {
            return bulletDamageResult;
        }
        Damage config = damageSnapshot.Config;
        if (!string.IsNullOrEmpty(damageSnapshot.Condition))
        {
            BulletDamageContext bulletDamageContext = new BulletDamageContext
            {
                ContextType = EContextType.BulletDamage,
                DamageParam = damageParam,
                BulletEntityId = bulletEntityId,
                ContextId = contextId,
                Result = bulletDamageResult,
                Victim = this
            };
            ControllerBase<ExpressionTreeController>.Instance.DoDamageExpression(bulletDamageContext, config, damageParam.Attacker);
            return bulletDamageContext.Result;
        }
        BulletEntity bulletEntity = Singleton<EntitySystem>.Instance.Get<BulletEntity>(bulletEntityId);
        BulletInfo bulletInfo = bulletEntity.GetBulletInfo();
        if (damageSnapshot.CalculateType == 0 && TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.不受子弹伤害"]))
        {
            return bulletDamageResult;
        }
        RequirementPayload requirementPayload = new RequirementPayload();
        requirementPayload.BulletId = long.Parse(bulletInfo.BulletRowName);
        requirementPayload.BulletMessageId = contextId;
        requirementPayload.SkillId = bulletInfo.BulletInitParams.SkillId;
        requirementPayload.SkillMessageId = bulletInfo.BulletInitParams.SkillContextId;
        requirementPayload.SkillDamageCount = ModelBase<CombatMessageModel>.Instance.AddSkillDamageCount(bulletInfo.BulletInitParams.SkillContextId);
        requirementPayload.SkillDamageCountByVictim = ModelBase<CombatMessageModel>.Instance.AddSkillDamageCountByVictim(bulletInfo.BulletInitParams.SkillContextId, base.Entity.Id);
        requirementPayload.BulletDamageCount = ModelBase<CombatMessageModel>.Instance.AddBulletDamageCount(bulletInfo.ContextId);
        requirementPayload.BulletTags = bulletInfo.Tags;
        requirementPayload.BattleFlags = bulletInfo.BulletInitParams.BattleContext?.BattleFlags ?? new List<string>();
        requirementPayload.PartId = damageParam.PartId;
        if (requirementPayload.PartId >= 0)
        {
            requirementPayload.PartTag = base.Entity.GetComponent<CharacterPartComponent>().GetPartByIndex(requirementPayload.PartId.Value).PartTag?.TagId();
        }
        requirementPayload.CounterType = damageParam.CounterType;
        BaseDamageComponent baseDamageComponent = damageParam.Attacker.CheckGetComponent<BaseDamageComponent>();
        BaseBuffComponent attacker = damageParam.Attacker.CheckGetComponent<BaseBuffComponent>();
        if (baseDamageComponent == null)
        {
            CombatLog instance = Singleton<CombatLog>.Instance;
            Entity entity = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("damageId", config.Id),
                ("attacker id", damageParam.Attacker.Id)
            ];
            instance.Error(CombatLog.EDebugModule.Damage, entity, "伤害结算无合法施加者", details);
            return bulletDamageResult;
        }
        TDamageParam inputParam = new TDamageParam
        {
            DamageData = config,
            Attacker = baseDamageComponent,
            DirectTarget = damageParam.DirectTarget,
            SkillLevel = damageParam.SkillLevel,
            HitPosition = damageParam.HitPosition,
            IsAddEnergy = damageParam.IsAddEnergy,
            IsCounterAttack = damageParam.IsCounterAttack,
            ForceCritical = damageParam.ForceCritical,
            IsBlocked = damageParam.IsBlocked,
            ExtraRate = damageParam.ExtraRate,
            SourceType = DamageSourceType.FromBullet,
            Accumulation = ExtraEffectDamageAccumulation.GetAccumulation(bulletEntity.Id),
            Element = (EElementType)(((int?)ModifyDamageElement.ApplyEffects(attacker, damageSnapshot.Id)) ?? damageSnapshot.Element),
            PartId = damageParam.PartId,
            RandomSeed = ModelBase<PlayerInfoModel>.Instance.GetRandomSeed(),
            ContextId = contextId,
            SkillContextId = bulletInfo.BulletInitParams.SkillContextId,
            CounterSkillMessageId = damageParam.CounterSkillMessageId
        };
        ReviseInputSkillLevel(inputParam);
        float toughResult = ProcessDamage(requirementPayload, inputParam, bulletDamageResult);
        bulletDamageResult.ToughResult = toughResult;
        return bulletDamageResult;
    }

    // Token 0x0601864c; original public, non-virtual; canonical source lines 396-432.
    public float ProcessDamage(RequirementPayload partialRequirements, TDamageParam inputParam, BulletDamageResult? result)
    {
        if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["功能.逻辑状态标识.无敌"]) && inputParam.DamageData.ImmuneType == 0)
        {
            return 0f;
        }
        if (result != null)
        {
            result.IsAddEnergy = true;
        }
        BaseDamageComponent attacker = inputParam.Attacker;
        if (partialRequirements.SkillId > 0)
        {
            partialRequirements.SkillGenre = ((int?)(byte?)(attacker.SkillComponent?.GetSkill(partialRequirements.SkillId.Value))?.SkillInfo?.SkillGenre) ?? (-1);
        }
        partialRequirements.DamageId = inputParam.DamageData.Id;
        partialRequirements.DamageType = inputParam.DamageData.Type;
        partialRequirements.DamageSubTypes = inputParam.DamageData.GetSubTypeArray();
        partialRequirements.CalculateType = inputParam.DamageData.CalculateType;
        partialRequirements.SmashType = inputParam.DamageData.SmashType;
        partialRequirements.ElementType = inputParam.Element;
        SnapshotPayload snapshotPayload = AllocateSnapshot(attacker, inputParam);
        partialRequirements.WeaponType = snapshotPayload.Attacker.RoleGrowComponent?.GetWeaponType() ?? (-1);
        partialRequirements.SourceType = inputParam.SourceType;
        ProcessDamageOptimize(partialRequirements, inputParam, snapshotPayload);
        CharacterPart part = null;
        if (inputParam.PartId >= 0)
        {
            part = base.Entity.GetComponent<CharacterPartComponent>()?.GetPartByIndex(inputParam.PartId);
        }
        float extraToughRate = ((partialRequirements.SkillGenre.GetValueOrDefault() == 5) ? GetExtraToughRate("ToughRateOnCounter") : GetExtraToughRate("ToughRate")) / 10000f;
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.MonsterDebug, attacker.Entity.Id);
        GetServerDamage(inputParam, partialRequirements, inputParam.ContextId, part);
        float toughResult = CalculateToughReduce(inputParam, snapshotPayload, extraToughRate);
        toughResult = DamageModifier.ApplyEffects(partialRequirements, snapshotPayload, toughResult);
        ExecToughReduceRedirection(inputParam, snapshotPayload, toughResult);
        return toughResult;
    }

    // Token 0x06018652; original private, non-virtual; canonical source lines 563-660.
    // The inline response delegate is the associated 0x06049eb8 MethodDef; its
    // original captures and parent ldftn are pinned in METHOD_EVIDENCE_MAP.md.
    private void GetServerDamage(TDamageParam inputParam, RequirementPayload partialRequirements, long preContextId, CharacterPart? part)
    {
        TDamageParam inputParam2 = inputParam;
        RequirementPayload partialRequirements2 = partialRequirements;
        CharacterPart part2 = part;
        BaseDamageComponent attacker = inputParam2.Attacker;
        Damage damageData = inputParam2.DamageData;
        bool isBreakWeakness = partialRequirements2.SkillGenre.GetValueOrDefault() == 13 && TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["功能.逻辑状态标识.弱点机制.弱点状态"]);
        DamageExecuteRequest request = new DamageExecuteRequest();
        request.DamageId = Singleton<MathUtils>.Instance.NumberToLong(damageData.Id);
        request.SkillLevel = inputParam2.SkillLevel;
        request.AttackerEntityId = Singleton<MathUtils>.Instance.NumberToLong(attacker.Entity.GetComponent<CreatureDataComponent>().GetCreatureDataId());
        request.TargetEntityId = Singleton<MathUtils>.Instance.NumberToLong(inputParam2.DirectTarget.GetComponent<CreatureDataComponent>().GetCreatureDataId());
        request.IsAddEnergy = inputParam2.IsAddEnergy;
        request.IsCounterAttack = inputParam2.IsCounterAttack;
        request.ForceCritical = inputParam2.ForceCritical;
        request.IsBlocked = inputParam2.IsBlocked;
        request.PartId = inputParam2.PartId;
        request.CounterSkillMessageId = (inputParam2.CounterSkillMessageId.HasValue ? Singleton<MathUtils>.Instance.BigIntToLong(inputParam2.CounterSkillMessageId.Value) : 0);
        DamageContext damageContext = new DamageContext();
        request.DamageContext = damageContext;
        damageContext.SourceType = inputParam2.SourceType;
        damageContext.BulletId = partialRequirements2.BulletId.GetValueOrDefault(-1L);
        damageContext.BulletTags.AddRange((IEnumerable<int>)partialRequirements2.BulletTags);
        damageContext.SkillId = partialRequirements2.SkillId.GetValueOrDefault();
        if (inputParam2.SkillContextId.HasValue)
        {
            damageContext.SkillMessageId = inputParam2.SkillContextId.Value;
        }
        request.RandomSeed = ModelBase<PlayerInfoModel>.Instance.AdvanceRandomSeed(ERandomReason.StartDamage);
        request.IsBreakWeakness = isBreakWeakness;
        bool shouldLogOnline = ModelBase<GameModeModel>.Instance.IsMulti && damageData.Id == 1505600001;
        if (shouldLogOnline)
        {
            CombatLog instance = Singleton<CombatLog>.Instance;
            Entity entity = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("攻击方", Singleton<MathUtils>.Instance.BigIntToLong(request.AttackerEntityId)),
                ("受击方", Singleton<MathUtils>.Instance.BigIntToLong(request.TargetEntityId)),
                ("结算id", damageData.Id),
                ("BulletId", partialRequirements2.BulletId.GetValueOrDefault())
            ];
            instance.Info(CombatLog.EDebugModule.Damage, entity, "发起结算请求", details);
        }
        Singleton<CombatNet>.Instance.Call(ERequestMessageId.DamageExecuteRequest, base.Entity, (IMessage)(object)request, delegate(DamageExecuteResponse response)
        {
            if (response.ImmuneType == EDamageImmune.Invincible)
            {
                if (shouldLogOnline)
                {
                    CombatLog instance2 = Singleton<CombatLog>.Instance;
                    Entity entity2 = base.Entity;
                    ReadOnlySpan<(string, object)> details2 =
                    [
                        ("攻击方", request.AttackerEntityId),
                        ("受击方", request.TargetEntityId),
                        ("结算id", damageData.Id)
                    ];
                    instance2.Info(CombatLog.EDebugModule.Damage, entity2, "EDamageImmune_Invincible", details2);
                }
            }
            else
            {
                DamageResult damageResult = new DamageResult
                {
                    Damage = -response.Damage,
                    ChangeLife = response.ChangeLife,
                    ShieldCoverDamage = response.ShieldCoverDamage,
                    IsCritical = response.IsCrit,
                    IsTargetKilled = response.KilledTarget,
                    KillerEntityId = 0L,
                    IsImmune = (response.ImmuneType == EDamageImmune.BuffEffectElement),
                    Element = (EElementType)response.ElementType,
                    DamageData = inputParam2.DamageData,
                    IsCounterAttack = inputParam2.IsCounterAttack,
                    IsBlocked = inputParam2.IsBlocked,
                    SourceType = inputParam2.SourceType
                };
                partialRequirements2.IsCritical = damageResult.IsCritical;
                partialRequirements2.IsImmune = damageResult.IsImmune;
                partialRequirements2.IsTargetKilled = damageResult.IsTargetKilled;
                partialRequirements2.ChangeWeaknessType = (EChangeWeaknessType)response.ChangeWeakness;
                if (shouldLogOnline)
                {
                    CombatLog instance3 = Singleton<CombatLog>.Instance;
                    Entity entity3 = base.Entity;
                    ReadOnlySpan<(string, object)> details3 =
                    [
                        ("攻击方", request.AttackerEntityId),
                        ("受击方", request.TargetEntityId),
                        ("结算id", damageData.Id),
                        ("伤害值", damageResult.Damage),
                        ("errorCode", response.ErrorCode)
                    ];
                    instance3.Info(CombatLog.EDebugModule.Damage, entity3, "收到结算回包", details3);
                }
                if (response.ErrorCode == Aki.Protocol.ErrorCode.Success)
                {
                    GetServerDamageCallBack(partialRequirements2, inputParam2, damageResult, part2);
                }
            }
        }, preContextId);
    }

    // Token 0x06018653; original private, non-virtual; canonical source lines 662-675.
    private void GetServerDamageCallBack(RequirementPayload requirements, TDamageParam inputParam, DamageResult serverResult, CharacterPart? part)
    {
        Entity entity = base.Entity;
        if (entity != null && entity.Valid)
        {
            BaseDamageComponent? attacker = inputParam.Attacker;
            if (attacker != null && (attacker.Entity?.Valid).GetValueOrDefault())
            {
                ExecDamageEvent(serverResult, inputParam, requirements);
                ExecDamageEffect(serverResult, inputParam, requirements);
                part?.OnDamage(serverResult.Damage, inputParam.ForceCritical, inputParam.Attacker.Entity, deductLife: false);
            }
        }
    }
}

/*
 * R18 declaration header above applies to the preserved first declaration only.
 * R26 adds six whole ordinary source-level method bodies and three inline
 * associated generated callbacks; the callbacks are not additional ordinary
 * rows. The original class was not partial; `partial` here is source-view
 * convenience only and is not original source structure.
 */
public partial class BaseDamageComponent : EntityComponent
{
    // Canonical field 0x0400bb3d; retained only because the selected R26 bodies use it.
    protected BaseAttributeComponent? AttributeComponent;

    // Canonical field 0x0400bb3f; retained only because the selected R26 bodies use it.
    protected CharacterBuffComponent? BuffComponent;

    // Canonical field 0x0400bb44; retained only because the selected R26 bodies use it.
    protected CreatureDataComponent? CreatureDataComponent;

    // Canonical field 0x0400bb45; retained only because the selected R26 bodies use it.
    private ITagTask? HitInAirFinishTask;

    // Canonical field 0x0400bb46; retained only because the selected R26 bodies use it.
    private TimerHandle? WeakTimer;

    // Token 0x06018658; original private instance, non-virtual; canonical source lines 763-769.
    private float CalculateToughReduce(TDamageParam inputParam, SnapshotPayload snapshots, float extraToughRate = 1f)
    {
        AttributeSnapshot? attackerSnapshot = snapshots.AttackerSnapshot;
        AttributeSnapshot targetSnapshot = snapshots.TargetSnapshot;
        int levelValue = AbilityUtils.GetLevelValue(inputParam.DamageData.ToughLv(), inputParam.SkillLevel, 0);
        return Calculation.ToughCalculation(attackerSnapshot, targetSnapshot, (float)levelValue * extraToughRate);
    }

    // Token 0x06018659; original private instance, non-virtual; canonical source lines 771-783.
    // EH: generated foreach dispose finally, try 71..124 and handler 124..138.
    private void ExecToughReduceRedirection(TDamageParam inputParam, SnapshotPayload snapshots, double toughResult)
    {
        if (!snapshots.HasDamageTransfer)
        {
            _ = snapshots.TargetSnapshot.CurrentValues;
            ExecToughReduce(inputParam, snapshots.TargetSnapshot.GetCurrentValue(EAttributeType.ToughRecoverDelayTime), snapshots.TargetSnapshot.GetCurrentValue(EAttributeType.WeakTime), (float)toughResult);
            return;
        }
        foreach (DamageTransfer damageTransfer in snapshots.DamageTransfers)
        {
            damageTransfer.TransferTarget?.ExecToughReduce(inputParam, damageTransfer.ToughRecoverDelayTime, damageTransfer.WeakTime, (float)toughResult);
        }
    }

    // Token 0x0601865a; original private instance, non-virtual; canonical source lines 785-839.
    private void ExecToughReduce(TDamageParam inputParam, float toughRecoverDelayTime, float weakTime, float toughResult)
    {
        BaseDamageComponent attacker = inputParam.Attacker;
        if (toughResult != 0f)
        {
            float num = 1f;
            CreatureDataComponent component = base.Entity.GetComponent<CreatureDataComponent>();
            if (ModelBase<GameModeModel>.Instance.IsMulti && component.IsMonster())
            {
                int currentTeamSize = ModelBase<OnlineModel>.Instance.GetCurrentTeamSize();
                if (currentTeamSize > 1)
                {
                    num = ((currentTeamSize > 2) ? ConfigCommonParamById.GetFloatConfig("MutiWorldToughRatio3").Value : ConfigCommonParamById.GetFloatConfig("MutiWorldToughRatio2").Value);
                }
            }
            // CIL uses neg followed by mul; do not rewrite this as 0f - toughResult.
            AttributeComponent.AddBaseValue(EAttributeType.Tough, -toughResult * num);
        }
        if (AttributeComponent.GetCurrentValue(EAttributeType.Tough) > 0f)
        {
            if (toughRecoverDelayTime > 0f && toughResult != 0f)
            {
                BuffComponent.AddBuff(3024L, new AddBuffParam
                {
                    InstigatorId = (attacker.CreatureDataComponent?.GetCreatureDataId() ?? (-1)),
                    ApplyType = ApplyGEType.UseExtraTime,
                    Reason = "韧性扣减后触发"
                });
            }
        }
        else
        {
            if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["Damage.Vulnerable"]) || !(weakTime > 0f))
            {
                return;
            }
            RequestWeakTime(isStart: true);
            float currentValue = AttributeComponent.GetCurrentValue(EAttributeType.WeakTime);
            // Inline associated generated MethodDef 0x06018666; raw line 742, RVA 7163138, 59 B, EH0; parent ldftn offset 312.
            WeakTimer = TimerSystem.Instance.Delay(delegate
            {
                if (TagComponent != null)
                {
                    if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.动作状态.受击.击飞"]))
                    {
                        ListenForHitInAirFinishTask();
                    }
                    else
                    {
                        RequestWeakTime(isStart: false);
                    }
                }
                WeakTimer = null;
            }, currentValue);
            AttributeComponent.SetBaseValue(EAttributeType.ToughRecover, 0f);
        }
    }

    // Token 0x0601865b; original private instance, non-virtual; canonical source lines 841-868.
    private void RequestWeakTime(bool isStart)
    {
        if (isStart)
        {
            TagComponent.AddTag(GameplayTagDefine.EGameplayTagId["Damage.Vulnerable"]);
        }
        else
        {
            TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["Damage.Vulnerable"]);
            HitInAirFinishTask?.EndTask();
        }
        FragileChangeRequest fragileChangeRequest = FragileChangeRequest.Create();
        fragileChangeRequest.EntityId = base.Entity.GetComponent<CreatureDataComponent>().GetCreatureDataId();
        fragileChangeRequest.Flag = isStart;
        Singleton<CombatNet>.Instance.Call<FragileChangeResponse>(ERequestMessageId.FragileChangeRequest, base.Entity, (IMessage)(object)fragileChangeRequest, Response);

        // Inline associated generated MethodDef 0x06018667; raw line 743, RVA 7163200, 73 B, EH0; parent ldftn offset 138.
        void Response(FragileChangeResponse response)
        {
            if (response.ErrorCode != 0)
            {
                if (WeakTimer != null)
                {
                    TimerSystem.Instance.Remove(WeakTimer);
                    WeakTimer = null;
                }
                TagComponent.RemoveTag(GameplayTagDefine.EGameplayTagId["Damage.Vulnerable"]);
            }
        }
    }

    // Token 0x0601865c; original private instance, non-virtual; canonical source lines 870-879.
    private void ListenForHitInAirFinishTask()
    {
        HitInAirFinishTask = TagComponent.ListenForTagAddOrRemove(GameplayTagDefine.EGameplayTagId["行为状态.动作状态.受击.击飞"], delegate(int tagId, bool tagExists)
        {
            if (!tagExists)
            {
                RequestWeakTime(isStart: false);
            }
        });
    }

    // Inline associated generated MethodDef 0x06018668; raw line 744, RVA 7163285, 11 B, EH0; parent ldftn offset 28.

    // Token 0x0601865d; original public instance, non-virtual; canonical source lines 881-892.
    public void TryExitWeakTime()
    {
        if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["Damage.Vulnerable"]))
        {
            if (WeakTimer != null)
            {
                TimerSystem.Instance.Remove(WeakTimer);
                WeakTimer = null;
            }
            RequestWeakTime(isStart: false);
        }
    }
}