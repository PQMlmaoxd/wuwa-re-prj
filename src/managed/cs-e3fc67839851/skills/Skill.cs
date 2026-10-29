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
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Character.BaseCharacter.Abilities.GA;
using CSharpScript.Game.Module.Battle;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Pawn.Component;
using CSharpScript.Game.Utils;
using UnrealEngine;

public partial class Skill
{
    public UGameplayAbility? ActiveAbility;

    private UAsyncTaskPlayMontageAndWait? ActiveMontageTask;

    private Action<bool>? MontageCallback;

    public long? MontageContextId;

    public long? SkillBehaviorAnimNotifyMessageId;

    public int FightStateHandle;

    private bool ActiveInternal;

    public int CurrentMontageIndex = -1;

    public int NeedPlayMontageIndex = -1;

    private readonly List<int> ActiveBuffHandles = new List<int>();

    private int SkillLevel;

    private BaseSkillComponent? SkillComp;

    private CharacterBuffComponent? BuffComp;

    private BaseTagComponent? TagComp;

    // Token 0x0601a106; original public, non-virtual; canonical source lines 615-664.
    public bool EndSkill()
    {
        if (!Active)
        {
            return false;
        }
        ClearBurstLockTimer();
        InterruptLevel = SkillInfo.InterruptLevel;
        ActiveInternal = false;
        ActiveAbility = null;
        SkillComp.FightStateComp?.ExitState(FightStateHandle);
        ClearMontage();
        ClearEffects();
        foreach (int activeBuffHandle in ActiveBuffHandles)
        {
            BuffComp?.RemoveBuffByHandle(activeBuffHandle, -1, "技能结束移除");
        }
        ActiveBuffHandles.Clear();
        if (TagComp != null && SkillInfo.GroupId == 1)
        {
            if (SkillInfo.IsFullBodySkill)
            {
                TagComp.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.技能中.全身动作"]);
            }
            else
            {
                TagComp.RemoveTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.技能中.部分动作"]);
            }
        }
        SkillBehaviorAnimNotifyMessageId = null;
        if (BuffComp != null && !IsSimulated)
        {
            for (int i = 0; i < SkillInfo.SkillEndBuff.Num(); i++)
            {
                BuffComp.AddBuff(SkillInfo.SkillEndBuff.Get(i), new AddBuffParam
                {
                    InstigatorId = BuffComp.CreatureDataId,
                    Level = SkillLevel,
                    Reason = $"技能{SkillId}结束时添加",
                    PreMessageId = CombatMessageId
                });
            }
            for (int j = 0; j < SkillInfo.EndRemoveBuffIds.Num(); j++)
            {
                BuffComp.RemoveBuff(SkillInfo.EndRemoveBuffIds.Get(j), -1, $"技能{SkillId}结束时移除", CombatMessageId);
            }
        }
        NeedPlayMontageIndex = -1;
        return true;
    }

    // Token 0x0601a109; original public, non-virtual; canonical source lines 698-751.
    public bool PlayMontage(int index, float rate, string startSection, float startTime, Action<bool>? onCompleted = null, long? messageId = null)
    {
        Action<bool> onCompleted2 = onCompleted;
        UAnimMontage montageByIndex = GetMontageByIndex(index);
        if (montageByIndex == null || !montageByIndex.IsValid())
        {
            CombatLog instance = Singleton<CombatLog>.Instance;
            Entity entity = SkillComp.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("技能id:", SkillId),
                ("技能名:", SkillName),
                ("MontageIndex", index)
            ];
            instance.Error(CombatLog.EDebugModule.Skill, entity, "PlaySkillMontage 播放的蒙太奇索引不存在", details);
            return false;
        }
        CurrentMontageIndex = index;
        TArray<FCompositeSection> compositeSections = montageByIndex.CompositeSections;
        bool flag = false;
        for (int i = 0; i < compositeSections.Num(); i++)
        {
            if (compositeSections.Get(i).SectionName.Equals(Singleton<CharacterNameDefines>.Instance.LOOP_SECTION))
            {
                flag = true;
                break;
            }
        }
        float num = startTime;
        float sequenceLength = montageByIndex.SequenceLength;
        if (sequenceLength > 0f && num > sequenceLength && flag)
        {
            num = 0f;
        }
        FName startingSection = FNameUtil.GetDynamicFName(startSection) ?? FNameUtil.EMPTY;
        ActiveMontageTask = UAsyncTaskPlayMontageAndWait.ListenForPlayMontage(SkillComp.GetMainAnimInstance(), montageByIndex, rate, num, startingSection);
        if (ActiveMontageTask.MontageLength <= 0f)
        {
            onCompleted2?.Invoke(true);
            return false;
        }
        ActiveMontageTask.EndCallback.Add(delegate(bool isInterrupted)
        {
            onCompleted2?.Invoke(isInterrupted);
        });
        MontageCallback = onCompleted2;
        MontageContextId = messageId ?? ModelBase<CombatMessageModel>.Instance.GenMessageId();
        SkillComp.MontageComp?.PushMontageInfo(new MontageInfo
        {
            MontageNames = new List<string>(),
            SkillId = SkillId,
            MontageIndex = index,
            MontageTaskMessageId = MontageContextId
        }, montageByIndex);
        return true;
    }

    // Token 0x0601a10a; original private, non-virtual; canonical source lines 753-763.
    private void ClearMontage()
    {
        if (ActiveMontageTask != null)
        {
            UAnimMontage montageToPlay = ActiveMontageTask.MontageToPlay;
            ActiveMontageTask.EndTask();
            ActiveMontageTask = null;
            SkillComp.GetMainAnimInstance().Montage_Stop(0.2f, montageToPlay);
        }
        MontageCallback = null;
    }

    // Token 0x0601a10b; original public, non-virtual; canonical source lines 765-773.
    public void RequestStopMontage(bool isInterrupted)
    {
        if (MontageCallback != null)
        {
            Action<bool>? montageCallback = MontageCallback;
            MontageCallback = null;
            montageCallback?.Invoke(isInterrupted);
        }
    }
}
