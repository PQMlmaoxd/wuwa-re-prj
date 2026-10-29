/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002967 (global::SceneTeamModel)
 * Scope: three selected whole source-level method bodies only.
 * Missing members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using System.Linq;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Game;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.RoleUi;
using CSharpScript.Game.Module.SeamlessTravel;
using CSharpScript.Game.Ui;
using UnrealEngine;

[Model(0)]
public partial class SceneTeamModel : ModelBase<SceneTeamModel>
{
    private Stat? OnChangeRoleStat;

    private readonly Dictionary<int, SceneTeamPlayer> TeamPlayerMap = new Dictionary<int, SceneTeamPlayer>();

    private SceneTeamItem? CurrentItem;

    private double ChangeRoleCooldown;

    private double DefaultChangeRoleCooldown;

    // Token 0x0601510b; original protected virtual; canonical source lines 100-108.
    protected override bool OnInit()
    {
        DefaultChangeRoleCooldown = ConfigCommonParamById.GetFloatConfig("change_role_cooldown").Value;
        ResetChangeRoleCooldown();
        OnChangeRoleStat = Stat.Create("SceneTeamModel.OnChangeRoleStat", "", "STATGROUP_KuroBattle");
        MarkTeamNotReady();
        Singleton<EventSystem>.Instance.Add(EEventName.OnGlobalFootstepMaterialChange, (Delegate)new Action<UPhysicalMaterial>(OnPhysicalMaterialChange));
        return true;
    }

    // Token 0x06015133; original public, non-virtual; canonical source lines 890-952.
    public void ChangeRole(long creatureDataId, IChangeRoleParams? @params = null)
    {
        Log instance = Singleton<Log>.Instance;
        (string, object) reference = ("CreatureDataId", creatureDataId);
        instance.Info((ELogModule)43, (ELogAuthor)49, "开始切换角色", new ReadOnlySpan<(string, object)>(ref reference));
        SceneTeamItem teamItem = GetTeamItem(creatureDataId, new GetTeamItemOptions
        {
            ParamType = ETeamParamType.CreatureDataId
        });
        if (teamItem == null || !teamItem.IsMyRole())
        {
            Log instance2 = Singleton<Log>.Instance;
            (string, object) reference2 = ("CreatureDataId", creatureDataId);
            instance2.Warn((ELogModule)43, (ELogAuthor)49, "队伍实例不存在或非本机角色", new ReadOnlySpan<(string, object)>(ref reference2));
            return;
        }
        bool valueOrDefault = (@params?.ForceChangeRole).GetValueOrDefault();
        if (!teamItem.CanControl(valueOrDefault))
        {
            Log instance3 = Singleton<Log>.Instance;
            (string, object) reference3 = ("CreatureDataId", creatureDataId);
            instance3.Warn((ELogModule)43, (ELogAuthor)49, "角色不允许操控", new ReadOnlySpan<(string, object)>(ref reference3));
            return;
        }
        EntityHandle entityHandle = GetCurrentTeamItem?.EntityHandle;
        EntityHandle entityHandle2 = teamItem.EntityHandle;
        if (entityHandle2 == null)
        {
            Log instance4 = Singleton<Log>.Instance;
            (string, object) reference4 = ("CreatureDataId", creatureDataId);
            instance4.Warn((ELogModule)43, (ELogAuthor)49, "角色实体无效", new ReadOnlySpan<(string, object)>(ref reference4));
            return;
        }
        int? id = ModelBase<PlayerInfoModel>.Instance.GetId();
        if (TeamPlayerMap.ContainsKey(id.Value))
        {
            TeamPlayerMap[id.Value].GetCurrentGroup()?.SetCurrentRole(teamItem.GetCreatureDataId());
        }
        CurrentItem = teamItem;
        Singleton<EventSystem>.Instance.Emit(EEventName.OnBeforeChangeRole, entityHandle2, entityHandle);
        bool valueOrDefault2 = (@params?.UseGoBattleSkill).GetValueOrDefault();
        double valueOrDefault3 = (@params?.CoolDown).GetValueOrDefault();
        bool valueOrDefault4 = (@params?.GoDownWaitSkillEnd).GetValueOrDefault();
        bool? flag = @params?.AllowRefreshTransform;
        bool? obj;
        if (!flag.HasValue)
        {
            WorldEntity? entity = entityHandle2.Entity;
            obj = ((entity != null) ? new bool?(!entity.Active) : null);
        }
        else
        {
            obj = flag;
        }
        bool? flag2 = obj;
        bool valueOrDefault5 = (@params?.ForceInheritTransform).GetValueOrDefault(true);
        RoleTeamComponent.OnChangeRole(entityHandle, entityHandle2, valueOrDefault2, valueOrDefault3, valueOrDefault4, flag2.GetValueOrDefault(), valueOrDefault5);
        if (entityHandle != null)
        {
            Singleton<EventSystem>.Instance.Emit(EEventName.OnRoleGoDown, entityHandle.Id);
        }
        Singleton<EventSystem>.Instance.Emit(EEventName.OnChangeRole, entityHandle2, entityHandle);
    }

    // Token 0x06015138; original public, non-virtual; canonical source lines 1024-1027.
    public double GetChangeRoleCooldown()
    {
        return ChangeRoleCooldown;
    }
}
