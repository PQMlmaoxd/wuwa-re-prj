/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; it is not original source and is not a
 * standalone-compiling whole-class recovery. `partial` is an overlay
 * convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e93 (global::CharacterAttributeComponent; raw type row 39)
 * Scope: four selected R23 ordinary method bodies only (rows 226-229).
 * InitializeGASAttributes and TsAbilitySystemComponent are the only selected
 * field declarations; the hidden BuffComponent property and other members
 * remain external.
 */

using System.Collections.Generic;
using Aki.Protocol;
using CSharpScript.Game.Common.Event;
using Google.Protobuf.Collections;
using UnrealEngine;

public partial class CharacterAttributeComponent : BaseAttributeComponent
{
    // 0x0400bcb5; actual initializer observed in the canonical declaration.
    protected bool InitializeGASAttributes = true;

    // 0x0400bcb6; no initializer observed in the canonical declaration.
    private UBaseAbilitySystemComponent? TsAbilitySystemComponent;

    // Token 0x060187a6; original private, non-virtual; canonical source lines 95-110;
    // raw methods row 766; enumerator finally retained by the source foreach.
    private void OnAnyCurrentValueChange(EAttributeType attrId, float newValue, float oldValue)
    {
        bool flag = false;
        foreach (EAttributeType value in CharacterAttributeTypes.attributeIdsWithMax.Values)
        {
            if (value == attrId)
            {
                flag = true;
                break;
            }
        }
        if (CharacterAttributeTypes.stateAttributeIds.Contains(attrId) || flag)
        {
            ApplyModToAttributeToAbilitySystem((int)attrId, EGameplayModOp.Override, newValue);
        }
    }

    // Token 0x060187a7; original public static, non-virtual; canonical source lines 112-132;
    // raw methods row 767; enumerator finally retained by the source foreach.
    [CombatListen(ENotifyMessageId.AttributeChangedNotify, true, false)]
    public static void AttributeChangedNotify(Entity? entity, AttributeChangedNotify data, CombatCommon? combatCommon = null)
    {
        BaseAttributeComponent baseAttributeComponent = entity?.GetComponent<BaseAttributeComponent>();
        if (baseAttributeComponent == null)
        {
            return;
        }
        foreach (GameplayAttributeData attribute in data.Attributes)
        {
            if (CharacterAttributeTypes.stateAttributeIds.Contains((EAttributeType)attribute.AttributeType))
            {
                attribute.CurrentValue = attribute.BaseValue;
            }
            if (attribute.AttributeType != 0)
            {
                baseAttributeComponent.SyncValueFromServer((EAttributeType)attribute.AttributeType, attribute.BaseValue, attribute.CurrentValue);
            }
        }
        Singleton<EventSystem>.Instance.Emit(EEventName.OnServerAttributeChange, entity.Id, data);
    }

    // Token 0x060187ae; original public, non-virtual; canonical source lines 202-208;
    // raw methods row 774; native/GAS boundary remains external.
    public void ApplyModToAttributeToAbilitySystem(int attributeId, EGameplayModOp modifierOp, float modifierMagnitude)
    {
        if (InitializeGASAttributes)
        {
            TsAbilitySystemComponent?.InternalApplyModToAttribute(attributeId, modifierOp, modifierMagnitude);
        }
    }

    // Token 0x060187af; original protected virtual; canonical source lines 210-214;
    // raw methods row 775; base dispatch precedes the ability event.
    protected override void DispatchCurrentValueEventImplement(EAttributeType attrId, float newValue, float oldValue)
    {
        base.DispatchCurrentValueEventImplement(attrId, newValue, oldValue);
        ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.OnAttributeChange, (long)attrId, attrId, base.Entity, newValue, oldValue);
    }
}
