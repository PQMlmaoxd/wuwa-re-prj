/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003202 (global::RoleQteComponent)
 * Canonical source: .../c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5/source/_global/RoleQteComponent.cs
 * Canonical raw methods: .../c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5/evidence/raw/methods.jsonl
 * Canonical raw types: .../c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5/evidence/raw/types.jsonl
 * Scope: selected whole source-level bodies for R22 rows 201-209 only.
 * OnActiveQteTagChange, QteConsume, OnEnterOnlineWorld, OnQteSkillTagChange,
 * SelectTarget, ExecuteQteNotify, lifecycle cleanup, and other callback method
 * groups remain external without implementations or stubs.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using AkiClient.Game.Aki.Data.Fight.Struct;
using CSharpScript.Game;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component;
using Google.Protobuf;
using UnrealEngine;

public partial class RoleQteComponent : EntityComponent
{
    // Field tokens: normalQteTag 0x0400d6c0; ActorComponent 0x0400d6c2;
    // AbilityComponent 0x0400d6c3; TagComponent 0x0400d6c4;
    // BuffComponent 0x0400d6c5; TeamComponent 0x0400d6c6;
    // SkillComponent 0x0400d6c7; ElementComponent 0x0400d6c8.
    private readonly int normalQteTag = GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.普通QTE"];
    private CharacterActorComponent? ActorComponent;
    private CharacterAbilityComponent? AbilityComponent;
    private BaseTagComponent? TagComponent;
    private CharacterBuffComponent? BuffComponent;
    private RoleTeamComponent? TeamComponent;
    private CharacterSkillComponent? SkillComponent;
    private RoleElementComponent? ElementComponent;

    // Field tokens: ConsumedQteEntitySet 0x0400d6cc; IsInQte 0x0400d6cd;
    // QteTagListeners 0x0400d6ce; QteTagMap 0x0400d6cf;
    // QteTagRowName 0x0400d6d0; GoBattleActor 0x0400d6d1.
    private readonly HashSet<int> ConsumedQteEntitySet = new HashSet<int>();
    public bool IsInQte;
    private readonly List<ITagTask> QteTagListeners = new List<ITagTask>();
    private readonly Dictionary<int, string> QteTagMap = new Dictionary<int, string>();
    private string QteTagRowName = "";
    public TsBaseCharacter? GoBattleActor;

    // MethodDef 0x0601a8c1; protected virtual override; source lines 58-73; raw row 673.
    protected override bool OnStart()
    {
        ActorComponent = base.Entity.GetComponent<CharacterActorComponent>();
        AbilityComponent = base.Entity.CheckGetComponent<CharacterAbilityComponent>();
        TagComponent = base.Entity.CheckGetComponent<BaseTagComponent>();
        BuffComponent = base.Entity.CheckGetComponent<CharacterBuffComponent>();
        TeamComponent = base.Entity.CheckGetComponent<RoleTeamComponent>();
        SkillComponent = base.Entity.CheckGetComponent<CharacterSkillComponent>();
        ElementComponent = base.Entity.CheckGetComponent<RoleElementComponent>();
        QteTagListeners.Add(TagComponent.ListenForTagAddOrRemove(GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.激活QTE"], OnActiveQteTagChange));
        QteTagListeners.Add(TagComponent.ListenForTagAddOrRemove(GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.QTE"], OnQteSkillTagChange));
        Singleton<EventSystem>.Instance.Add(EEventName.OnEnterOnlineWorld, OnEnterOnlineWorld);
        Singleton<EventSystem>.Instance.Add<int>(EEventName.CharQteConsume, QteConsume);
        InitTriggerTags();
        return true;
    }

    // MethodDef 0x0601a8c3; private; source lines 86-101; raw row 675.
    private void InitTriggerTags()
    {
        TagComponent.AddTag(normalQteTag);
        Dictionary<int, string>.ValueCollection values = ConfigBase<WorldConfig>.Instance.GetQteTagDataMap().Values;
        UDataTable qteTagDataTable = ConfigBase<WorldConfig>.Instance.GetQteTagDataTable();
        foreach (string item in values)
        {
            int num = DataTableUtil.GetDataTableRow<SQteTag>(qteTagDataTable, item).QteTag.TagId();
            QteTagListeners.Add(TagComponent.ListenForTagAddOrRemove(num, OnQteTagTriggerChange));
            if (TagComponent.HasTag(num))
            {
                QteTagMap[num] = item;
            }
        }
        RefreshQteTagRowName();
    }

    // MethodDef 0x0601a8c8; public; source lines 143-202; raw row 680.
    // Raw EH: finally try 301..333, handler 333..347.
    public bool IsQteReady(EntityHandle goDownPlayerHandle)
    {
        if (IsInQte)
        {
            return false;
        }
        if (!ControllerBase<FormationDataController>.Instance.GlobalIsInFight)
        {
            return false;
        }
        if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.濒死"]))
        {
            return false;
        }
        if (TagComponent.HasTag(GameplayTagDefine.EGameplayTagId["战斗状态.技能限制.禁止QTE"]))
        {
            return false;
        }
        SQteTag qteTagData = GetQteTagData();
        if (qteTagData == null)
        {
            return false;
        }
        if (TagComponent.HasTag(qteTagData.NoTag.TagId()))
        {
            return false;
        }
        BaseTagComponent component = goDownPlayerHandle.Entity.GetComponent<BaseTagComponent>();
        if (!component.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.激活QTE"]) || component.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.濒死"]))
        {
            return false;
        }
        if (!component.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.激活QTE.不消耗能量"]) && ConsumedQteEntitySet.Contains(goDownPlayerHandle.Id))
        {
            return false;
        }
        CharacterActorComponent component2 = goDownPlayerHandle.Entity.GetComponent<CharacterActorComponent>();
        if (ControllerBase<FormationDataController>.Instance.IsBattleMulti())
        {
            if (component2.IsAutonomousProxy)
            {
                return false;
            }
            List<SceneTeamItem> teamItemsInRange = ModelBase<SceneTeamModel>.Instance.GetTeamItemsInRange(ActorComponent.ActorLocationProxy, 5000f);
            bool flag = false;
            foreach (SceneTeamItem item in teamItemsInRange)
            {
                if (item.EntityHandle == goDownPlayerHandle)
                {
                    flag = true;
                    break;
                }
            }
            if (!flag)
            {
                return false;
            }
        }
        return true;
    }

    // MethodDef 0x0601a8c9; public; source lines 204-216; raw row 681.
    public void UseExitSkill(EntityHandle goBattlePlayerHandle)
    {
        SQteTag qteTagData = goBattlePlayerHandle.Entity.GetComponent<RoleQteComponent>().GetQteTagData();
        if (!(qteTagData == null) && !(qteTagData.ExitSkillTrigger.TagName == FName.NAME_None))
        {
            GoBattleActor = goBattlePlayerHandle.Entity.GetComponent<CharacterActorComponent>().Actor;
            FGameplayEventData fGameplayEventData = new FGameplayEventData();
            fGameplayEventData.Instigator = ActorComponent.Actor;
            fGameplayEventData.Target = GoBattleActor;
            AbilityComponent.SendGameplayEventToActor(qteTagData.ExitSkillTrigger, fGameplayEventData);
            GoBattleActor = null;
        }
    }

    // MethodDef 0x0601a8ca; public; source lines 218-252; raw row 682.
    public bool ExecuteQte(EntityHandle goDownPlayerHandle)
    {
        SQteTag qteTagData = GetQteTagData();
        if (qteTagData == null || qteTagData.QteTrigger.TagName == FName.NAME_None)
        {
            return false;
        }
        long value = ExecuteQte2Server(goDownPlayerHandle);
        SelectTarget(goDownPlayerHandle);
        TeamComponent.InterruptDisableWithEffect();
        TeamComponent.SetTeamTag(ETeamState.OnStage);
        base.Entity.EnableByKey(EEntityDisableKey.GoDown);
        AbilityComponent.SendGameplayEventToActor(qteTagData.QteTrigger);
        for (int i = 0; i < qteTagData.QteBuffs.Num(); i++)
        {
            BuffComponent.AddBuff(qteTagData.QteBuffs.Get(i), new AddBuffParam
            {
                InstigatorId = BuffComponent.CreatureDataId,
                PreMessageId = value,
                Reason = "ExecuteQte"
            });
        }
        RoleElementComponent component = goDownPlayerHandle.Entity.GetComponent<RoleElementComponent>();
        BaseTagComponent? component2 = goDownPlayerHandle.Entity.GetComponent<BaseTagComponent>();
        component.TriggerEvents(base.Entity);
        if (!component2.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.激活QTE.不消耗能量"]))
        {
            for (int j = 0; j < qteTagData.ConsumeBuffs.Num(); j++)
            {
                component.ClearElementEnergy(base.Entity, qteTagData.ConsumeBuffs.Get(j));
            }
        }
        Singleton<EventSystem>.Instance.Emit(EEventName.CharExecuteQte, base.Entity.Id, goDownPlayerHandle.Id);
        return true;
    }

    // MethodDef 0x0601a8d4; private; source lines 598-612; raw row 692.
    private void OnQteTagTriggerChange(int tagId, bool bTagExists)
    {
        if (bTagExists)
        {
            QteTagMap[tagId] = ConfigBase<WorldConfig>.Instance.GetQteTagDataMap()[tagId];
            RefreshQteTagRowName();
            return;
        }
        string valueOrDefault = QteTagMap.GetValueOrDefault(tagId);
        QteTagMap.Remove(tagId);
        if (QteTagRowName == valueOrDefault)
        {
            RefreshQteTagRowName();
        }
    }

    // MethodDef 0x0601a8d5; private; source lines 614-629; raw row 693.
    private void RefreshQteTagRowName()
    {
        UDataTable qteTagDataTable = ConfigBase<WorldConfig>.Instance.GetQteTagDataTable();
        int num = 0;
        foreach (string value in QteTagMap.Values)
        {
            SQteTag dataTableRow = DataTableUtil.GetDataTableRow<SQteTag>(qteTagDataTable, value);
            if (dataTableRow.Priority >= num)
            {
                QteTagRowName = value;
                ElementComponent.TriggerEnergy = dataTableRow.Energy;
                num = dataTableRow.Priority;
            }
        }
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.CharQteTagRowNameChanged);
    }

    // MethodDef 0x0601a8d6; public; source lines 631-638; raw row 694.
    public SQteTag? GetQteTagData()
    {
        if (!string.IsNullOrEmpty(QteTagRowName))
        {
            return DataTableUtil.GetDataTableRow<SQteTag>(ConfigBase<WorldConfig>.Instance.GetQteTagDataTable(), QteTagRowName);
        }
        return null;
    }

    // MethodDef 0x0601a8d7; private; source lines 640-648, actual closing brace 649; raw row 695.
    private long ExecuteQte2Server(EntityHandle goDownPlayerHandle)
    {
        ExecuteQtePush executeQtePush = ExecuteQtePush.Create();
        executeQtePush.DownEntityId = ModelBase<CreatureModel>.Instance.GetCreatureDataId(goDownPlayerHandle.Entity.Id);
        executeQtePush.UpEntityId = ModelBase<CreatureModel>.Instance.GetCreatureDataId(base.Entity.Id);
        executeQtePush.QteId = UGASBPLibrary.FnvHash(QteTagRowName);
        long num = ModelBase<CombatMessageModel>.Instance.GenMessageId();
        Singleton<CombatNet>.Instance.Send(EPushMessageId.ExecuteQtePush, base.Entity, (IMessage)(object)executeQtePush, null, num);
        return num;
    }
}

/*
 * R29 provenance: reconstructed semantic source-view append.
 * Type: 0x02003202 global::RoleQteComponent
 * Method: 0x0601a8d3 SelectTarget(EntityHandle goDownPlayerHandle) -> void
 * Canonical source: <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_01>/src\reconstructed\managed\full\CSharpScript\shards\c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5\source\_global\RoleQteComponent.cs:569-596
 * Canonical raw methods: <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_01>/src\reconstructed\managed\full\CSharpScript\shards\c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5\evidence\raw\methods.jsonl:691
 * Canonical raw types: <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_01>/src\reconstructed\managed\full\CSharpScript\shards\c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5\evidence\raw\types.jsonl:36
 * RVA: 8241012; code size: 217; EH: 0
 * Visibility: private instance; virtual: false; compiler-generated: false
 * Canonical source SHA256: 91a425f0bf4bf330ab603239f812b9bc25ebc53dd9ddb98f8c100765d6b6844d
 * Canonical raw methods SHA256: fa47489dc9abf6c3f610b58f3eedd03390a86bc273b01a32686bd5404bf17189
 * Canonical raw types SHA256: abbd7ec46dd6ec222a2a28459fdc438f126c67ba12f9fdec112a0829abbddd91
 * Assembly: <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_01>/private\r5b1\materialized\CSharpScript.dll
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Semantics: whole selected source-level body only; this partial class is a noncanonical source view.
 * Semantics limits: dependency bodies, lock-on ranking, runtime delivery, camera/native behavior,
 * lifecycle/cleanup, and successful target effects remain external or UNKNOWN.
 */
public partial class RoleQteComponent : EntityComponent
{
	private void SelectTarget(EntityHandle goDownPlayerHandle)
	{
		WorldEntity entity = goDownPlayerHandle.Entity;
		bool flag = entity.GetComponent<CharacterManipulateComponent>()?.IsManipulating() ?? false;
		CharacterSkillComponent component = entity.GetComponent<CharacterSkillComponent>();
		EntityHandle skillTarget = component.SkillTarget;
		if (!flag && skillTarget != null && skillTarget.Valid)
		{
			WorldEntity? entity2 = skillTarget.Entity;
			if (entity2 != null && entity2.Active)
			{
				BaseTagComponent? component2 = skillTarget.Entity.GetComponent<BaseTagComponent>();
				if (component2 == null || !component2.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.濒死"]))
				{
					SkillComponent.SkillTarget = skillTarget;
					SkillComponent.SkillTargetSocket = component.SkillTargetSocket;
					return;
				}
			}
		}
		CharacterLockOnComponent component3 = entity.GetComponent<CharacterLockOnComponent>();
		component3.DetectSoftLockTarget(new SkillTargetImpl
		{
			LockOnConfigId = 4
		});
		SkillComponent.SkillTarget = component3.GetCurrentTarget();
		SkillComponent.SkillTargetSocket = component3.GetCurrentTargetSocketName();
	}
}