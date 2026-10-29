/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020017b0 (global::QteCooperationHandler)
 * Canonical source: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/source/_global/QteCooperationHandler.cs
 * Canonical raw methods: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/evidence/raw/methods.jsonl
 * Canonical raw types: .../f013d5a273ab972d20138fa6303ab21312b78ce3fe7048cbf1db33ce76512324/evidence/raw/types.jsonl
 * Scope: selected whole source-level body for R22 row 199 only.
 * Clear remains external; no empty implementation is invented.
 * The lowered two-element tag InlineArray is represented as a source span
 * literal solely to preserve observed tag order/count.
 */

using System;
using AkiClient.Game.Aki.Data.Fight.Struct;

public partial class QteCooperationHandler : ICooperationHandler
{
    // MethodDef 0x0600ab1e; public final virtual; source lines 6-42; raw row 100.
    public bool Trigger(SceneTeamItem goDownRole, SceneTeamItem goBattleRole)
    {
        if (!goBattleRole.IsMyRole())
        {
            Log instance = Singleton<Log>.Instance;
            (string, object) reference = ("CreatureDataId", goBattleRole.GetCreatureDataId());
            instance.Info((ELogModule)43, (ELogAuthor)49, "上场角色为其他玩家的角色", new ReadOnlySpan<(string, object)>(ref reference));
            ControllerBase<SceneTeamController>.Instance.TryUseMultiQte(goBattleRole.EntityHandle);
            return true;
        }
        EntityHandle entityHandle = goDownRole.EntityHandle;
        EntityHandle entityHandle2 = goBattleRole.EntityHandle;
        RoleQteComponent component = entityHandle.Entity.GetComponent<RoleQteComponent>();
        RoleQteComponent component2 = entityHandle2.Entity.GetComponent<RoleQteComponent>();
        BaseTagComponent component3 = entityHandle.Entity.GetComponent<BaseTagComponent>();
        if (component == null || component2 == null)
        {
            return false;
        }
        SQteTag qteTagData = component2.GetQteTagData();
        if (qteTagData == null)
        {
            return false;
        }
        // Source-only lift of the compiler-lowered InlineArray2<int>.
        ReadOnlySpan<int> tagIds =
        [
            GameplayTagDefine.EGameplayTagId["行为状态.位置状态.攀爬"],
            GameplayTagDefine.EGameplayTagId["行为状态.位置状态.水中"]
        ];
        bool num = !component3.HasAnyTag(in tagIds) && component2.IsQteReady(entityHandle);
        bool flag = false;
        if (num)
        {
            component.UseExitSkill(entityHandle2);
            flag = component2.ExecuteQte(entityHandle);
        }
        return !(flag ? qteTagData.ChangeRoleOnQte : qteTagData.ChangeRole);
    }
}