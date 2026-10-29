/*
 * PERF-CIL17 — NONCANONICAL RECONSTRUCTED STATIC_EVIDENCE_ONLY
 * NON_STANDALONE.
 *
 * Meaningful C#-like method fragment only.  This file deliberately declares no
 * containing class, fields, helper types, dependency stubs, or generic
 * constraints.  TName, Event<TName,TType>, and every external member below are
 * context aliases for the selected method; their declarations and bodies are
 * not reconstructed here.
 *
 * Managed authority: private/r5b1/materialized/CSharpScript.dll
 * SHA-256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * TypeDef: 0x020000A3 (Event`2; namespace empty)
 * MethodDef: 0x06000434 (EmitInternal<T1>)
 * Signature: instance bool(!0 name, IEventArgs args, !!0 conditionParam)
 * RVA: 0x187C0
 * Header: file 0x169C0; 12 bytes; 1b3007000803000012010011; MaxStack=7;
 *         InitLocals=true; MoreSects=true; LocalSig=0x11000112
 * Code: file [0x169CC,0x16CD4); RVA [0x187CC,0x18AD4); 776 bytes
 * Code SHA-256: 9faf65ff85a41eece47cfe0ed99213faacf23fd1ae6d4efae9a4944c94222827
 * Full body extent: file [0x169C0,0x16D20); RVA [0x187C0,0x18B20);
 *                    864 bytes = 12 header + 776 code + 76 EH
 * EH section: file 0x16CD4; six small FINALLY clauses; raw hex is retained in
 * the report and in the immutable text evidence below.
 * Raw evidence: evidence/performance/PERF_CIL17_DIRECT_CIL.txt
 *
 * The six explicit enumerator try/finally regions below are source-like
 * projections of constrained Dispose calls.  `e.Dispose()` is not a claim
 * about a public concrete Dispose method, interface boxing, or successful
 * completion; see the accompanying report's EH overlay.
 */

// Method fragment only: Event<TName,TType> context; no declaration is supplied.
private bool EmitInternal<T1>(TName name, IEventArgs args, T1 conditionParam)
{
    // IL_0000..IL_0007: the guard has no observed mask setter on this path.
    if (IsEmitting(name))
    {
        var logger = Singleton<Log>.Instance;

        // External compiler-generated aliases.  This is a value local, not a
        // heap array or tuple[]; helper declarations are intentionally absent.
        InlineArray2<ValueTuple<string, object>> logPayload = default;

        // The element-reference helper is evaluated before boxing name and
        // constructing the first tuple, matching the selected CIL order.
        InlineArrayElementRef(ref logPayload, 0) =
            new ValueTuple<string, object>("name", (object)name);

        // The second element reference is acquired before reading the field.
        InlineArrayElementRef(ref logPayload, 1) =
            new ValueTuple<string, object>("emittingEventInSet", EmittingMaskSet);

        logger.Error(
            (ELogModule)10,
            (ELogAuthor)1,
            "事件重复派发，请检查事件链是否产生循环调用",
            InlineArrayAsReadOnlySpan(ref logPayload, 2));
        return false;
    }

    // IL_0067..IL_00E5: conditional dispatch precedes ordinary dispatch.
    SetEmitting(name, true);
    var conditionListener = GetConditionListener<T1>(name);
    if (conditionListener is not null && (object)conditionParam is not null)
    {
        var conditionalHandles = conditionListener.GetHandlesByParam(conditionParam);
        if (conditionalHandles is not null)
        {
            ExecuteHandlesInternal<T1>(
                name, conditionalHandles, conditionListener, args, conditionParam);
            if (conditionalHandles.Count == 0)
            {
                // The boolean result is ignored, as in the selected CIL.
                conditionListener.DeleteHandlesByParam(conditionParam);
            }
        }
    }

    // IL_00AC..IL_00E5: ordinary dispatch has no pre-count guard and uses
    // default(T1), not conditionParam.
    if (Handles.TryGetValue(name, out var ordinaryHandles))
    {
        ExecuteHandlesInternal<T1>(
            name, ordinaryHandles, null, args, default(T1));
        if (ordinaryHandles.Count == 0)
        {
            // The boolean result is ignored, as in the selected CIL.
            Handles.Remove(name);
        }
    }

    // IL_00E5..IL_00ED: mask false follows both dispatch/prune phases and
    // precedes all pending work; the setter call is at IL_00E8.  There is no
    // outer finally around this method.
    SetEmitting(name, false);

    // IL_00ED..IL_0146: ordinary pending removals.  The getter is evaluated
    // once for TryGetValue and again for Remove, exactly as selected.
    if (PendingRemoveHandles.TryGetValue(name, out var pendingRemove))
    {
        var removeEnumerator = pendingRemove.GetEnumerator();
        try
        {
            while (removeEnumerator.MoveNext())
            {
                var item = removeEnumerator.Current;
                DoRemove(name, item);
            }
        }
        finally
        {
            // Pseudo-source for constrained.<raw enumerator type>.
            // IDisposable::Dispose; no boxing or completion guarantee claimed.
            removeEnumerator.Dispose();
        }

        // The captured found bucket is cleared only after its finally.
        pendingRemove.Clear();
        PendingRemoveHandles.Remove(name);
    }

    // IL_0146..IL_01AB: ordinary pending additions.
    if (PendingAddHandles.TryGetValue(name, out var pendingAdd))
    {
        var addEnumerator = pendingAdd.GetEnumerator();
        try
        {
            while (addEnumerator.MoveNext())
            {
                var item = addEnumerator.Current;
                DoAdd(name, item.Key, item.Value);
            }
        }
        finally
        {
            // Pseudo-source for constrained.<raw enumerator type>.
            // IDisposable::Dispose; no boxing or completion guarantee claimed.
            addEnumerator.Dispose();
        }

        // The captured found bucket is cleared only after its finally.
        pendingAdd.Clear();
        PendingAddHandles.Remove(name);
    }

    // IL_01AB..IL_0308: a null captured listener returns normally after the
    // ordinary pending phases and skips all conditional pending work.
    if (conditionListener is null)
    {
        return true;
    }

    // Conditional pending removals: the inner enumerator is acquired inside
    // the outer try but before the inner try, preserving nested EH coverage.
    if (conditionListener.PendingRemoveHandles is not null &&
        conditionListener.PendingRemoveHandles.Count > 0)
    {
        var outerRemoveEnumerator = conditionListener.PendingRemoveHandles.GetEnumerator();
        try
        {
            while (outerRemoveEnumerator.MoveNext())
            {
                var outerRemoveEntry = outerRemoveEnumerator.Current;
                var innerRemoveEnumerator = outerRemoveEntry.Value.GetEnumerator();
                try
                {
                    while (innerRemoveEnumerator.MoveNext())
                    {
                        var innerRemoveEntry = innerRemoveEnumerator.Current;
                        DoRemoveWithCondition(
                            name, outerRemoveEntry.Key, innerRemoveEntry);
                    }
                }
                finally
                {
                    // Pseudo-source for constrained.<raw enumerator type>.
                    // IDisposable::Dispose; no completion guarantee claimed.
                    innerRemoveEnumerator.Dispose();
                }
            }
        }
        finally
        {
            // Pseudo-source for constrained.<raw enumerator type>.
            // IDisposable::Dispose; no completion guarantee claimed.
            outerRemoveEnumerator.Dispose();
        }

        // This clear is inside the positive-count branch, after its outer
        // finally. Null and non-positive-count branches skip it.
        conditionListener.PendingRemoveHandles.Clear();
    }

    // Conditional pending additions: same outer/inner acquisition and EH
    // nesting shape, with key/value evaluation order retained.
    if (conditionListener.PendingAddHandles is not null &&
        conditionListener.PendingAddHandles.Count > 0)
    {
        var outerAddEnumerator = conditionListener.PendingAddHandles.GetEnumerator();
        try
        {
            while (outerAddEnumerator.MoveNext())
            {
                var outerAddEntry = outerAddEnumerator.Current;
                var innerAddEnumerator = outerAddEntry.Value.GetEnumerator();
                try
                {
                    while (innerAddEnumerator.MoveNext())
                    {
                        var innerAddEntry = innerAddEnumerator.Current;
                        DoAddWithCondition(
                            name,
                            conditionListener,
                            innerAddEntry.Key,
                            innerAddEntry.Value,
                            outerAddEntry.Key);
                    }
                }
                finally
                {
                    // Pseudo-source for constrained.<raw enumerator type>.
                    // IDisposable::Dispose; no completion guarantee claimed.
                    innerAddEnumerator.Dispose();
                }
            }
        }
        finally
        {
            // Pseudo-source for constrained.<raw enumerator type>.
            // IDisposable::Dispose; no completion guarantee claimed.
            outerAddEnumerator.Dispose();
        }

        // This clear is inside the positive-count branch, after its outer
        // finally. Null and non-positive-count branches skip it.
        conditionListener.PendingAddHandles.Clear();
    }

    if (conditionListener.IsHandlesEmpty())
    {
        ConditionHandles.Remove(name);
    }

    return true;
}
