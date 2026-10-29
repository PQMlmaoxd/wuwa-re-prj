/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003110 (global::BaseSkillCdComponent)
 * Scope: one selected whole method body only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System.Collections.Generic;
using CSharpScript.Game.Module.Battle;

public partial class BaseSkillCdComponent : EntityComponent
{
    // Observed member used by the selected method.
    private Dictionary<long, PassiveSkillCdInfo>? PassiveSkillCdInfoMap;

    // Token 0x06019f90; original public, non-virtual.
    public bool StartPassiveCd(long skillId, int entityId, float cdTime = -1f)
    {
        if (!PassiveSkillCdInfoMap.TryGetValue(skillId, out PassiveSkillCdInfo info))
        {
            return false;
        }
        info.StartCd(info.SkillId, entityId, cdTime);
        return true;
    }
}
