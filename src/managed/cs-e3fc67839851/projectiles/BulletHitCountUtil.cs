/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002dd6 (global::BulletHitCountUtil; raw type row 85)
 * Scope: one selected whole source-level method body only.
 * Other hit-count helpers remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System.Collections.Generic;

public static partial class BulletHitCountUtil
{
    // Token 0x06017a58; original public static, non-virtual; canonical source lines 5-42.
    public static bool HitCountCondition(BulletInfo bulletInfo, Entity victimEntity)
    {
        if (bulletInfo.CountByParent && bulletInfo.ParentBulletInfo != null)
        {
            return HitCountCondition(bulletInfo.ParentBulletInfo, victimEntity);
        }
        _ = Singleton<BulletConstant>.Instance.OpenHitActorLog;
        Entity entity = victimEntity;
        if (entity == null)
        {
            return false;
        }
        entity = entity.GetComponent<CharacterFollowComponent>()?.GetAttributeHolderExceptVisionSummon() ?? entity;
        BulletDataMain bulletDataMain = bulletInfo.BulletDataMain;
        if (bulletDataMain.Base.VictimCount >= 0 && bulletInfo.EntityHitSet.Count >= bulletDataMain.Base.VictimCount && !bulletInfo.EntityHitSet.Contains(entity.Id))
        {
            return false;
        }
        if (bulletDataMain.Base.HitCountMax >= 0 && bulletInfo.HitNumberAll >= bulletDataMain.Base.HitCountMax)
        {
            return false;
        }
        if (!bulletInfo.EntityHitCount.TryGetValue(entity.Id, out var value))
        {
            bulletInfo.EntityHitCount[entity.Id] = 1;
            bulletInfo.EntityHitSet.Add(entity.Id);
            bulletInfo.HitNumberAll++;
            return true;
        }
        if (bulletDataMain.Base.HitCountPerVictim > 0 && value >= bulletDataMain.Base.HitCountPerVictim)
        {
            return false;
        }
        bulletInfo.EntityHitCount[entity.Id] = value + 1;
        bulletInfo.EntityHitSet.Add(entity.Id);
        bulletInfo.HitNumberAll++;
        return true;
    }
}
