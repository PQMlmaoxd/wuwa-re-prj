/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; it is not original source and is not a
 * standalone-compiling whole-class recovery. `partial` is an overlay
 * convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e53 (global::BaseAttributeComponent; raw type row 110)
 * Scope: seven selected R23 ordinary/accessor bodies only (rows 219-225).
 * BaseValues, CurrentValues, ModifierLists, ModifierScratch,
 * NonStateBoundsLockerMap, and ModifierHandleGenerator are shared with the
 * earlier BaseAttributeComponent semantic view and are not redeclared here.
 * Missing members and callees remain external.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;

public partial class BaseAttributeComponent : EntityComponent, IAttributeSet, IStaticVariableResetter
{
    // Observed fields from canonical source lines 168-206 and raw type row 110.
    // Only fields absent from the earlier semantic view are declared here.
    // 0x0400bab7
    private readonly List<IBoundsLocker> BoundsLockerScratch = new List<IBoundsLocker>();

    // 0x0400bab9
    protected readonly Dictionary<EAttributeType, Dictionary<int, IBoundsLocker>> BoundsLockerMap = new Dictionary<EAttributeType, Dictionary<int, IBoundsLocker>>();

    // 0x0400baba
    protected CreatureDataComponent? CreatureDataComponent;

    // 0x0400babb
    protected BaseBuffComponent? BuffComponent;

    // Static-variable fields are declared without invented initializers. Their
    // observed initialization belongs to the unselected type initializer.
    // 0x0400babd
    [StaticVariableRuleIgnore]
    private static readonly Stat UpdateCurrentValueStat;

    // 0x0400babe
    [StaticVariableRuleIgnore]
    private static readonly Stat AddModifierStat;

    // 0x0400babf
    [StaticVariableRuleIgnore]
    private static readonly Stat RemoveModifierStat;

    // 0x0400bac0
    [StaticVariableRuleIgnore]
    private static readonly Stat EvaluateModifiersStat;

    // 0x0400bac1
    [StaticVariableRuleIgnore]
    private static readonly Stat AutoRecoverAttrStat;

    // 0x0400bac2
    protected readonly Dictionary<EAttributeType, HashSet<Action<EAttributeType, float, float>>> CurrentValueListenerMap = new Dictionary<EAttributeType, HashSet<Action<EAttributeType, float, float>>>();

    // 0x0400bac3
    protected readonly HashSet<Action<EAttributeType, float, float>> AnyCurrentValueListenerSet = new HashSet<Action<EAttributeType, float, float>>();

    // 0x0400bac4
    [StaticVariableRuleIgnore]
    private static readonly Stat DispatchCurrentValueEventStat;

    // 0x0400bac5
    [StaticVariableRuleIgnore]
    private static readonly Dictionary<EAttributeType, Stat> StatCurrentAttrEventMap;

    // 0x0400bac6
    [StaticVariableRuleIgnore]
    private static readonly Stat StatAnyCurrentAttrEvent;

    // Token 0x060184ba; original public, non-virtual; canonical source lines 261-280;
    // raw methods row 643.
    public bool IsWritableAttribute(EAttributeType attrId)
    {
        if (CharacterAttributeTypes.attrsAutoRecoverSpeedMap.ContainsKey(attrId) || CharacterAttributeTypes.attrsAutoRecoverMaxMap.ContainsKey(attrId))
        {
            return true;
        }
        if (attrId == EAttributeType.Tough || attrId == EAttributeType.Jump)
        {
            return true;
        }
        if (CharacterAttributeTypes.specialEnergyIds.Contains(attrId))
        {
            if (BuffComponent != null)
            {
                return BuffComponent.HasBuffAuthority();
            }
            return true;
        }
        return false;
    }

    // Token 0x060184bb; original public, non-virtual; canonical source lines 282-297;
    // raw methods row 644.
    public bool IsLocalAttribute(EAttributeType attrId)
    {
        if (attrId == EAttributeType.Jump)
        {
            return true;
        }
        if (CharacterAttributeTypes.specialEnergyIds.Contains(attrId))
        {
            if (BuffComponent != null)
            {
                return BuffComponent.HasBuffAuthority();
            }
            return true;
        }
        return false;
    }

    // Token 0x060184bc; original public, non-virtual; canonical source lines 299-323;
    // raw methods row 645.
    public void SetBaseValue(EAttributeType attrId, float value)
    {
        if (IsWritableAttribute(attrId))
        {
            float num = value;
            float? maxValue = GetMaxValue(attrId);
            if (maxValue.HasValue)
            {
                num = RefreshBounds(attrId, num, maxValue.Value);
            }
            if (!CharacterAttributeTypes.attrsNotClampZero.Contains(attrId))
            {
                num = Math.Max(num, 0f);
            }
            num = (float)Math.Floor(num);
            if (BaseValues[(int)attrId] != num)
            {
                BaseValues[(int)attrId] = num;
                float oldValue = CurrentValues[(int)attrId];
                UpdateCurrentValue(attrId);
                float newValue = CurrentValues[(int)attrId];
                DispatchCurrentValueEvent(attrId, newValue, oldValue);
            }
        }
    }

    // Token 0x060184bd; original public, non-virtual; canonical source lines 325-328;
    // raw methods row 646.
    public void AddBaseValue(EAttributeType attrId, float value)
    {
        SetBaseValue(attrId, BaseValues[(int)attrId] + value);
    }

    // Token 0x060184bf; CLR metadata marks this implicit-interface virtual as
    // final; source visibility is public normal (not an invented C# virtual).
    // Canonical source lines 335-338; raw methods row 648.
    public float GetCurrentValue(EAttributeType attrId)
    {
        return CurrentValues[(int)attrId];
    }

    // Token 0x060184c3; original public, non-virtual; canonical source lines 357-389;
    // raw methods row 652. The six-tuple ReadOnlySpan is a representation-only
    // lift of the observed compiler InlineArray; it is not an original-syntax or
    // ABI claim. Logger receiver evaluation remains before tuple evaluation.
    public void SyncValueFromServer(EAttributeType attrId, float baseValue, float currentValue)
    {
        if (IsLocalAttribute(attrId))
        {
            return;
        }
        if (attrId < EAttributeType.None || attrId >= EAttributeType.Max)
        {
            Log instance = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> details =
            [
                ("CreatureDataId", CreatureDataComponent?.GetCreatureDataId()),
                ("PbDataId", CreatureDataComponent?.GetPbDataId()),
                ("AttributeType", (int)attrId),
                ("BaseValue", baseValue),
                ("CurrentValue", currentValue),
                ("Max", 143)
            ];
            instance.Warn((ELogModule)35, (ELogAuthor)36, "[属性同步]服务端下发越界属性Id，已跳过", details);
        }
        else
        {
            if (BaseValues[(int)attrId] != baseValue)
            {
                BaseValues[(int)attrId] = baseValue;
            }
            float oldValue = CurrentValues[(int)attrId];
            CurrentValues[(int)attrId] = currentValue;
            DispatchCurrentValueEvent(attrId, currentValue, oldValue);
            if (CharacterAttributeTypes.AttributeIdsMaxToAttrId.TryGetValue(attrId, out var value))
            {
                SetBaseValue(value, BaseValues[(int)value]);
            }
        }
    }

    // Token 0x060184e2; original protected virtual; canonical source lines 934-985;
    // raw methods row 683; EH: two System.Exception catches.
    protected virtual void DispatchCurrentValueEventImplement(EAttributeType attrId, float newValue, float oldValue)
    {
        Action<EAttributeType, float, float>[] array2;
        if (CurrentValueListenerMap.TryGetValue(attrId, out HashSet<Action<EAttributeType, float, float>> value))
        {
            if (!StatCurrentAttrEventMap.TryGetValue(attrId, out Stat value2))
            {
                value2 = null;
                StatCurrentAttrEventMap[attrId] = value2;
            }
            Action<EAttributeType, float, float>[] array = new Action<EAttributeType, float, float>[value.Count];
            value.CopyTo(array);
            array2 = array;
            foreach (Action<EAttributeType, float, float> action in array2)
            {
                if (value.Contains(action))
                {
                    try
                    {
                        action(attrId, newValue, oldValue);
                    }
                    catch (Exception e)
                    {
                        CombatLog instance = Singleton<CombatLog>.Instance;
                        Entity entity = base.Entity;
                        (string, object) reference = ("属性", attrId);
                        instance.ErrorWithStack(CombatLog.EDebugModule.Attribute, entity, "属性回调异常", e, new ReadOnlySpan<(string, object)>(ref reference));
                    }
                }
            }
        }
        Action<EAttributeType, float, float>[] array3 = new Action<EAttributeType, float, float>[AnyCurrentValueListenerSet.Count];
        AnyCurrentValueListenerSet.CopyTo(array3);
        array2 = array3;
        foreach (Action<EAttributeType, float, float> action2 in array2)
        {
            if (AnyCurrentValueListenerSet.Contains(action2))
            {
                try
                {
                    action2(attrId, newValue, oldValue);
                }
                catch (Exception e2)
                {
                    CombatLog instance2 = Singleton<CombatLog>.Instance;
                    Entity entity2 = base.Entity;
                    (string, object) reference2 = ("属性", attrId);
                    instance2.ErrorWithStack(CombatLog.EDebugModule.Attribute, entity2, "全局属性回调异常", e2, new ReadOnlySpan<(string, object)>(ref reference2));
                }
            }
        }
    }
}
