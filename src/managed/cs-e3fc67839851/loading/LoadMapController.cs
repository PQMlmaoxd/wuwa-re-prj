/*
 * PERF04 PARTIAL SOURCE-LIKE FRAGMENT — NONCANONICAL / RECONSTRUCTED.
 * TypeDef: 0x02003474.
 * Full type: global::LoadMapController.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/796efa63c14dc6787fd3a5b794821dd15ce56ec010a2cfc3522e25c1e1943eff/source/_global/LoadMapController.cs
 * Selected body: LoadSubLevelsAsync (0x0601C4F8) and its generated
 * <LoadSubLevelsAsync>d__16.MoveNext (0x0604A444).
 * Omitted-members node: all other members remain external; no behavior stubs
 * or invented lifecycle members are supplied by this fragment.
 * `partial` marks reconstruction packaging only; it does not prove the
 * original declaration used the partial modifier.
 *
 * Static at-rest source-like projection. Compiler-generated async lowering is
 * projected once as readable async source; generated identity, method tokens,
 * and raw-CIL references remain in the provenance comments. External
 * controller, protocol, and UniTask types remain unresolved boundaries.
 * Source assembly: CSharpScript.dll SHA-256
 * e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.
 */

using Aki.Protocol;
using CSharpScript.Game.World.Controller;
using Cysharp.Threading.Tasks;

public partial class LoadMapController : ControllerBase<LoadMapController>
{
    // MethodDef 0x0601C4F8; RVA 0x8775d8; code size 55.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:7-48.
    // Generated MoveNext 0x0604A444; RVA 0x14228d4; code size 199.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:50-157.
    private async UniTask LoadSubLevelsAsync(SceneInformation sceneInformation)
    {
        ControllerBase<GameModeController>.Instance.LoadDataLayers(sceneInformation);
        ControllerBase<GameModeController>.Instance.UpdateStreamingQualityLevel();
        await ControllerBase<SubLevelController>.Instance.CheckLoadSubLevels(sceneInformation);
        ControllerBase<LoadingController>.Instance.SetProgress(40, null, 1, false, true);
    }
}
