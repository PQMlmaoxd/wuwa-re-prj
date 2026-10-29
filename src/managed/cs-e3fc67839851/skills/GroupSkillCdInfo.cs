/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02005f43 (CSharpScript.Game.Module.Battle.GroupSkillCdInfo)
 * Scope: seven selected whole method/property bodies only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using CSharpScript.Game.Common.Event;

namespace CSharpScript.Game.Module.Battle
{
    public partial class GroupSkillCdInfo
    {
        // Observed members used by the selected cooldown view.
        public int RemainingCount;
        public int LimitCount;
        public readonly Dictionary<int, SkillCdInfo> SkillCdInfoMap = new Dictionary<int, SkillCdInfo>();
        public readonly Queue<int> SkillIdQueue = new Queue<int>();
        public readonly Queue<float> CdQueue = new Queue<float>();
        public float CurMaxCd;
        public double SkillCdFinishStamp;
        private TimerHandle? SkillCdTimer;

        // Token 0x0603d3fd (getter). The property is represented as a
        // property, not as an invented callable getter.
        public float CurRemainingCd
        {
            get
            {
                if (SkillCdFinishStamp != 0.0)
                {
                    return (float)((SkillCdFinishStamp - Singleton<Time>.Instance.FlowTime) * Singleton<TimeUtil>.Instance.Millisecond);
                }
                return 0f;
            }
        }

        // Token 0x0603d3fe; original public, non-virtual.
        public bool IsInCd()
        {
            return RemainingCount < LimitCount;
        }

        // Token 0x0603d403; original private, non-virtual.
        private void OnSkillCdFinish(float delta)
        {
            double finishStamp = SkillCdFinishStamp;
            SkillCdTimer = null;
            SkillCdFinishStamp = 0.0;
            RemainingCount++;
            float elapsed = (float)(Math.Max(0.0, Singleton<Time>.Instance.FlowTime - finishStamp) * Singleton<TimeUtil>.Instance.Millisecond);
            (int? nextSkillId, float? nextCd) = GetNextSkill();
            while (nextCd.HasValue && elapsed >= nextCd.Value)
            {
                RemainingCount++;
                elapsed -= nextCd.Value;
                (nextSkillId, nextCd) = GetNextSkill();
            }
            if (RemainingCount > LimitCount)
            {
                RemainingCount = LimitCount;
                Log logger = Singleton<Log>.Instance;
                logger.Error((ELogModule)35, (ELogAuthor)17, "技能CD结束，可用次数已达上限", default(ReadOnlySpan<(string, object)>));
            }
            OnCountChanged();
            if (nextSkillId.HasValue && nextCd.HasValue)
            {
                StartSkillCdTimer(nextSkillId.Value, nextCd.Value - elapsed);
            }
        }

        // Token 0x0603d404; original private, non-virtual.
        private (int?, float?) GetNextSkill()
        {
            if (SkillIdQueue.Size <= 0 || CdQueue.Size <= 0)
            {
                return (null, null);
            }
            return (SkillIdQueue.Pop(), CdQueue.Pop());
        }

        // Token 0x0603d406; original private, non-virtual.
        private void AheadFinishSkillCd()
        {
            if (SkillCdTimer != null)
            {
                TimerSystem.FlowTimeInstance.Remove(SkillCdTimer);
                OnSkillCdFinish(0f);
            }
        }

        // Token 0x0603d407; original private, non-virtual.
        private void ChangeSkillCd(float newCd)
        {
            // The finish stamp is written even when no timer exists.
            SkillCdFinishStamp = newCd <= 0f
                ? Singleton<Time>.Instance.FlowTime
                : Singleton<Time>.Instance.FlowTime + (double)(newCd * (float)Singleton<TimeUtil>.Instance.InverseMillisecond);
            if (SkillCdTimer != null)
            {
                TimerSystem.FlowTimeInstance.Remove(SkillCdTimer);
                float interval = Math.Max(20f, newCd * (float)Singleton<TimeUtil>.Instance.InverseMillisecond);
                SkillCdTimer = TimerSystem.FlowTimeInstance.Delay(OnSkillCdFinish, interval);
            }
        }

        // Token 0x0603d40d; original public, non-virtual.
        public void ModifyRemainingCd(float modifyTime, float changeTimePercentage)
        {
            if (IsInCd())
            {
                float newCd = CurRemainingCd + modifyTime + CurMaxCd * changeTimePercentage;
                // Keep ordered <= semantics: NaN does not take the early-finish branch.
                if (newCd <= 0f)
                {
                    AheadFinishSkillCd();
                    return;
                }
                ChangeSkillCd(newCd);
                Singleton<EventSystem>.Instance.Emit(EEventName.CharSkillRemainCdChanged, this);
            }
        }
    }
}
