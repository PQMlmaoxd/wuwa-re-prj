/*
 * PERF_CIL15_CONCRETE_INVOKE — NONCANONICAL / RECONSTRUCTED /
 * STATIC_EVIDENCE_ONLY / NON_STANDALONE.
 *
 * This is a meaningful C#-like method fragment in the global-namespace
 * EventArgs<T1,T2> context. It is not recovered developer source, a standalone
 * compilation unit, or a declaration of the original class layout.
 *
 * Managed authority: private/r5b1/materialized/CSharpScript.dll
 * SHA-256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * TypeDef: 0x020000A9 (global EventArgs`2)
 * MethodDef: 0x06000458; instance bool Invoke(Enum name, Delegate handle)
 * RVA: 0x195F8; header/file offset: 0x177F8; code RVA/file: 0x19604/0x17804
 * Header: 12 bytes; code: 103 bytes; extent: [0x19604,0x1966B)
 * Direct-CIL code SHA-256:
 * f522fc595814f480cc7c263fd56980eb5478f6e330fbda0a6d282a99fafef91b
 *
 * Generic context is !0=T1 and !1=T2; the method is not generic. The aliases
 * below are direct field-read notation only:
 *   this.payload1 -> MemberRef 0x0A000659, '<p1>P', signature !0
 *   this.payload2 -> MemberRef 0x0A00065A, '<p2>P', signature !1
 * No field, property, getter, helper body, partial declaration, or constraint
 * is invented here.
 */

// Method fragment only: global EventArgs<T1,T2> context.
public bool Invoke(Enum name, Delegate handle)
{
    // IL_0000..IL_001c: first-compatible Action<T1,T2> branch wins. The
    // isinst test is compatibility testing, not exact delegate-type equality.
    Action<T1, T2> typed = handle as Action<T1, T2>;
    if (typed is not null)
    {
        // Direct reads of MemberRefs 0x0A000659 then 0x0A00065A occur only
        // immediately before the selected Action<T1,T2>::Invoke call.
        typed.Invoke(this.payload1, this.payload2);

        // IL_001c branches to the shared IL_0065 true return. This early
        // return is the source-like projection of that normal-flow join.
        return true;
    }

    // IL_001e..IL_0050: the second compatible form is GenericEventHandler.
    GenericEventHandler generic = handle as GenericEventHandler;
    if (generic is not null)
    {
        // newarr object[2] is evaluated before either payload field read.
        // Slot 0 is populated before slot 1; each cast preserves the observed
        // box !0 / box !1 operation for value-type instantiations.
        object[] arguments = new object[2];
        arguments[0] = (object)this.payload1;
        arguments[1] = (object)this.payload2;
        generic.Invoke(arguments);

        // IL_0050 also branches to the shared IL_0065 true return.
        return true;
    }

    // IL_0052..IL_0064: name is unused by both delegate branches and is passed
    // only to the unresolved fallback call, in this exact argument order.
    CALL_METHOD_06000450(typeof(Action<T1, T2>), handle, name);
    return false;
}
