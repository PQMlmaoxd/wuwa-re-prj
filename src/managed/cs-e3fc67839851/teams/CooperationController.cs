/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020017ad (global::CooperationController)
 * Canonical source: .../8690662f128b2e76df84c1e174a639299c941845656fe18c97b136e334b87f99/source/_global/CooperationController.cs
 * Canonical raw methods: .../8690662f128b2e76df84c1e174a639299c941845656fe18c97b136e334b87f99/evidence/raw/methods.jsonl
 * Canonical raw types: .../8690662f128b2e76df84c1e174a639299c941845656fe18c97b136e334b87f99/evidence/raw/types.jsonl
 * Scope: selected whole source-level bodies for R22 rows 197-198 only.
 * FormationInputHandler and all other members remain external; no callback or
 * unrelated body is implemented here.
 * The lowered two-element tuple InlineArray log in TryCooperate is represented
 * as a source collection expression solely to preserve observed order/count.
 */

using System;
using CSharpScript.Game.Module.BattleUi;
using CSharpScript.Game.Module.DeadRevive;

[Controller(0)]
public partial class CooperationController : ControllerBase<CooperationController>
{
    // MethodDef 0x0600ab16; public; source lines 33-133; raw row 76.
    public void TryCooperate(long creatureDataId)
    {
        if (ModelBase<SceneTeamModel>.Instance.ChangingRole)
        {
            Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "在换人请求返回前尝试换人", default(ReadOnlySpan<(string, object)>));
            return;
        }
        SceneTeamModel instance = ModelBase<SceneTeamModel>.Instance;
        SceneTeamItem getCurrentTeamItem = instance.GetCurrentTeamItem;
        EntityHandle entityHandle = getCurrentTeamItem?.EntityHandle;
        if (entityHandle == null || getCurrentTeamItem.GetCreatureDataId() == creatureDataId)
        {
            Log instance2 = Singleton<Log>.Instance;
            (string, object) reference = ("CreatureDataId", creatureDataId);
            instance2.Info((ELogModule)43, (ELogAuthor)49, "下场角色实体不存在或相同", new ReadOnlySpan<(string, object)>(ref reference));
            return;
        }
        BaseTagComponent baseTagComponent = entityHandle.Entity.CheckGetComponent<BaseTagComponent>();
        SceneTeamItem teamItem = instance.GetTeamItem(creatureDataId, new GetTeamItemOptions
        {
            ParamType = ETeamParamType.CreatureDataId
        });
        if (teamItem?.EntityHandle == null)
        {
            Log instance3 = Singleton<Log>.Instance;
            (string, object) reference2 = ("CreatureDataId", creatureDataId);
            instance3.Info((ELogModule)43, (ELogAuthor)49, "上场角色实体不存在", new ReadOnlySpan<(string, object)>(ref reference2));
        }
        else if (!teamItem.IsMyRole())
        {
            Log instance4 = Singleton<Log>.Instance;
            (string, object) reference3 = ("CreatureDataId", creatureDataId);
            instance4.Info((ELogModule)43, (ELogAuthor)49, "上场角色为其他玩家的角色", new ReadOnlySpan<(string, object)>(ref reference3));
            if (teamItem.IsDead())
            {
                ControllerBase<DeadReviveController>.Instance.CheckOtherPlayerReviveCooldown(teamItem.GetPlayerId(), teamItem.GetCreatureDataId());
            }
            else if (!baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.BUFF通用标识.不能切人"]))
            {
                CooperateAction(getCurrentTeamItem, teamItem);
            }
        }
        else if (baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.濒死"]))
        {
            if (teamItem.IsDead())
            {
                ControllerBase<DeadReviveController>.Instance.TryReviveRoleWhenCurrentRoleDead(teamItem.GetCreatureDataId(), teamItem.GetConfigId);
            }
        }
        else
        {
            if (baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.溺水"]))
            {
                return;
            }
            bool flag = ModelBase<TowerModel>.Instance.CheckInTower();
            bool flag2 = ModelBase<ShipTowerModel>.Instance.CheckInBattleShipTower();
            bool flag3 = flag || flag2;
            if (baseTagComponent.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.BUFF通用标识.不能切人"]))
            {
                if (flag3 && !ControllerBase<FormationDataController>.Instance.GlobalIsInFight)
                {
                    ControllerBase<ScrollingTipsController>.Instance.ShowTipsById("CannotChangeRoleBeforeStartBattle");
                }
                return;
            }
            if (instance.CurrentGroupType.GetValueOrDefault() != ETeamGroupType.Battle)
            {
                Singleton<Log>.Instance.Info((ELogModule)43, (ELogAuthor)49, "当前正在非战斗编队组，不能切角色", default(ReadOnlySpan<(string, object)>));
                return;
            }
            if (teamItem.IsDead())
            {
                if (ModelBase<SceneTeamModel>.Instance.IsAllDid() || flag)
                {
                    if (flag)
                    {
                        ControllerBase<ScrollingTipsController>.Instance.ShowTipsById("InstanceDungeonShieldViewCantOpen");
                    }
                }
                else
                {
                    ControllerBase<DeadReviveController>.Instance.TryReviveRole(teamItem.GetCreatureDataId(), teamItem.GetConfigId);
                }
                return;
            }
            EGoBattleResultType eGoBattleResultType = teamItem.CanGoBattle();
            if (eGoBattleResultType != 0)
            {
                Log instance5 = Singleton<Log>.Instance;
                // Source-only lift of the compiler-lowered InlineArray2<(string, object)>.
                ReadOnlySpan<(string, object)> details =
                [
                    ("Result", eGoBattleResultType),
                    ("roleId", teamItem.GetConfigId)
                ];
                instance5.Info((ELogModule)43, (ELogAuthor)49, "上场角色无法换人", details);
            }
            else
            {
                CooperateAction(getCurrentTeamItem, teamItem);
            }
        }
    }

    // MethodDef 0x0600ab17; private; source lines 135-141; raw row 77.
    private void CooperateAction(SceneTeamItem goDownRole, SceneTeamItem goBattleRole)
    {
        ICooperationHandler[] handlers = ModelBase<CooperationModel>.Instance.GetHandlers();
        for (int i = 0; i < handlers.Length && !handlers[i].Trigger(goDownRole, goBattleRole); i++)
        {
        }
    }
}