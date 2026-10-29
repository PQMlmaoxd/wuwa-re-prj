/*
 * PERF-CIL18 — NONCANONICAL RECONSTRUCTED STATIC_EVIDENCE_ONLY
 * NON_STANDALONE.
 *
 * Source-like projection of the complete selected body only.  This fragment
 * uses the existing partial Event<TName,TType> packaging convention; it adds
 * no namespace, generic constraints, helper, or external declaration.
 *
 * Managed authority identity was supplied by the primary investigator:
 * private/r5b1/materialized/CSharpScript.dll
 * SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * TypeDef 0x020000A3; MethodDef 0x0600042E; RVA 0x18710; body 21 bytes.
 * Raw evidence: evidence/performance/PERF_CIL18_DIRECT_CIL.txt:16-28.
 *
 * The selected CIL constructs EventArgs<T1,T2> through a value-type
 * GENERICINST, explicitly BOXes that value, reloads p1, and calls
 * EmitInternal<T1>.  The `(object)` cast below is an expository projection of
 * that BOX before the IEventArgs signature adaptation; it is not a claim that
 * a CIL `castclass` instruction was present.
 */

public partial class Event<TName, TType>
{
    // MethodDef 0x0600042E; public hidebysig instance bool; generic arity 2.
    // Source parameter names are retained for readable forwarding only.
    // UNKNOWN: constructor side effects and EmitInternal helper exceptions/effects are not modeled.
    public bool Emit<T1, T2>(TName name, T1 p1, T2 p2)
    {
        // No IL locals: newobj(value) -> box(EventArgs<T1,T2>) -> p1 -> call.
        return EmitInternal<T1>(
            name,
            (IEventArgs)(object)new EventArgs<T1, T2>(p1, p2),
            p1);
    }
}
