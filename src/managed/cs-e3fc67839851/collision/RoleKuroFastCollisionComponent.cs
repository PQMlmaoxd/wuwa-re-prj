/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003129 (global::RoleKuroFastCollisionComponent; raw type row 94)
 * Original base: BaseKuroFastCollisionComponent.
 * Scope: nine selected whole source-level method bodies only.
 * ClearComponent and other lifecycle members remain external;
 * see METHOD_EVIDENCE_MAP.md.
 */

using System.Collections.Generic;

public partial class RoleKuroFastCollisionComponent : BaseKuroFastCollisionComponent
{
    // Field 0x0400d0f6; canonical source line 5.
    protected BaseTagComponent? TagComp;

    // Field 0x0400d0f7; canonical source line 7.
    private bool HasDodgeTag;

    // Field 0x0400d0f8; canonical source line 9.
    private bool HasUltimateDodgeTag;

    // Field 0x0400d0f9; canonical source line 11.
    private readonly List<ITagTask> TagTaskList = new List<ITagTask>();

    // Field 0x0400d0fa; canonical source line 13.
    private bool IsListening;

    // Token 0x0601a0de; original public, non-virtual; canonical source lines 15-22.
    public bool CanDodge()
    {
        if (HasDodgeTag)
        {
            return !HasUltimateDodgeTag;
        }
        return false;
    }

    // Token 0x0601a0df; original protected virtual; canonical source lines 24-28.
    protected override bool OnStart()
    {
        TagComp = base.Entity.GetComponent<BaseTagComponent>();
        return base.OnStart();
    }

    // Token 0x0601a0e0; original protected virtual; canonical source lines 30-48.
    protected override void OnStartKuroFastCollision()
    {
        if (IsListening)
        {
            return;
        }
        IsListening = true;
        if (TagComp != null)
        {
            HasDodgeTag = TagComp.HasTag(GameplayTagDefine.EGameplayTagId["功能.逻辑状态标识.无敌.角色闪避"]);
            HasUltimateDodgeTag = TagComp.HasTag(GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.极限闪避"]);
            ListenForTagAddOrRemoveChanged(GameplayTagDefine.EGameplayTagId["功能.逻辑状态标识.无敌.角色闪避"], OnDodgeTagChanged);
            ListenForTagAddOrRemoveChanged(GameplayTagDefine.EGameplayTagId["角色.Common.技能通用标识.极限闪避"], OnUltimateDodgeTagChanged);
            if (CanDodge())
            {
                UpdateDodgeState();
            }
        }
    }

    // Token 0x0601a0e1; original protected virtual; canonical source lines 50-57.
    protected override void OnStopKuroFastCollision()
    {
        if (IsListening)
        {
            IsListening = false;
            ClearAllTagCountChangedCallback();
        }
    }

    // Token 0x0601a0e2; original private, non-virtual; canonical source lines 59-63.
    private void OnDodgeTagChanged(int tagId, bool tagExist)
    {
        HasDodgeTag = tagExist;
        UpdateDodgeState();
    }

    // Token 0x0601a0e3; original private, non-virtual; canonical source lines 65-69.
    private void OnUltimateDodgeTagChanged(int tagId, bool tagExist)
    {
        HasUltimateDodgeTag = tagExist;
        UpdateDodgeState();
    }

    // Token 0x0601a0e4; original private, non-virtual; canonical source lines 71-81.
    private void ListenForTagAddOrRemoveChanged(int tagId, BaseTagComponent.TTagSwitchedCallback onTagChange)
    {
        if (TagComp != null)
        {
            ITagTask tagTask = TagComp.ListenForTagAddOrRemove(tagId, onTagChange);
            if (tagTask != null)
            {
                TagTaskList.Add(tagTask);
            }
        }
    }

    // Token 0x0601a0e5; original private, non-virtual; canonical source lines 83-90.
    // EH: enumerator finally (try 12..37; handler 37..51).
    private void ClearAllTagCountChangedCallback()
    {
        foreach (ITagTask tagTask in TagTaskList)
        {
            tagTask.EndTask();
        }
        TagTaskList.Clear();
    }

    // Token 0x0601a0e6; original private, non-virtual; canonical source lines 92-95.
    private void UpdateDodgeState()
    {
        ModelBase<BulletModel>.Instance.GetBulletHitWorldEntityManager()?.SetEntityCanDodge(base.Entity.Id, CanDodge());
    }
}
