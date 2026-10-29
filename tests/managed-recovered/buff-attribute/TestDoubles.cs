// Authored test doubles for RE-BUFF-DIRECT-TEST01.
// This file contains NO recovered method body. It supplies synthetic storage,
// contracts, observers and log sinks required to execute exact recovered source
// slices that are generated into the private scratch fixture.
using System;
using System.Collections.Generic;

// Synthetic IDs only. These are deliberately not the game's EAttributeType values.
public enum EAttributeType
{
    TEST_DOUBLE_101 = 101,
    TEST_DOUBLE_102 = 102,
    TEST_DOUBLE_103 = 103,
    TEST_DOUBLE_110 = 110,
    TEST_DOUBLE_120 = 120,
    TEST_DOUBLE_130 = 130,
    TEST_DOUBLE_140 = 140,
    TEST_DOUBLE_142 = 142,
    TEST_DOUBLE_INVALID_200 = 200
}

// Recovered enum names/values as read from the selected enum shard.
public enum ECalculationPolicyType
{
    AdvancedMultiplyMagnitude1 = -1,
    AddValue = 0,
    AddRate = 1,
    AddFromAttr = 2,
    OverrideValue = 3,
    OverrideFromAttr = 4,
    AddTimeScaledValue = 5,
    AddTimeScaledRate = 6,
    AddFromAttrRate = 9
}

// Recovered enum names/values as read from the selected enum shard.
public enum EAttributeBasedFloatCalculationType
{
    BaseValue,
    CurrentValue,
    BonusValue
}

public interface IAttributeSet
{
    float GetBaseValue(EAttributeType attrId);

    float GetCurrentValue(EAttributeType attrId);
}

public interface IBoundsLocker
{
    bool LockUpperBounds { get; set; }

    bool LockLowerBounds { get; set; }

    float UpperPercent { get; set; }

    float UpperOffset { get; set; }

    float LowerPercent { get; set; }

    float LowerOffset { get; set; }
}

// Contract shape retained; arrays are nullable in the double so malformed-input
// cases can reach the exact recovered callee unchanged.
public interface IBuffModifierData
{
    EAttributeType AttributeId { get; }

    float[]? Value1 { get; }

    float[]? Value2 { get; }

    int[] CalculationPolicy { get; }
}

public sealed class Entity
{
    public string Name { get; set; } = "TEST_DOUBLE_ENTITY";
}

public class Singleton<T> where T : Singleton<T>, new()
{
    public static T Instance { get; } = new T();
}

// Minimal log sink for the exact recovered missing-instigator/error branch.
public sealed class CombatLog : Singleton<CombatLog>
{
    public enum EDebugModule
    {
        Buff = 3,
        Attribute = 20
    }

    public sealed class Entry
    {
        public EDebugModule Module { get; set; }

        public Entity? Entity { get; set; }

        public string Message { get; set; } = "";

        public List<(string Key, object Value)> Pairs { get; } = new List<(string, object)>();
    }

    private readonly List<Entry> _entries = new List<Entry>();

    public int Count => _entries.Count;

    public Entry? Last => _entries.Count == 0 ? null : _entries[_entries.Count - 1];

    public void Reset()
    {
        _entries.Clear();
    }

    public void Error(EDebugModule module, Entity? entity, string message, ReadOnlySpan<(string, object)> pairs)
    {
        Entry entry = new Entry
        {
            Module = module,
            Entity = entity,
            Message = message
        };
        for (int i = 0; i < pairs.Length; i++)
        {
            entry.Pairs.Add(pairs[i]);
        }
        _entries.Add(entry);
    }
}

public readonly record struct DispatchRecord(EAttributeType AttrId, float NewValue, float OldValue);

public sealed class TestBoundsLocker : IBoundsLocker
{
    public bool LockUpperBounds { get; set; }

    public bool LockLowerBounds { get; set; }

    public float UpperPercent { get; set; }

    public float UpperOffset { get; set; }

    public float LowerPercent { get; set; }

    public float LowerOffset { get; set; }
}

public sealed class TestBuffModifier : IBuffModifierData
{
    public TestBuffModifier(EAttributeType attributeId, float[]? value1, float[]? value2, int[] calculationPolicy)
    {
        AttributeId = attributeId;
        Value1 = value1;
        Value2 = value2;
        CalculationPolicy = calculationPolicy;
    }

    public EAttributeType AttributeId { get; }

    public float[]? Value1 { get; }

    public float[]? Value2 { get; }

    public int[] CalculationPolicy { get; }
}

public static class CharacterAttributeTypes
{
    public const int ATTRIBUTE_ID_MAX = 143;

    public static EAttributeType[] attrsNotClampZero = Array.Empty<EAttributeType>();

    public static HashSet<EAttributeType> stateAttributeIds = new HashSet<EAttributeType>();

    public static Dictionary<EAttributeType, EAttributeType> attributeIdsWithMax =
        new Dictionary<EAttributeType, EAttributeType>();

    public static void ResetForTest()
    {
        attrsNotClampZero = Array.Empty<EAttributeType>();
        stateAttributeIds.Clear();
        attributeIdsWithMax.Clear();
    }
}

// Partial host for exact recovered methods. All members below are authored test
// scaffolding: permission, update, collector, dispatch-implement and logging are
// deliberate TEST_DOUBLE/UNKNOWN_RUNTIME_OVERRIDE surfaces.
public partial class BaseAttributeComponent : IAttributeSet
{
    protected readonly float[] BaseValues = new float[CharacterAttributeTypes.ATTRIBUTE_ID_MAX];
    protected readonly float[] CurrentValues = new float[CharacterAttributeTypes.ATTRIBUTE_ID_MAX];
    protected readonly List<IBoundsLocker> BoundsLockerScratch = new List<IBoundsLocker>();

    public Entity Entity { get; set; } = new Entity();

    public Func<EAttributeType, bool>? WritablePredicate { get; set; }

    public Action<BaseAttributeComponent, EAttributeType>? UpdateObserver { get; set; }

    public Action<EAttributeType, float, float>? DispatchObserver { get; set; }

    public List<IBoundsLocker> SuppliedLockers { get; } = new List<IBoundsLocker>();

    public List<DispatchRecord> DispatchCalls { get; } = new List<DispatchRecord>();

    public List<string> Trace { get; } = new List<string>();

    public int WritableCalls { get; private set; }

    public int UpdateCalls { get; private set; }

    public int CollectorCalls { get; private set; }

    public int DispatchImplementCalls { get; private set; }

    public bool IsWritableAttribute(EAttributeType attrId)
    {
        WritableCalls++;
        return WritablePredicate?.Invoke(attrId) ?? false;
    }

    public virtual void UpdateCurrentValue(EAttributeType attrId)
    {
        UpdateCalls++;
        Trace.Add("update");
        UpdateObserver?.Invoke(this, attrId);
    }

    public virtual void CollectBoundsLockers(EAttributeType attrId, List<IBoundsLocker> output)
    {
        CollectorCalls++;
        output.AddRange(SuppliedLockers);
    }

    protected virtual void DispatchCurrentValueEventImplement(EAttributeType attrId, float newValue, float oldValue)
    {
        DispatchImplementCalls++;
        Trace.Add("dispatch");
        DispatchCalls.Add(new DispatchRecord(attrId, newValue, oldValue));
        DispatchObserver?.Invoke(attrId, newValue, oldValue);
    }

    public float GetRawBase(EAttributeType attrId)
    {
        return BaseValues[(int)attrId];
    }

    public void SetRawBase(EAttributeType attrId, float value)
    {
        BaseValues[(int)attrId] = value;
    }

    public float GetRawCurrent(EAttributeType attrId)
    {
        return CurrentValues[(int)attrId];
    }

    public void SetRawCurrent(EAttributeType attrId, float value)
    {
        CurrentValues[(int)attrId] = value;
    }
}
