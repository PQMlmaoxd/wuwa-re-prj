/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002db9 (global::BulletController)
 * Scope: seven selected whole source-level method bodies only.
 * The nested BulletCreateParams type and all unselected members remain external;
 * see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using AkiClient.Game.Aki.Core.Fight;
using CSharpScript.Core.Common;
using CSharpScript.Core.Framework;
using CSharpScript.Game;
using CSharpScript.Game.Common.Event;
using Google.Protobuf;
using UnrealEngine;

[TickController(0)]
public partial class BulletController : ControllerBase<BulletController>
{
    // Token 0x060179a1; original private, non-virtual; canonical source lines 160-165.
    private void OnAddEvents()
    {
        Singleton<EventSystem>.Instance.Add<int, int>(EEventName.OnSkillEnd, OnSkillEnd);
        Singleton<EventSystem>.Instance.Add<ERemoveEntityType, EntityHandle>(EEventName.RemoveEntity, OnRemoveEntity);
        Singleton<EventSystem>.Instance.Add(EEventName.SetNiagaraQuality, OnSetNiagaraQuality);
    }

    // Token 0x060179a2; original private, non-virtual; canonical source lines 167-172.
    private void OnRemoveEvents()
    {
        Singleton<EventSystem>.Instance.Remove<int, int>(EEventName.OnSkillEnd, OnSkillEnd);
        Singleton<EventSystem>.Instance.Remove<ERemoveEntityType, EntityHandle>(EEventName.RemoveEntity, OnRemoveEntity);
        Singleton<EventSystem>.Instance.Remove(EEventName.SetNiagaraQuality, OnSetNiagaraQuality);
    }

    // Token 0x060179a3; original private, non-virtual; canonical source lines 174-207.
    // EH: enumerator-only finally regions (try 33..129; handler 129..139; try 147..198; handler 198..212).
    private void OnSkillEnd(int entityId, int skillId)
    {
        if (skillId == 0)
        {
            return;
        }
        IReadOnlyCollection<BulletEntity> bulletSetByAttacker = ModelBase<BulletModel>.Instance.GetBulletSetByAttacker(entityId);
        if (bulletSetByAttacker == null)
        {
            return;
        }
        List<BulletEntity> list = new List<BulletEntity>();
        foreach (BulletEntity item in bulletSetByAttacker)
        {
            BulletInfo bulletInfo = item.GetBulletInfo();
            if (skillId == bulletInfo.BulletInitParams.SkillId)
            {
                BulletDataMain? bulletDataMain = bulletInfo.BulletDataMain;
                if (bulletDataMain.Move.IsDetachOnSkillEnd)
                {
                    bulletInfo.Actor.K2_DetachFromActor(EDetachmentRule.KeepWorld, EDetachmentRule.KeepWorld, EDetachmentRule.KeepWorld);
                }
                if (bulletDataMain.Base.DestroyOnSkillEnd)
                {
                    list.Add(item);
                }
            }
        }
        foreach (BulletEntity item2 in list)
        {
            item2.GetBulletInfo().IsDestroyByCharSkillEnd = true;
            DestroyBullet(item2.Id, summonChild: false);
        }
    }

    // Token 0x060179aa; original public, non-virtual; canonical source lines 244-247.
    public BulletEntity? CreateBulletCustomTarget(Entity? owner, string bulletRowName, FTransformDouble? initialTransform, BulletCreateParams? bulletCreateParams = null, long? contextId = null, EBulletCreateSource createSource = EBulletCreateSource.Others)
    {
        return CreateBulletCustomTargetInternal(owner, bulletRowName, initialTransform, bulletCreateParams, contextId, createSource);
    }

    // Token 0x060179ac; original private, non-virtual; canonical source lines 255-325.
    private BulletEntity? CreateBulletCustomTargetInternal(Entity? owner, string bulletRowName, FTransformDouble? initialTransform, BulletCreateParams? bulletCreateParams, long? contextId, EBulletCreateSource createSource)
    {
        if (bulletCreateParams == null)
        {
            bulletCreateParams = new BulletCreateParams();
        }
        if (!ModelBase<GameModeModel>.Instance.WorldDone)
        {
            return null;
        }
        CreateBulletStatStart(bulletRowName);
        if (owner == null)
        {
            Log instance = Singleton<Log>.Instance;
            (string, object) reference = ("rowName", bulletRowName);
            instance.Error((ELogModule)71, (ELogAuthor)21, "创建子弹时Owner为空", new ReadOnlySpan<(string, object)>(ref reference));
        }
        if (!contextId.HasValue && !SpecialIgnoreBullet.CheckBulletInSpecialList(bulletRowName))
        {
            CombatLog instance2 = Singleton<CombatLog>.Instance;
            (string, object) reference2 = ("rowName", bulletRowName);
            instance2.Error(CombatLog.EDebugModule.Bullet, null, "创建子弹时contextId为空", new ReadOnlySpan<(string, object)>(ref reference2));
            CreateBulletStatSop(bulletRowName);
            return null;
        }
        BulletDataMain bulletData = ConfigBase<BulletConfig>.Instance.GetBulletData(owner, bulletRowName);
        if (bulletData == null)
        {
            CreateBulletStatSop(bulletRowName);
            return null;
        }
        if (createSource == EBulletCreateSource.Skill && bulletData.Base.DestroyOnSkillEnd)
        {
            BaseSkillComponent component = owner.GetComponent<BaseSkillComponent>();
            if (component == null || !component.Valid)
            {
                Log instance3 = Singleton<Log>.Instance;
                ReadOnlySpan<(string, object)> details =
                [
                    ("bulletRowName", bulletRowName),
                    ("SkillId", bulletCreateParams.SkillId)
                ];
                instance3.Error((ELogModule)71, (ELogAuthor)21, "勾选了技能结束是否销毁子弹, 技能组件不存在", details);
                CreateBulletStatSop(bulletRowName);
                return null;
            }
            Skill? skill = component.GetSkill(bulletCreateParams.SkillId);
            if (skill == null || !skill.Active)
            {
                CreateBulletStatSop(bulletRowName);
                return null;
            }
        }
        EBulletSyncType eBulletSyncType = ModSyncType(bulletCreateParams.SyncType, bulletData);
        if (bulletCreateParams.CreateOnAuthority && eBulletSyncType == EBulletSyncType.SyncCreate && !HasAuthority(owner))
        {
            CreateBulletStatSop(bulletRowName);
            return null;
        }
        int valueOrDefault = bulletCreateParams.ParentVictimId.GetValueOrDefault();
        int valueOrDefault2 = bulletCreateParams.ParentTargetId.GetValueOrDefault();
        int targetId = GetTargetId(owner, bulletData, bulletRowName, valueOrDefault, valueOrDefault2);
        int baseVelocityId = GetBaseVelocityId(owner, bulletData, bulletRowName, valueOrDefault, valueOrDefault2);
        int baseTransformId = GetBaseTransformId(owner, bulletData, bulletRowName, valueOrDefault, valueOrDefault2);
        bulletCreateParams.SyncType = eBulletSyncType;
        bulletCreateParams.BulletData = bulletData;
        bulletCreateParams.TargetId = targetId;
        bulletCreateParams.BaseTransformId = baseTransformId;
        bulletCreateParams.BaseVelocityId = baseVelocityId;
        BulletEntity? result = CreateBullet(owner, bulletRowName, initialTransform, bulletCreateParams, contextId, createSource);
        CreateBulletStatSop(bulletRowName);
        return result;
    }

    // Token 0x060179b4; original private, non-virtual; canonical source lines 535-544.
    private BulletEntity? CreateBullet(Entity owner, string bulletRowName, FTransformDouble? initialTransform, BulletCreateParams bulletCreateParams, long? contextId = null, EBulletCreateSource createSource = EBulletCreateSource.Others)
    {
        bool fromRemote = bulletCreateParams.SyncType == EBulletSyncType.SyncFromRemote;
        BulletEntity bulletEntity = ModelBase<BulletModel>.Instance.CreateBullet(owner, bulletRowName, initialTransform, bulletCreateParams.InitTargetLocation, bulletCreateParams.SkillId, bulletCreateParams.ParentId, fromRemote, bulletCreateParams.TargetId.GetValueOrDefault(), bulletCreateParams.BaseTransformId, bulletCreateParams.BaseVelocityId, bulletCreateParams.Size, bulletCreateParams.BulletData, bulletCreateParams.SyncType, contextId, bulletCreateParams.SkillContextId, bulletCreateParams.Source, bulletCreateParams.LocationOffset, bulletCreateParams.BeginRotatorOffset, bulletCreateParams.RandomPosOffset, bulletCreateParams.RandomInitSpeedOffset, bulletCreateParams.BattleContext, bulletCreateParams.ParentIds, createSource);
        if (bulletEntity == null || !bulletEntity.Valid)
        {
            return null;
        }
        return bulletEntity;
    }

    // Token 0x060179b5; original public, non-virtual; canonical source lines 546-549.
    public void DestroyBullet(int id, bool summonChild, EBulletDestroyReason destroyReason = EBulletDestroyReason.Normal, bool destroyEffectImmediately = false)
    {
        ModelBase<BulletModel>.Instance.DestroyBullet(id, summonChild, destroyReason);
    }
}
