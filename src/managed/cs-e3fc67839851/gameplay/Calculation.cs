/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002eb2 (global::Calculation; raw type row 29)
 * Scope: one selected whole source-level method body only. The remaining
 * Calculation members and static machinery remain external/unselected.
 * The original type is `public class`, not a static class; `_formulas`,
 * construction, registration, and reset members are not reconstructed here.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;

public partial class Calculation : IStaticVariableResetter
{
    // Token 0x0601886e; original public static, non-virtual; canonical source lines 142-145.
    // Raw EAttributeType operands 87, 88, and 107 retain their canonical names;
    // their numeric enum identities remain unproved independently here.
    public static float ToughCalculation(AttributeSnapshot attackerAttr, AttributeSnapshot targetAttr, float rate)
    {
        return rate * (attackerAttr.GetCurrentValue(EAttributeType.ToughChange) * 0.0001f) * (targetAttr.GetCurrentValue(EAttributeType.ToughReduce) * 0.0001f) * (targetAttr.GetCurrentValue(EAttributeType.SkillToughRatio) * 0.0001f);
    }
}
