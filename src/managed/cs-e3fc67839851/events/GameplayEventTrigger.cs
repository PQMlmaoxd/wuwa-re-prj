/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002fd6 (global::GameplayEventTrigger)
 * Scope: four selected whole method bodies only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using CSharpScript.Utils;
using UnrealEngine;

public partial class GameplayEventTrigger : Trigger
{
    // Observed field; constructor and the rest of Trigger remain external to this view.
    protected int TagId;

    // Token 0x06018f67; original public, virtual override.
    public override void OnInitParams(string[] triggerParams)
    {
        string tagName = triggerParams.Length != 0 ? triggerParams[0].Trim() : string.Empty;
        TagId = GameplayTagUtils.GetTagIdByName(tagName);
    }

    // Token 0x06018f68; original protected, virtual override.
    protected override void OnActive()
    {
        (base.OwnerTriggerComp?.Entity?.GetComponent<BaseAbilityComponent>())?.AddGameplayEventListener(TagId, OnEvent);
    }

    // Token 0x06018f69; original protected, virtual override.
    protected override void OnInactive()
    {
        (base.OwnerTriggerComp?.Entity?.GetComponent<BaseAbilityComponent>())?.RemoveGameplayEventListener(TagId, OnEvent);
    }

    // Token 0x06018f6a; original private, non-virtual.
    private void OnEvent(int _, FGameplayEventData payload)
    {
        if (base.Checker == null || base.Checker())
        {
            WorldEntity targetEntity =
                payload.Target == null
                    ? null
                    : ActorUtils.GetEntityByActor(payload.Target, showLog: false)?.Entity;
            EvaluateAndExecute(new Dictionary<string, TFormulaValue>
            {
                ["Target"] = targetEntity
            });
        }
    }
}
