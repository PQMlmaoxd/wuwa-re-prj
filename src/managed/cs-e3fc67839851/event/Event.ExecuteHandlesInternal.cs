/*
 * PERF_CIL13_HANDLER_EXECUTION — NONCANONICAL / RECONSTRUCTED /
 * STATIC_EVIDENCE_ONLY / NON_STANDALONE.
 *
 * This is a source-like projection of the selected direct-CIL body. It is not
 * recovered developer source and is intentionally not a standalone
 * compilation unit. External types and helper bodies remain unresolved.
 *
 * Managed authority: private/r5b1/materialized/CSharpScript.dll
 * SHA-256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * TypeDef: 0x020000A3 (Event`2)
 * MethodDef: 0x06000435; RVA: 0x18B20; code size: 337 bytes;
 * direct-CIL body SHA-256: 83e04c6bb9e2a05b700d8de5aefc0990e55c8b783984d248afc82446f822d158
 *
 * The Enum/Delegate constraints are retained from the accepted source
 * context, not newly parsed from this method body. `partial` is packaging for
 * reconstruction and does not prove the original declaration used it.
 */

using System;
using System.Collections.Generic;

public partial class Event<TName, TType>
    where TName : Enum
    where TType : Delegate
{
    // MethodDef 0x06000435; private hidebysig; generic arity 1; void.
    // RECONSTRUCTED projection backed by the DIRECT_CIL_VERIFIED full selected body.
    private void ExecuteHandlesInternal<T1>(
        TName name,
        Dictionary<Delegate, EHandleType> handles,
        ConditionListener<T1> conditionListener,
        IEventArgs args,
        T1 conditionParam)
    {
        // IL_0000..IL_002e: local 0 starts null. Stat.Enable gates both
        // NameStateMap operations; a failed TryGetValue stores that same
        // out-local value. There is no observed Stat construction or timing.
        Stat stat = null;
        if (Stat.Enable)
        {
            if (!EventConstant.NameStateMap.TryGetValue((Enum)(object)name, out stat))
            {
                EventConstant.NameStateMap[(Enum)(object)name] = stat;
            }
        }

        // IL_002f..IL_0034 is outside the protected region. The explicit
        // try/finally below projects the outer EH region beginning at IL_0035.
        var enumerator = handles.GetEnumerator();
        try
        {
            while (enumerator.MoveNext())
            {
                KeyValuePair<Delegate, EHandleType> entry = enumerator.Current;
                Delegate handler = entry.Key;

                // IL_004b..IL_0063: the pending query is selected solely by
                // conditionListener reference nullness (no overloaded equality). A true result skips every
                // later operation for this entry and continues enumeration.
                bool pending = conditionListener is null
                    ? CheckInPendingRemoveHandles(name, handler)
                    : conditionListener.IsInPendingRemove(conditionParam, handler);
                if (pending)
                {
                    continue;
                }

                // IL_0068..IL_008c: literal EHandleType value 1 selects the
                // removal call. Its bool result is intentionally ignored.
                if (entry.Value == (EHandleType)1)
                {
                    if (conditionListener is not null)
                    {
                        RemoveWithConditionInternal(name, handler, conditionParam);
                    }
                    else
                    {
                        RemoveInternal(name, handler);
                    }
                }

                // IL_008d..IL_00ab: local 5 is null and the lookup result is
                // ignored. This preparation is outside the inner try/filter.
                Stat handleStat = null;
                if (Stat.Enable)
                {
                    EventConstant.HandleStatMap.TryGetValue(handler.Method, out handleStat);
                }

                // IL_00ac..IL_00bc: VERIFIED_CALL_IN_SELECTED_CIL. This is
                // IEventArgs.Invoke, not direct delegate invocation.
                try
                {
                    args.Invoke((Enum)(object)name, handler);
                }
                // The filter is the direct isinst Exception plus constant
                // unsigned comparison (1U > 0U). It does not mean that every
                // exception is caught or that the handler always completes.
                catch (Exception error) when (1U > 0U)
                {
                    // IL_00d3 gets Singleton<Log>.Instance before payload
                    // construction. InlineArray2 and PrivateImplementationDetails
                    // are RECONSTRUCTED aliases for the exact compiler-generated
                    // <>y__InlineArray2`1 / helper names; their declarations and
                    // bodies are external and intentionally not stubbed here.
                    Log logger = Singleton<Log>.Instance;
                    InlineArray2<(string, object)> logPayload = default;

                    // This is a value local, not a heap array/list. The first
                    // element reference is acquired before boxing name.
                    PrivateImplementationDetails.InlineArrayElementRef(ref logPayload, 0) =
                        new ValueTuple<string, object>("name", (object)name);

                    // The second element reference is acquired before reading
                    // error.Message, matching the selected CIL evaluation order.
                    PrivateImplementationDetails.InlineArrayElementRef(ref logPayload, 1) =
                        new ValueTuple<string, object>("error", error.Message);

                    // The external helper produces the two-element
                    // ReadOnlySpan; ErrorWithStack is the observed logger call.
                    logger.ErrorWithStack(
                        (ELogModule)10,
                        (ELogAuthor)1,
                        "事件处理方法执行异常",
                        error,
                        PrivateImplementationDetails.InlineArrayAsReadOnlySpan(ref logPayload, 2));
                }
            }
        }
        finally
        {
            // C# projection of the constrained. Enumerator::Dispose call at
            // IL_0144..IL_014f; no cast to IDisposable is introduced.
            enumerator.Dispose();
        }
    }
}
