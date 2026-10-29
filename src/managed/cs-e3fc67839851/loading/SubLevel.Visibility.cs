/*
 * PERF04B visibility-completion companion — NONCANONICAL / RECONSTRUCTED / PARTIAL.
 * Full type: global::SubLevel; TypeDef 0x0200347d.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/4e917791d98d890279255c9e7fab873b11dfbc5ff3806dcc7db5c644bc77d5e9/source/_global/SubLevel.cs
 *
 * This companion adds only the observed SetLevelVisiblePromise field and the
 * bounded visibility-completion members selected for PERF04B. `partial` is
 * reconstruction packaging only; it does not prove original source syntax.
 * Generated MoveNext identities are represented once as readable async source;
 * no generated state-machine types are invented here. Tuple-array log storage
 * is a readable projection of the observed InlineArray/ReadOnlySpan operands,
 * not a claim about game allocations or compiler lowering.
 *
 * Source assembly: CSharpScript.dll
 * SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3
 */

using System;
using Cysharp.Threading.Tasks;
using UnrealEngine;

public partial class SubLevel
{
    // FieldDef 0x0400E3E7; observed in SetLevelVisibleNoBeginPlay,
    // SetLevelVisibleFinished, and Dispose.
    private CustomPromise<bool>? SetLevelVisiblePromise;

    // MethodDef 0x0601C56E; wrapper RVA 0x87c24c, code size 79.
    // Generated MoveNext 0x0604A495; RVA 0x1427070, code size 620.
    public async UniTask SetLevelVisible(
        bool bVisible,
        string reason,
        bool bWaitingLoading = true)
    {
        if (Level == null || !Level.IsLevelLoaded())
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)81,
                (ELogAuthor)19,
                "SetLevelVisible:失败,Level未加载完毕",
                new (string, object)[]
                {
                    ("path", Path),
                    ("LinkId", LinkId),
                    ("bVisible", bVisible),
                    ("reason", reason),
                });
            return;
        }

        if (bWaitingLoading && LoadState == ELoadLevelType.Loading)
        {
            await LoadPromise.Promise;
        }

        if (DependOnBeginPlayLogic)
        {
            // The field is read again after the optional load await; no
            // post-await Level recheck is represented or inferred.
            await SetLevelVisibleWithBeginPlay(Level!, bVisible, reason);
        }
        else
        {
            await SetLevelVisibleNoBeginPlay(bVisible, reason);
        }
    }

    // MethodDef 0x0601C56F; wrapper RVA 0x87c2a8, code size 71.
    // Generated MoveNext 0x0604A499; RVA 0x14274f8, code size 356.
    private async UniTask SetLevelVisibleWithBeginPlay(
        ULevelStreaming level,
        bool bVisible,
        string reason)
    {
        // The observed MoveNext allocates <>c__DisplayClass20_0 before the
        // IsLevelVisible comparison. This source-level lambda is a partial
        // projection of that lowering boundary; it does not claim that the
        // equal-visibility path performs no closure allocation.
        CustomPromise<bool>? setLevelVisiblePromise = null;

        if (level.IsLevelVisible() != bVisible)
        {
            setLevelVisiblePromise = new CustomPromise<bool>();
            Action onFinishFunc = () => setLevelVisiblePromise!.SetResult(true);
            FLevelStreamingVisibilityStatus @delegate =
                bVisible ? level.OnLevelShown : level.OnLevelHidden;

            @delegate.Add(onFinishFunc);
            level.SetShouldBeVisible(bVisible);
            await setLevelVisiblePromise.Promise;
            @delegate.Remove(onFinishFunc);
            InnerVisible = bVisible;
        }
    }

    // MethodDef 0x0601C570; wrapper RVA 0x87c2fc, code size 71.
    // Generated MoveNext 0x0604A497; RVA 0x1427314, code size 426.
    private async UniTask SetLevelVisibleNoBeginPlay(bool bVisible, string reason)
    {
        if (InnerVisible != bVisible)
        {
            InnerVisible = bVisible;
            UKuroSubLevelVisibleSubsystem subSystem =
                UKuroSubLevelVisibleSubsystem.GetSubSystem(GlobalData.GameInstance);
            SetLevelVisiblePromise = new CustomPromise<bool>();
            bool bSuccess = subSystem.SetLevelActorsVisible(
                LinkId,
                bVisible,
                FinishDelegate);

            // Read the current shared field after the native-boundary call;
            // the promise bool is intentionally discarded.
            await SetLevelVisiblePromise!.Promise;

            if (!bSuccess)
            {
                Singleton<Log>.Instance.Error(
                    (ELogModule)81,
                    (ELogAuthor)19,
                    "SetLevelVisibleNoBeginPlay:失败,Actors为空",
                    new (string, object)[]
                    {
                        ("path", Path),
                        ("LinkId", LinkId),
                        ("bVisible", bVisible),
                        ("reason", reason),
                    });
            }
        }
    }

    // MethodDef 0x0601C571; RVA 0x87c34f, code size 28.
    private void SetLevelVisibleFinished(int linkId)
    {
        if (linkId == LinkId)
        {
            SetLevelVisiblePromise?.SetResult(true);
        }
    }

    // MethodDef 0x0604A492; nested type 0x02009665; RVA 0x1426e76,
    // code size 13. The closure body is represented by the lambda above.

    // MethodDef 0x0601C56C; RVA 0x87c198, code size 92.
    public void Dispose()
    {
        if (SetLevelVisiblePromise != null && SetLevelVisiblePromise.IsPending)
        {
            SetLevelVisiblePromise.SetResult(false);
        }

        SetLevelVisiblePromise = null;
        UKuroSubLevelVisibleSubsystem.GetSubSystem(GlobalData.GameInstance)?.RemoveLevel(LinkId);
        Level = null;
        DelegateUtils.ReleaseManualReleaseDelegate(new Action<int>(SetLevelVisibleFinished));
    }
}
