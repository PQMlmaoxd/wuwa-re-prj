/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200295e (global::SceneTeamItem)
 * Scope: one selected whole source-level method body only.
 * Missing members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

public partial class SceneTeamItem
{
    private bool IsAutoRoleInternal;

    // Token 0x060150ba; original public, non-virtual; canonical source lines 146-166.
    public bool CanControl(bool skipCheckDead = false)
    {
        WorldEntity worldEntity = EntityHandle?.Entity;
        if (worldEntity == null)
        {
            return false;
        }
        if (!skipCheckDead)
        {
            BaseDeathComponent component = worldEntity.GetComponent<BaseDeathComponent>();
            if (component == null || component.IsDead())
            {
                return false;
            }
        }
        if (IsAutoRoleInternal)
        {
            return false;
        }
        return true;
    }
}
