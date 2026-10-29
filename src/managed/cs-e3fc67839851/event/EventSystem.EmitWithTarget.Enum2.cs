/*
 * PERF_CIL16_WRAPPER_RECOVERY — NONCANONICAL / RECONSTRUCTED /
 * STATIC_EVIDENCE_ONLY / NON_STANDALONE.
 *
 * This is a meaningful C#-like projection of one exact-token direct-CIL body.
 * It is not recovered developer source, a standalone compilation unit, or a
 * declaration of the original EventSystem layout. External types, helper
 * bodies, and the Event<TName,TType>.Emit implementation remain unexpanded.
 *
 * Managed authority: private/r5b1/materialized/CSharpScript.dll
 * SHA-256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * TypeDef: 0x0200706E (CSharpScript.Game.Common.Event.EventSystem)
 * MethodDef: 0x06045B73; instance public bool EmitWithTarget<T1,T2>(...)
 * Method RVA: 0x123C750; header/file offset: 0x123A950; EH/file offset: 0x123A9B0
 * Code: 81 bytes; RVA [0x123C75C,0x123C7AD), file [0x123A95C,0x123A9AD)
 * Body extent: RVA [0x123C750,0x123C7C0), file [0x123A950,0x123A9C0)
 * (12-byte header + 81-byte code + 3 alignment bytes + 16-byte EH section;
 * alignment padding values were not separately captured.)
 * Direct-CIL code SHA-256:
 * 921295c46903f2463aa46be875ec54a5d4baeeadb310850e5f5db9bd69d8b46d
 *
 * Generic parameters T1/T2 and the aliases below are reconstructed names from
 * the exact signature/context. Generic constraints and original field layout
 * were not recovered. `receiver` denotes the value returned by the observed
 * GetTargetEvent call; `module`, `author`, `message`, `loggedError`, and
 * `details` denote ordered local projections, not original-name proof.
 */

// Method fragment only: CSharpScript.Game.Common.Event.EventSystem context.
public bool EmitWithTarget<T1, T2>(object target, EEventName name, T1 p1, T2 p2)
{
    bool result;
    try
    {
        // IL_0000..IL_0002: call 0x06045B7F, an instance GetTargetEvent
        // call whose returned Event<EEventName,Delegate> is kept as receiver.
        // The original target-null condition is not locally validated;
        // GetTargetEvent effects remain unknown in this fragment.
        var receiver = GetTargetEvent(target);

        // IL_0007..IL_0012: explicit reference-null test. The non-null path
        // forwards name, p1, and p2 unchanged to the actual call 0x2B0043AF.
        if (receiver is null)
        {
            result = false;
        }
        else
        {
            // The selected instruction is `call`, not `callvirt`; this source
            // syntax is only the readable projection of that instance call.
            result = receiver.Emit<T1, T2>(name, p1, p2);
        }
    }
    catch (Exception error) when (1U > 0U)
    {
        // IL_002d..IL_004d: the filter has accepted System.Exception. The
        // handler does not read Exception.Message and has no nested handler.
        var log = Singleton<Log>.Instance;
        var module = (ELogModule)10;
        var author = (ELogAuthor)37;
        var message = "[EventSystem::EmitWithTarget<T1,T2>]throw exception";
        var loggedError = error;
        ReadOnlySpan<(string, object)> details = default;
        log.ErrorWithStack(module, author, message, loggedError, details);
        result = false;
    }

    // IL_004f..IL_0050: both the receiver-null false and Emit result reach this
    // shared return. A logging fault can escape before this statement.
    return result;
}
