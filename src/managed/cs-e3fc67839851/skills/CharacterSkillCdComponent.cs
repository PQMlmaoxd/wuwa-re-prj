/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003122 (global::CharacterSkillCdComponent)
 * Scope: one selected whole method body only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System.Collections.Generic;

public partial class CharacterSkillCdComponent : BaseSkillCdComponent
{
    // Observed member used by the selected method.
    private Dictionary<long, GroupSkillCdInfo>? GroupSkillCdInfoMap;

    // Token 0x0601a090; original public, non-virtual.
    public void ModifyCdTime(long[]? skillIds, float modifyTime, float changeTimePercentage)
    {
        if (skillIds == null || skillIds.Length == 0)
        {
            return;
        }
        if (skillIds.Length == 1)
        {
            if (GroupSkillCdInfoMap.TryGetValue(skillIds[0], out GroupSkillCdInfo group))
            {
                group.ModifyRemainingCd(modifyTime, changeTimePercentage);
            }
            return;
        }

        HashSet<GroupSkillCdInfo> groups = new HashSet<GroupSkillCdInfo>();
        foreach (long skillId in skillIds)
        {
            if (GroupSkillCdInfoMap.TryGetValue(skillId, out GroupSkillCdInfo group))
            {
                groups.Add(group);
            }
        }
        foreach (GroupSkillCdInfo group in groups)
        {
            group.ModifyRemainingCd(modifyTime, changeTimePercentage);
        }
    }
}
