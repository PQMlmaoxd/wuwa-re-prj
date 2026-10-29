/*
 * PERF04 PARTIAL SOURCE-LIKE FRAGMENT — NONCANONICAL / RECONSTRUCTED.
 * TypeDef: 0x0200349c.
 * Full type: global::GameModePromise.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/06feb1084d70c2d77f32096205570263da84de210fce65e857a10b295a685777/source/_global/GameModePromise.cs
 * Selected bodies: source/call-site projection of Promise (0x0601C6AF) and
 * direct-CIL reconstruction of SetResult (0x0601C6B0).
 * Omitted-members node: no additional lifecycle, cancellation, or retry
 * members are invented; external AkiUniTask behavior remains unresolved.
 * `partial` marks reconstruction packaging only; it does not prove the
 * original declaration used the partial modifier.
 *
 * Promise is a semantic source-only projection distinct from the failed
 * direct-CIL capture. Compiler-generated lowering is not re-pasted here.
 * Source assembly: CSharpScript.dll SHA-256
 * e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.
 */

using Cysharp.Threading.Tasks;

public partial class GameModePromise
{
    private readonly AkiUniTask<bool> promiseSource = new AkiUniTask<bool>();
    private bool HasResult;
    private bool Result;

    // MethodDef 0x0601C6AF; the direct-CIL request and retries failed.
    // UNKNOWN_DIRECT_CIL: evidence/performance/PERF04_DIRECT_CIL.txt:1869-1873,
    // 1983-1987, and 1989-1993. The body is bounded by canonical source and
    // call-site evidence only; no replacement parser was used.
    public UniTask<bool> Promise
    {
        get
        {
            if (HasResult)
            {
                return UniTask.FromResult(Result);
            }

            return promiseSource.GetTask();
        }
    }

    // MethodDef 0x0601C6B0; RVA 0x88336c; code size 85.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1875-1925.
    public void SetResult(bool value)
    {
        HasResult = true;
        Result = value;
        if (GlobalData.Networking() && !Singleton<Net>.Instance.IsServerConnected())
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)80,
                (ELogAuthor)3,
                "账号已经登出",
                new (string, object)[] { });
            promiseSource.TrySetCanceled();
            return;
        }

        promiseSource.TrySetResult(value);
    }
}
