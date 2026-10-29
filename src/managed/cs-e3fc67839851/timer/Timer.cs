// PERF-03: NONCANONICAL / RECONSTRUCTED source-like fragment.
// Source assembly: CSharpScript.dll SHA-256
// e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; Timer TypeDef 0x02000bec.
// Direct CIL: evidence/performance/PERF03_DIRECT_CIL.txt.
// Only Timer.Do (0x06003267) and Timer.Clear (0x06003269) are projected here.
// The constructor and other members are intentionally omitted; this fragment
// retains external game types rather than inventing dependency stubs.

using System;

public class Timer
{
    // OBSERVED field accesses in the selected CIL; declarations are context,
    // not a claim that this partial fragment is a standalone compilation unit.
    public double Now;
    public double Next;
    private int Count;
    public EState State;
    public readonly int Id;
    private TTimerAction? Action;
    public float Interval;
    private readonly int Loop;
    public float Dilation;
    public TimerHandle? Handle;
    private Stat? Stat;
    public readonly string? Reason;

    // MethodDef 0x06003267; RVA 0x226fc; code size 265.
    // DIRECT_IL_VERIFIED. Deadline/count advance before callback; the handle is
    // cleared before invocation. If ErrorWithStack returns, the filtered
    // handler consumes the callback exception; an exception thrown by the
    // logger escapes because the handler has no enclosing exception region.
    public bool Do()
    {
        double next = Next;
        float delta = (float)(next - Now);
        _ = Stat;
        Now = next;
        Next = next + (double)(Interval / Dilation);
        Count++;

        try
        {
            Handle = null;
            Action!.Invoke(delta);
        }
        catch (Exception ex) when (1 != 0)
        {
            // The explicit filter is the observed isinst Exception / true
            // filter shape. This array is a readable projection of the
            // compiler-generated inline-array ReadOnlySpan arguments, not a
            // claim about an actual game allocation.
            Singleton<Log>.Instance.ErrorWithStack(
                (ELogModule)22,
                (ELogAuthor)1,
                "定时器执行异常",
                ex,
                new (string, object)[]
                {
                    ("id", Id),
                    ("reason", Reason),
                    ("error", ex.Message),
                });
        }

        if (Loop != 0)
        {
            return Count < Loop;
        }

        return true;
    }

    // MethodDef 0x06003269; RVA 0x22898; code size 22.
    // DIRECT_IL_VERIFIED. Exactly Action, Handle, and Stat are nulled.
    public void Clear()
    {
        Action = null;
        Handle = null;
        Stat = null;
    }
}
