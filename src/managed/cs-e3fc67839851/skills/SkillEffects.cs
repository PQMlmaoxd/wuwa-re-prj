/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200312b (global::Skill)
 * Scope: four selected whole source-level method bodies only.
 * Missing members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using CSharpScript.Game.Effect;
using CSharpScript.Game.Module.Battle;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Pawn.Component;
using CSharpScript.Game.Utils;
using UnrealEngine;

public partial class Skill
{
    // Observed storage declarations needed by this selected view.
    // Observed field 0x0400d10e.
    private bool IsSimulatedInternal;

    // Observed field 0x0400d122.
    public EndSkillInfo? EndSkillInfo;

    // Observed field 0x0400d128.
    private PawnTimeScaleComponent? TimeScaleComp;

    // Observed field 0x0400d11d; the nested element declarations remain external.
    private readonly Dictionary<ESkillEffectType, List<ISkillEffect>> EffectsAttached = new Dictionary<ESkillEffectType, List<ISkillEffect>>();

    // Skill.ESkillEffectType 0x020093c2, Skill.ISkillEffect 0x020093c3, and
    // Skill.SkillEffect 0x020093c4 are external unselected nested declarations
    // from canonical Skill.cs:15-42 (raw types.jsonl rows 234-236). They are
    // intentionally not restated here; this source view is not standalone.

    // Token 0x0601a101; original public, non-virtual; canonical source lines 443-457.
    public void AttachEffect(int effect, ESkillEffectType type, FName boneName, float whenSkillEndEnableTime)
    {
        if (!EffectsAttached.TryGetValue(type, out List<ISkillEffect> value))
        {
            value = new List<ISkillEffect>();
            EffectsAttached[type] = value;
        }
        SkillEffect item = new SkillEffect
        {
            BoneName = boneName,
            EffectHandle = effect,
            WhenSkillEndEnableTime = whenSkillEndEnableTime
        };
        value.Add(item);
    }

    // Token 0x0601a102; original private, non-virtual; canonical source lines 459-502.
    private void CloseEffect(ESkillEffectType type, int handleId, float whenSkillEndEnableTime)
    {
        if (!Singleton<EffectSystem>.Instance.IsValid(handleId))
        {
            return;
        }
        Singleton<EffectSystem>.Instance.SetTimeScale(handleId, 1f);
        if (whenSkillEndEnableTime > 0f && Singleton<EffectSystem>.Instance.GetTotalPassTime(handleId) > whenSkillEndEnableTime)
        {
            return;
        }
        AActor sureEffectActor = Singleton<EffectSystem>.Instance.GetSureEffectActor(handleId);
        EDetachmentRule eDetachmentRule = EDetachmentRule.KeepWorld;
        switch (type)
        {
        case ESkillEffectType.Detach:
            sureEffectActor?.K2_DetachFromActor(eDetachmentRule, eDetachmentRule, eDetachmentRule);
            EffectUtil.ListenForeverTimeScale(handleId, TimeScaleComp);
            Singleton<EffectSystem>.Instance.StopEffectById(handleId, "[Skill.EffectsProcess] Detach", immediately: false);
            break;
        case ESkillEffectType.DetachDestroy:
            if (sureEffectActor != null)
            {
                sureEffectActor.K2_DetachFromActor(eDetachmentRule, eDetachmentRule, eDetachmentRule);
                Singleton<EffectSystem>.Instance.StopEffectById(handleId, "[Skill.EffectsProcess] DetachDestroy", immediately: true);
            }
            break;
        case ESkillEffectType.DetachEnd:
            if (sureEffectActor != null)
            {
                sureEffectActor.K2_DetachFromActor(eDetachmentRule, eDetachmentRule, eDetachmentRule);
                EffectUtil.ListenForeverTimeScale(handleId, TimeScaleComp);
                Singleton<EffectSystem>.Instance.StopEffectById(handleId, "[Skill.EffectsProcess] DetachEnd", immediately: false);
            }
            break;
        case ESkillEffectType.UnDetachDestroy:
            Singleton<EffectSystem>.Instance.StopEffectById(handleId, "[Skill.EffectsProcess] UnDetachDestroy", immediately: true);
            break;
        case ESkillEffectType.UnDetachEnd:
            EffectUtil.ListenForeverTimeScale(handleId, TimeScaleComp);
            Singleton<EffectSystem>.Instance.StopEffectById(handleId, "[Skill.EffectsProcess] UnDetachEnd", immediately: false);
            break;
        }
    }

    // Token 0x0601a103; original private, non-virtual; canonical source lines 504-522.
    // EH: enumerator-only finally (try 21..116; handler 116..130).
    private void ClearEffects()
    {
        if (EffectsAttached == null)
        {
            return;
        }
        foreach (KeyValuePair<ESkillEffectType, List<ISkillEffect>> item in EffectsAttached)
        {
            List<ISkillEffect> value = item.Value;
            for (int num = value.Count - 1; num >= 0; num--)
            {
                ISkillEffect skillEffect = value[num];
                value.RemoveAt(num);
                int effectHandle = skillEffect.EffectHandle;
                CloseEffect(item.Key, effectHandle, skillEffect.WhenSkillEndEnableTime);
            }
        }
        EffectsAttached.Clear();
    }

    // Token 0x0601a104; original public, non-virtual; canonical source lines 524-537.
    public bool BeginSkill()
    {
        if (Active)
        {
            return false;
        }
        CurrentMontageIndex = -1;
        ActiveInternal = true;
        IsSimulatedInternal = false;
        CombatMessageId = ModelBase<CombatMessageModel>.Instance.GenMessageId();
        MontageContextId = null;
        EndSkillInfo?.Reset();
        return true;
    }
}
