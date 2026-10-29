/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only partial view, not original source or dispatch proof.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02000d39 (global::TsAnimNotifyStateCounterAttack)
 * Scope: two selected whole source-level implementation bodies only.
 * Native-backed properties and generated UFunction dispatch remain external;
 * this partial view intentionally does not promise standalone compilation.
 */

using System;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Character.BaseCharacter.Tools;
using UnrealEngine;
using UnrealEngine.Extension;
using UnrealEngine.Interface;
using UnrealEngine.Service;
using UnrealEngine.Service.Flags;
using UnrealEngine.Utils;

[UClass("/Game/Aki/TypeScript/Game/AnimNotifyState/", EClassFlags.CLASS_None)]
[UnrealObjectPath("/Game/Aki/TypeScript/Game/AnimNotifyState/TsAnimNotifyStateCounterAttack.TsAnimNotifyStateCounterAttack_C")]
public partial class TsAnimNotifyStateCounterAttack : TsAnimNotifyStateBase, IUnrealUObject, IUnrealObject
{
    // Field 0x04001314; observed private nullable long.
    private long? AnMessageId;

    // Field 0x04001315; observed private int.
    private int SkillId;

    // Token 0x06004695; original protected virtual; canonical source lines 113-170.
    protected virtual bool K2_NotifyBegin_Implementation(USkeletalMeshComponent? meshComp, UAnimSequenceBase? animation, float totalDuration)
    {
        AActor owner = meshComp.GetOwner();
        Log instance = Singleton<Log>.Instance;
        (string, object) reference = ("Owner", owner);
        instance.Info((ELogModule)35, (ELogAuthor)21, "CounterAttack Begin", new ReadOnlySpan<(string, object)>(ref reference));
        if (owner is TsBaseCharacter)
        {
            Entity entity = (owner as TsBaseCharacter).CharacterActorComponent?.Entity;
            if (entity == null || !entity.Valid)
            {
                return false;
            }
            AnMessageId = entity.GetComponent<BaseBuffComponent>()?.CreateAnimNotifyContent(animation.GetName(), base.exportIndex);
            CharacterSkillComponent component = entity.GetComponent<CharacterSkillComponent>();
            CharacterHitComponent component2 = entity.GetComponent<CharacterHitComponent>();
            if (component == null || !component.Valid || component2 == null || !component2.Valid)
            {
                return false;
            }
            if (弹反设置 == null)
            {
                return false;
            }
            component2.SetCounterAttackAnsInfo(AnMessageId, base.exportIndex);
            if (弹反摄像机预设 != null || 弹反特效预设 != null)
            {
                SCounterAttack sCounterAttack = new SCounterAttack(弹反设置.弹反部位, 弹反设置.无弹反动作效果, 弹反设置.有弹反动作效果, 弹反设置.削韧倍率, 弹反设置.最大触发距离, 弹反设置.最大触发夹角, 弹反设置.被弹反者应用BuffID, 弹反设置.攻击者应用BuffID, 弹反设置.受击动画忽略Buff检测, 弹反设置.检测Buff列表, 弹反设置.ANS期间被弹反者生效的BuffID, 弹反设置.结束事件Tag, 弹反设置.QTE弹刀忽略角度距离检测);
                if (弹反摄像机预设 != null)
                {
                    sCounterAttack.无弹反动作效果.摄像机设置 = 弹反摄像机预设.CameraData;
                    sCounterAttack.无弹反动作效果.攻击者顿帧 = 弹反摄像机预设.AttackerTimeScale;
                    sCounterAttack.无弹反动作效果.被击者顿帧 = 弹反摄像机预设.VictimTimeScale;
                    sCounterAttack.无弹反动作效果.震屏 = 弹反摄像机预设.CameraShake;
                    sCounterAttack.有弹反动作效果.摄像机设置 = 弹反摄像机预设.CameraData;
                    sCounterAttack.有弹反动作效果.攻击者顿帧 = 弹反摄像机预设.AttackerTimeScale;
                    sCounterAttack.有弹反动作效果.被击者顿帧 = 弹反摄像机预设.VictimTimeScale;
                    sCounterAttack.有弹反动作效果.震屏 = 弹反摄像机预设.CameraShake;
                }
                if (弹反特效预设 != null)
                {
                    sCounterAttack.无弹反动作效果.特效DA = 弹反特效预设.EffectDA;
                    sCounterAttack.无弹反动作效果.特效Offset = 弹反特效预设.Offset;
                    sCounterAttack.无弹反动作效果.特效Scale = 弹反特效预设.Scale;
                    sCounterAttack.有弹反动作效果.特效DA = 弹反特效预设.EffectDA;
                    sCounterAttack.有弹反动作效果.特效Offset = 弹反特效预设.Offset;
                    sCounterAttack.有弹反动作效果.特效Scale = 弹反特效预设.Scale;
                }
                component2.SetCounterAttackInfo(sCounterAttack);
                component2.SetCounterAttackEndTime(totalDuration);
                return true;
            }
            SkillId = component.GetCurrentMontageCorrespondingSkillId();
            component2.SetCounterAttackInfo(弹反设置);
            component2.SetCounterAttackEndTime(totalDuration);
        }
        return true;
    }

    // Token 0x06004697; original protected virtual; canonical source lines 195-252.
    protected virtual bool K2_NotifyEnd_Implementation(USkeletalMeshComponent? meshComp, UAnimSequenceBase? animation)
    {
        USkeletalMeshComponent meshComp2 = meshComp;
        AActor owner = meshComp2.GetOwner();
        Log instance = Singleton<Log>.Instance;
        (string, object) reference = ("Owner", owner);
        instance.Info((ELogModule)35, (ELogAuthor)21, "CounterAttack End", new ReadOnlySpan<(string, object)>(ref reference));
        if (owner is TsBaseCharacter tsBaseCharacter)
        {
            Entity entity = tsBaseCharacter.CharacterActorComponent?.Entity;
            if (entity == null || !entity.Valid)
            {
                Log instance2 = Singleton<Log>.Instance;
                (string, object) reference2 = ("Owner", tsBaseCharacter);
                instance2.Error((ELogModule)35, (ELogAuthor)21, "CounterAttack End entity not valid", new ReadOnlySpan<(string, object)>(ref reference2));
                return false;
            }
            CharacterSkillComponent component = entity.GetComponent<CharacterSkillComponent>();
            CharacterHitComponent component2 = entity.GetComponent<CharacterHitComponent>();
            if (component == null || !component.Valid || component2 == null || !component2.Valid)
            {
                Log instance3 = Singleton<Log>.Instance;
                // Source-only representation of the observed lowered InlineArray3 log;
                // tuple evaluation order/count is retained, not original ABI/source syntax.
                ReadOnlySpan<(string, object)> details =
                [
                    ("Owner", tsBaseCharacter),
                    ("SkillComp", component?.Valid),
                    ("HitComp", component2?.Valid)
                ];
                instance3.Error((ELogModule)35, (ELogAuthor)21, "CounterAttack End skillComp or hitComp not valid", details);
                return false;
            }
            bool hadTriggerCounterAttack = component2.HadTriggerCounterAttack;
            component2.CounterAttackEnd();
            if (hadTriggerCounterAttack)
            {
                return true;
            }
            if (FNameUtil.IsNothing(生成子弹ID))
            {
                return false;
            }
            BulletUtil.CreateBulletFromAN(tsBaseCharacter, 生成子弹ID.ToString(), null, SkillId, needSync: false, AnMessageId);
            return true;
        }
        if (FNameUtil.IsNothing(生成子弹ID))
        {
            return false;
        }
        BP_EWorldType worldType = UKuroRenderingRuntimeBPPluginBPLibrary.GetWorldType(owner.GetWorld());
        if (worldType != BP_EWorldType.Editor && worldType != BP_EWorldType.EditorPreview)
        {
            return false;
        }
        Singleton<ResourceSystem>.Instance.LoadTypeAsync("BPL_BulletPreview", delegate
        {
            AActor Ret = null;
            BPL_BulletPreview_C.ShowBulletPreview(UKismetSystemLibrary.GetPathName(UKismetSystemLibrary.GetOuterObject(this)), 生成子弹ID, owner, meshComp2, owner.GetWorld(), ref Ret);
        });
        return false;
    }
}
