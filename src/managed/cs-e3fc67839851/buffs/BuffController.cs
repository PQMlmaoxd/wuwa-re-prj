/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; it is not original source and is not a
 * standalone-compiling whole-class recovery. `partial` is an overlay
 * convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e9c (global::BuffController; raw type row 40)
 * Scope: one selected R23 ordinary method body only (row 230).
 * Configuration, cache, AddBuffRef, and all other controller members remain
 * external. This view makes no claim about configuration-row contents.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Core.Common;
using CSharpScript.Core.Config;
using CSharpScript.Core.Framework;
using UnrealEngine;

[TickController(0)]
public partial class BuffController : ControllerBase<BuffController>
{
    // Token 0x06018810; original public, non-virtual; canonical source lines 124-146;
    // raw methods row 790. The two-tuple ReadOnlySpan is a representation-only
    // lift of the observed compiler InlineArray; it is not an original-syntax or
    // ABI claim. Logger receiver evaluation remains before tuple evaluation.
    public BuffDefinition? GetBuffDefinition(long buffId, string? reason = null)
    {
        BuffDefinition buffDefinition = ModelBase<BuffModel>.Instance.Get(buffId);
        if (buffDefinition != null)
        {
            return buffDefinition;
        }
        if (buffId == -3)
        {
            return null;
        }
        Buff? config = ConfigBuffById.GetConfig(buffId);
        if (!config.HasValue)
        {
            CombatLog instance = Singleton<CombatLog>.Instance;
            ReadOnlySpan<(string, object)> details =
            [
                ("BuffId", buffId),
                ("原因", reason ?? "")
            ];
            instance.Error(CombatLog.EDebugModule.Buff, null, "无法查找到对应编号的Buff。", details);
            return null;
        }
        return AddBuffRef(config.Value);
    }
}
