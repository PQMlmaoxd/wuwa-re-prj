/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003130 (global::SkillBehaviorMisc)
 * Scope: one selected whole source-level method body only.
 * Static initialization/reset helpers and missing members remain external;
 * see METHOD_EVIDENCE_MAP.md.
 */

using System.Collections.Generic;
using CSharpScript.Game.Utils;

public partial class SkillBehaviorMisc : IStaticVariableResetter
{
    // Observed field 0x0400d138; no initializer is supplied in this source view.
    public static Dictionary<Skill, List<EndSkillBehaviorParam>> ParamMap;

    // EndSkillBehaviorParam (TypeDef 0x02003135) remains an external type;
    // canonical context: EndSkillBehaviorParam.cs:4-20.

    // Token 0x0601a13b; original public static, non-virtual; canonical source lines 36-43.
    public static List<EndSkillBehaviorParam> GetEndSkillBehaviorParamList(Skill skill)
    {
        if (!ParamMap.ContainsKey(skill))
        {
            ParamMap.Add(skill, new List<EndSkillBehaviorParam>());
        }
        return ParamMap[skill];
    }
}
