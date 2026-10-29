/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02005f46 (CSharpScript.Game.Module.Battle.PassiveSkillCdInfo)
 * Scope: four selected whole method bodies only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;

namespace CSharpScript.Game.Module.Battle
{
    public partial class PassiveSkillCdInfo
    {
        // Observed fields needed by the selected cooldown view.
        public long SkillId;
        public float SkillCd;
        public float Threshold;
        public readonly HashSet<int> EntityIds = new HashSet<int>();
        public float CurMaxCd;
        public readonly OrderedDictionary<int, double> SkillCdFinishStampMap = new OrderedDictionary<int, double>();

        // Token 0x0603d41e; original public, non-virtual.
        public double GetCurRemainingCd(int entityId)
        {
            double finishStamp = SkillCdFinishStampMap.GetValueOrDefault(entityId, 0.0);
            if (finishStamp != 0.0)
            {
                return (finishStamp - Singleton<Time>.Instance.FlowTime) * Singleton<TimeUtil>.Instance.Millisecond;
            }
            return 0.0;
        }

        // Token 0x0603d41f; original public, non-virtual.
        public bool IsInCd(int entityId)
        {
            return GetCurRemainingCd(entityId) > (double)Math.Max(Threshold, 0f);
        }

        // Token 0x0603d420; original private, non-virtual.
        private void RemoveInvalidCd()
        {
            List<int> invalidEntityIds = new List<int>();
            foreach (int entityId in SkillCdFinishStampMap.Keys)
            {
                if (IsInCd(entityId))
                {
                    break;
                }
                invalidEntityIds.Add(entityId);
            }
            foreach (int entityId in invalidEntityIds)
            {
                SkillCdFinishStampMap.Remove(entityId);
            }
            if (SkillCdFinishStampMap.Count < 20)
            {
                return;
            }

            int? firstEntityId = null;
            if (EntityIds.Count > 0)
            {
                // The first HashSet item is selected; no stable-order guarantee is inferred.
                foreach (int entityId in EntityIds)
                {
                    firstEntityId = entityId;
                    break;
                }
            }

            Log logger = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> details =
            [
                ("skillId", SkillId),
                ("firstEntityId", firstEntityId)
            ];
            logger.Info((ELogModule)35, (ELogAuthor)86, "被动技能CD数量超上限,移除最先的entityCD", details);
            if (firstEntityId.HasValue)
            {
                SkillCdFinishStampMap.Remove(firstEntityId.Value);
            }
        }

        // Token 0x0603d421; original public, non-virtual.
        public bool StartCd(long skillId, int entityId, float cdTime = -1f)
        {
            if (IsInCd(entityId))
            {
                return false;
            }

            float duration = cdTime;
            if (duration == -1f)
            {
                duration = SkillCd;
            }
            // Deliberately retain <=: an ordinary nonpositive value returns
            // without writes, while NaN does not take this branch.
            if (duration <= 0f)
            {
                return true;
            }

            CurMaxCd = duration;
            double oldFinishStamp = SkillCdFinishStampMap.GetValueOrDefault(entityId, 0.0);
            double value =
                (Singleton<Time>.Instance.FlowTime > oldFinishStamp
                    ? Singleton<Time>.Instance.FlowTime
                    : oldFinishStamp)
                + (double)(duration * (float)Singleton<TimeUtil>.Instance.InverseMillisecond);
            RemoveInvalidCd();
            SkillCdFinishStampMap.Remove(entityId);
            SkillCdFinishStampMap.Add(entityId, value);
            return true;
        }
    }
}
