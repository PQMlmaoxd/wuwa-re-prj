// PERF11: NONCANONICAL / RECONSTRUCTED / PARTIAL boundary fragments.
// Static at-rest managed reconstruction only; these fragments are not
// standalone compilation units. Existing PERF03 TimerSystemInstance and the
// canonical EventSystem source are referenced rather than duplicated.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.

// MethodDef 0x0600326C; TimerSystemInstance.Has; RVA 0x229E6; code size 23.
// DIRECT_IL_VERIFIED; PERF11_DIRECT_CIL.txt: TimerSystemInstance::Has.
public partial class TimerSystemInstance
{
    public bool Has(TimerHandle? handle)
    {
        if (handle != null)
        {
            return Timers.ContainsKey(handle.Id);
        }
        return false;
    }

    // MethodDef 0x06003273; TimerSystemInstance.Remove; RVA 0x22AE4; code size 26.
    // DIRECT_IL_VERIFIED; PERF11_DIRECT_CIL.txt: TimerSystemInstance::Remove.
    public bool Remove(TimerHandle? handle)
    {
        if (handle != null)
        {
            Timer timer = Get(handle);
            if (timer != null)
            {
                return DoRemove(timer);
            }
            return false;
        }
        return true;
    }
}

// EventSystem generic target wrappers are retained as two honest source-only
// excerpts. The authority identity is the fully qualified
// CSharpScript.Game.Common.Event.EventSystem; this is not a global wrong-type
// declaration. Type metadata and canonical source identify their declarations
// and source shape, but the official disassemble_method route could not select
// overloaded generic MethodDefs 0x06045B72 and 0x06045B73. No stub body is
// substituted for either missing direct-CIL response.
namespace CSharpScript.Game.Common.Event
{
    public partial class EventSystem
    {
        // MethodDef 0x06045B72; source-only excerpt from the canonical
        // EventSystem.cs wrapper. The catch filter is source context only;
        // direct CIL remains UNKNOWN and no catch-all claim is made here.
        // public bool EmitWithTarget<T1>(object target, EEventName name, T1 p1)
        // {
        //     try { return GetTargetEvent(target)?.Emit(name, p1) ?? false; }
        //     catch (Exception error) when (1 != 0)
        //     {
        //         Singleton<Log>.Instance.ErrorWithStack(
        //             (ELogModule)10, (ELogAuthor)37,
        //             "[EventSystem::EmitWithTarget<T1>]throw exception", error,
        //             default(ReadOnlySpan<(string, object)>));
        //         return false;
        //     }
        // }
        // DIRECT_CIL_STATUS: UNKNOWN_DIRECT_CIL; selector blocker recorded in
        // PERF11_EVIDENCE.json and PERF11_DIRECT_CIL.txt.

        // MethodDef 0x06045B73; source-only excerpt from the canonical
        // EventSystem.cs wrapper. The catch filter is source context only;
        // direct CIL remains UNKNOWN and no catch-all claim is made here.
        // public bool EmitWithTarget<T1, T2>(object target, EEventName name, T1 p1, T2 p2)
        // {
        //     try { return GetTargetEvent(target)?.Emit(name, p1, p2) ?? false; }
        //     catch (Exception error) when (1 != 0)
        //     {
        //         Singleton<Log>.Instance.ErrorWithStack(
        //             (ELogModule)10, (ELogAuthor)37,
        //             "[EventSystem::EmitWithTarget<T1,T2>]throw exception", error,
        //             default(ReadOnlySpan<(string, object)>));
        //         return false;
        //     }
        // }
        // DIRECT_CIL_STATUS: UNKNOWN_DIRECT_CIL; selector blocker recorded in
        // PERF11_EVIDENCE.json and PERF11_DIRECT_CIL.txt.
    }
}
