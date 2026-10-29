/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e53 (global::BaseAttributeComponent)
 * Scope: eleven selected whole method bodies only.
 * Unknown members and callees remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;

public partial class BaseAttributeComponent : EntityComponent, IAttributeSet, IStaticVariableResetter
{
    // Observed storage declarations used by the selected view.
    protected readonly float[] BaseValues = new float[143];
    protected readonly float[] CurrentValues = new float[143];
    protected readonly Dictionary<int, CharacterAttributeTypes.IModifier>[] ModifierLists = new Dictionary<int, CharacterAttributeTypes.IModifier>[143];
    private readonly List<CharacterAttributeTypes.IModifier> ModifierScratch = new List<CharacterAttributeTypes.IModifier>();
    protected readonly Dictionary<EAttributeType, Dictionary<int, IBoundsLocker>> NonStateBoundsLockerMap = new Dictionary<EAttributeType, Dictionary<int, IBoundsLocker>>();
    public static int ModifierHandleGenerator;

    // Token 0x060184c4; original public, virtual; canonical source lines 391-410.
    public virtual void UpdateCurrentValue(EAttributeType attrId)
    {
        if (IsWritableAttribute(attrId))
        {
            float originValue = EvaluateModifiers(attrId);
            originValue = RefreshNonStateBounds(attrId, originValue);
            if (CharacterAttributeTypes.attrsCurrentValueClamp.TryGetValue(attrId, out var maxValue))
            {
                originValue = Math.Min(originValue, maxValue);
            }
            if (!CharacterAttributeTypes.attrsNotClampZero.Contains(attrId))
            {
                originValue = Math.Max(originValue, 0f);
            }
            if (CurrentValues[(int)attrId] != originValue)
            {
                CurrentValues[(int)attrId] = originValue;
            }
        }
    }

    // Token 0x060184c6; original public, non-virtual; canonical source lines 420-438.
    public int AddModifier(EAttributeType attrId, CharacterAttributeTypes.IModifier modifier)
    {
        if (!IsWritableAttribute(attrId))
        {
            return -1;
        }
        if (attrId <= EAttributeType.None || attrId >= EAttributeType.Max)
        {
            return -1;
        }
        int handle = ModifierHandleGenerator++;
        ModifierLists[(int)attrId] = ModifierLists[(int)attrId] ?? new Dictionary<int, CharacterAttributeTypes.IModifier>();
        ModifierLists[(int)attrId][handle] = modifier;
        float oldValue = CurrentValues[(int)attrId];
        UpdateCurrentValue(attrId);
        float newValue = CurrentValues[(int)attrId];
        DispatchCurrentValueEvent(attrId, newValue, oldValue);
        return handle;
    }

    // Token 0x060184c7; original public, non-virtual; canonical source lines 440-449.
    public void RemoveModifier(EAttributeType attrId, int handle)
    {
        if (ModifierLists[(int)attrId] != null && ModifierLists[(int)attrId].Remove(handle))
        {
            float oldValue = CurrentValues[(int)attrId];
            UpdateCurrentValue(attrId);
            float newValue = CurrentValues[(int)attrId];
            DispatchCurrentValueEvent(attrId, newValue, oldValue);
        }
    }

    // Token 0x060184c8; original public, virtual; canonical source lines 451-462.
    public virtual void CollectModifiers(EAttributeType attrId, List<CharacterAttributeTypes.IModifier> output)
    {
        Dictionary<int, CharacterAttributeTypes.IModifier> dictionary = ModifierLists[(int)attrId];
        if (dictionary == null)
        {
            return;
        }
        foreach (CharacterAttributeTypes.IModifier modifier in dictionary.Values)
        {
            output.Add(modifier);
        }
    }

    // Token 0x060184c9; original private, non-virtual; canonical source lines 464-555.
    private float EvaluateModifiers(EAttributeType attrId)
    {
        float baseValue = BaseValues[(int)attrId];
        if (ModifierLists[(int)attrId] == null)
        {
            return baseValue;
        }
        float flatValue = 0f;
        float rateValue = 0f;
        float advancedFactor = 1f;
        bool needAdvancedMultiply = CheckIfNeedAdvanceMultiply(attrId);
        ModifierScratch.Clear();
        CollectModifiers(attrId, ModifierScratch);
        foreach (CharacterAttributeTypes.IModifier modifier in ModifierScratch)
        {
            if (!(modifier is CharacterAttributeTypes.AttributeModifier attributeModifier))
            {
                continue;
            }
            float modifierValue = 0f;
            switch (attributeModifier.Type)
            {
            case ECalculationPolicyType.AddValue:
                modifierValue = attributeModifier.Value1;
                break;
            case ECalculationPolicyType.AddRate:
                rateValue += attributeModifier.Value1;
                break;
            case ECalculationPolicyType.AddFromAttr:
            case ECalculationPolicyType.OverrideFromAttr:
            case ECalculationPolicyType.AddFromAttrRate:
            {
                float sourceValue = attributeModifier.SnapshotSource.GetValueOrDefault();
                if (sourceValue == 0f)
                {
                    sourceValue = AbilityUtils.GetAttrValue(
                        (attributeModifier.SourceEntity == 0L)
                            ? this
                            : ModelBase<CreatureModel>.Instance.GetEntity(attributeModifier.SourceEntity)?.Entity?.GetComponent<BaseAttributeComponent>(),
                        attributeModifier.SourceAttributeId,
                        attributeModifier.SourceCalculationType);
                }
                float minimum = attributeModifier.Min.GetValueOrDefault();
                if (minimum != 0f)
                {
                    sourceValue -= minimum;
                    if (sourceValue <= 0f)
                    {
                        break;
                    }
                }
                float ratio = attributeModifier.Ratio.GetValueOrDefault();
                if (ratio != 0f)
                {
                    sourceValue /= ratio;
                    if (attributeModifier.FloorAfterRatio)
                    {
                        sourceValue = (float)Math.Floor(sourceValue);
                    }
                }
                if (attributeModifier.Type == ECalculationPolicyType.AddFromAttrRate)
                {
                    rateValue += sourceValue * attributeModifier.Value1 * 0.0001f;
                    break;
                }
                modifierValue = sourceValue * attributeModifier.Value1 * 0.0001f + attributeModifier.Value2;
                float maximum = attributeModifier.Max.GetValueOrDefault();
                if (maximum != 0f && modifierValue > maximum)
                {
                    modifierValue = maximum;
                }
                if (attributeModifier.Type != ECalculationPolicyType.OverrideFromAttr)
                {
                    break;
                }
                return modifierValue;
            }
            case ECalculationPolicyType.OverrideValue:
                return attributeModifier.Value1;
            case ECalculationPolicyType.AdvancedMultiplyMagnitude1:
                advancedFactor *= attributeModifier.Value1 * 0.0001f;
                break;
            }
            if (modifierValue != 0f)
            {
                if (!needAdvancedMultiply)
                {
                    flatValue += modifierValue;
                }
                else
                {
                    advancedFactor *= modifierValue * 0.0001f + 1f;
                }
            }
        }
        return (float)Math.Floor((baseValue * (rateValue * 0.0001f + 1f) + flatValue) * advancedFactor);
    }

    // Token 0x060184d6; original private; canonical source lines 774-805.
    private float RefreshNonStateBounds(EAttributeType attrId, float originValue)
    {
        if (!NonStateBoundsLockerMap.TryGetValue(attrId, out Dictionary<int, IBoundsLocker> value) || value.Count == 0)
        {
            return originValue;
        }
        float result = originValue;
        float? lower = null;
        float? upper = null;
        foreach (IBoundsLocker locker in GetAllNonStateBoundsLocker(attrId))
        {
            if (locker.LockLowerBounds)
            {
                float lowerOffset = locker.LowerOffset;
                lower = Math.Max(lower.GetValueOrDefault(lowerOffset), lowerOffset);
            }
            if (locker.LockUpperBounds)
            {
                float upperOffset = locker.UpperOffset;
                upper = Math.Min(upper.GetValueOrDefault(upperOffset), upperOffset);
            }
        }
        if (lower.HasValue)
        {
            result = Math.Max(lower.Value, result);
        }
        if (upper.HasValue)
        {
            result = Math.Min(upper.Value, result);
        }
        return result;
    }

    // Token 0x060184d7; original public; canonical source lines 807-832.
    public int AddNonStateBoundsLocker(EAttributeType attrId, IBoundsLocker locker, int lockerHandle)
    {
        if (!IsWritableAttribute(attrId))
        {
            return -1;
        }
        if (!NonStateBoundsLockerMap.TryGetValue(attrId, out Dictionary<int, IBoundsLocker> value))
        {
            value = new Dictionary<int, IBoundsLocker>();
            NonStateBoundsLockerMap[attrId] = value;
        }
        if (value.ContainsKey(lockerHandle))
        {
            Log instance = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> arguments = new (string, object)[2]
            {
                ("attrId", attrId),
                ("handle", lockerHandle)
            };
            instance.Error((ELogModule)35, (ELogAuthor)103, "重复添加非状态属性BoundsLock", arguments);
            return lockerHandle;
        }
        value[lockerHandle] = locker;
        float oldValue = CurrentValues[(int)attrId];
        UpdateCurrentValue(attrId);
        DispatchCurrentValueEvent(attrId, CurrentValues[(int)attrId], oldValue);
        return lockerHandle;
    }

    // Token 0x060184d8; original public; canonical source lines 834-848.
    public bool RemoveNonStateBoundsLocker(EAttributeType attrId, int lockerHandle)
    {
        if (!NonStateBoundsLockerMap.TryGetValue(attrId, out Dictionary<int, IBoundsLocker> value))
        {
            return false;
        }
        if (value.Remove(lockerHandle))
        {
            float oldValue = CurrentValues[(int)attrId];
            UpdateCurrentValue(attrId);
            DispatchCurrentValueEvent(attrId, CurrentValues[(int)attrId], oldValue);
            return true;
        }
        return false;
    }

    // Token 0x060184d9; original public; canonical source lines 850-873.
    public void AddNonStateAttributeLock(int handleId, EAttributeType attributeId, float offset)
    {
        if (CharacterAttributeTypes.stateAttributeIds.Contains(attributeId))
        {
            Log instance = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> arguments = new (string, object)[2]
            {
                ("attrId", attributeId),
                ("handle", handleId)
            };
            instance.Error((ELogModule)35, (ELogAuthor)103, "非状态属性锁定的属性不是非状态属性", arguments);
        }
        else
        {
            BoundsLocker locker = new BoundsLocker
            {
                LockUpperBounds = true,
                LockLowerBounds = true,
                UpperPercent = 0f,
                UpperOffset = offset,
                LowerPercent = 0f,
                LowerOffset = offset
            };
            AddNonStateBoundsLocker(attributeId, locker, handleId);
        }
    }

    // Token 0x060184da; original public; canonical source lines 875-878.
    public void RemoveNonStateAttributeLock(int handleId, EAttributeType attributeId)
    {
        RemoveNonStateBoundsLocker(attributeId, handleId);
    }

    // Token 0x060184e1; original public; canonical source lines 926-932.
    public void DispatchCurrentValueEvent(EAttributeType attrId, float newValue, float oldValue)
    {
        if (oldValue != newValue)
        {
            DispatchCurrentValueEventImplement(attrId, newValue, oldValue);
        }
    }
}
