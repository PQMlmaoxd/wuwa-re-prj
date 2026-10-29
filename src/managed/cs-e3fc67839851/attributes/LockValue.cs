/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002f34 (global::LockValue)
 * Scope: three selected whole method bodies only; constructor, OnExecute, and
 * GetDebugEffectString remain external and are not reconstructed here.
 */

using Aki.Protocol;

public partial class LockValue : BuffEffect
{
    protected EAttributeType AttributeId;
    protected float Offset;
    protected float Percent;

    // Token 0x06018bc2; original protected, override; canonical source lines 16-35.
    protected override void InitParameters(ExtraEffectParameters parameters)
    {
        string[] extraEffectParameters_ = parameters.ExtraEffectParameters_;
        if (extraEffectParameters_ != null)
        {
            if (extraEffectParameters_.Length != 0)
            {
                AttributeId = (EAttributeType)int.Parse(extraEffectParameters_[0]);
            }
            if (extraEffectParameters_.Length > 1)
            {
                Offset = float.Parse(extraEffectParameters_[1]);
            }
            Percent = 0f;
            if (extraEffectParameters_.Length > 2)
            {
                Percent = float.Parse(extraEffectParameters_[2]);
            }
        }
    }

    // Token 0x06018bc3; original public, override; canonical source lines 37-51.
    public override void OnCreated()
    {
        BaseAttributeComponent baseAttributeComponent = base.OwnerEntity?.CheckGetComponent<BaseAttributeComponent>();
        if (baseAttributeComponent != null)
        {
            if (CharacterAttributeTypes.stateAttributeIds.Contains(AttributeId))
            {
                baseAttributeComponent.AddStateAttributeLock(ActiveHandleId, AttributeId, Percent, Offset);
            }
            else
            {
                baseAttributeComponent.AddNonStateAttributeLock(ActiveHandleId, AttributeId, Offset);
            }
        }
    }

    // Token 0x06018bc5; original public, override; canonical source lines 58-72.
    public override void OnRemoved(bool bPremature)
    {
        BaseAttributeComponent baseAttributeComponent = base.OwnerEntity?.CheckGetComponent<BaseAttributeComponent>();
        if (baseAttributeComponent != null)
        {
            if (CharacterAttributeTypes.stateAttributeIds.Contains(AttributeId))
            {
                baseAttributeComponent.RemoveStateAttributeLock(ActiveHandleId, AttributeId);
            }
            else
            {
                baseAttributeComponent.RemoveNonStateAttributeLock(ActiveHandleId, AttributeId);
            }
        }
    }
}
