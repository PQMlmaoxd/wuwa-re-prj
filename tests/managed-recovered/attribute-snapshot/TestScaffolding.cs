// Authored test scaffolding for RE-SNAPSHOT-DIRECT-TEST01.
// This file contains NO recovered method body. It supplies the minimal host
// field shape, enum adapters, SnapshotPayload adapter, and one public wrapper
// that calls the exact private recovered Calculation method in the private
// scratch fixture.
using Aki.Protocol;

namespace Aki.Protocol
{
    // Minimal authored adapter subset, not a whole-enum or build-compatibility claim.
    // Values are the retained authority names/values needed by this task:
    // Lv=1, CritDamage=9, Def=10, DamageChange=15, Max=143.
    public enum EAttributeType
    {
        Lv = 1,
        CritDamage = 9,
        Def = 10,
        DamageChange = 15,
        Max = 143
    }
}

// Exact saved-source names/values: EAttrOwnerType.cs lines 1-5 (Attacker, Target).
// The selected body proves only that 0 selects attacker and any nonzero value
// selects target; this adapter does not claim more enum members.
public enum EAttrOwnerType
{
    Attacker = 0,
    Target = 1
}

// Minimal adapter for the only canonical SnapshotPayload members consumed by the
// selected method: lines 9 and 11 of the pinned SnapshotPayload.cs context source.
// The other canonical properties are intentionally omitted.
public class SnapshotPayload
{
    public AttributeSnapshot? AttackerSnapshot { get; set; }

    public AttributeSnapshot? TargetSnapshot { get; set; }
}

// Authored TEST_DOUBLE host. Two 143-entry arrays are the only state; no setter,
// recompute, copy, or selector algorithm is defined here. The exact recovered
// TakeSnapshot body in the private fixture reads these arrays.
public partial class BaseAttributeComponent
{
    public readonly float[] BaseValues = new float[143];

    public readonly float[] CurrentValues = new float[143];
}

// Authored TEST_DOUBLE allocator/scaffolding for the original whole-class context.
// The field shape matches the pinned source: two public readonly float[143]
// arrays at lines 5 and 7. The exact recovered GetCurrentValue body is in the
// private fixture; no base getter is authored here.
public partial class AttributeSnapshot
{
    public readonly float[] BaseValues = new float[143];

    public readonly float[] CurrentValues = new float[143];
}

// Authored adapter only. The private declaration and exact body of
// GetAttrFromSnapshots remain unchanged in the fixture; this wrapper calls it.
public partial class Calculation
{
    public static double GetAttrFromSnapshotsForTest(SnapshotPayload snap, EAttrOwnerType attrOwnerType, EAttributeType attrId)
    {
        return GetAttrFromSnapshots(snap, attrOwnerType, attrId);
    }
}
