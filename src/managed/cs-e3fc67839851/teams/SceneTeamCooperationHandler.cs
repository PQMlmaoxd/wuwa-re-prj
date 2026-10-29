/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020017b1 (global::SceneTeamCooperationHandler)
 * Canonical source: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/source/_global/SceneTeamCooperationHandler.cs
 * Canonical raw methods: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/evidence/raw/methods.jsonl
 * Canonical raw types: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/evidence/raw/types.jsonl
 * Scope: selected whole source-level body for R22 row 200 only.
 * Clear remains external; no empty implementation is invented.
 */

using System;
using System.Collections.Generic;

public partial class SceneTeamCooperationHandler : ICooperationHandler
{
    // MethodDef 0x0600ab21; public final virtual; source lines 6-34; raw row 103.
    public bool Trigger(SceneTeamItem goDownRole, SceneTeamItem goBattleRole)
    {
        EntityHandle? entityHandle = goBattleRole.EntityHandle;
        bool isInQte = entityHandle.Entity.GetComponent<RoleQteComponent>().IsInQte;
        bool flag = entityHandle.Entity.CheckGetComponent<RoleTeamComponent>().IsChangeRoleCoolDown();
        if (!isInQte)
        {
            if (flag)
            {
                ControllerBase<ScrollingTipsController>.Instance.ShowTipsById("EditBattleTeamInCD");
                return false;
            }
            BaseTagComponent component = goDownRole.EntityHandle.Entity.GetComponent<BaseTagComponent>();
            if (component.HasTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.被击硬直时间"]) && component.HasAnyTag((IEnumerable<int>)SceneTeamDefine.beHitTagList))
            {
                Log instance = Singleton<Log>.Instance;
                (string, object) reference = ("roleId", goDownRole.GetConfigId);
                instance.Info((ELogModule)43, (ELogAuthor)49, "被击硬直时间无法换人", new ReadOnlySpan<(string, object)>(ref reference));
                return false;
            }
        }
        ControllerBase<SceneTeamController>.Instance.RequestChangeRole(goBattleRole.GetCreatureDataId(), new RequestChangeRoleParams
        {
            FilterSameRole = true,
            GoDownWaitSkillEnd = true,
            ForceInheritTransform = false
        });
        return true;
    }
}