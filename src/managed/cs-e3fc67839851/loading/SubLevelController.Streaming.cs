/*
 * PERF04 PARTIAL SOURCE-LIKE FRAGMENT — NONCANONICAL / RECONSTRUCTED.
 * TypeDef: 0x020046e8.
 * Full type: CSharpScript.Game.World.Controller.SubLevelController.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/030a6e55936d3ad6fdcbb8fe5a2e986e14b59f6ad0a05a5518faf8e716b654d0/source/CSharpScript/Game/World/Controller/SubLevelController.cs
 * Selected bodies: CheckLoadSubLevels (0x0602F360), LoadSubLevels
 * (0x0602F362), LoadSubLevel (0x0602F361), OnLoadSubLevel (0x0602F365),
 * and generated callback <OnLoadSubLevel>b__0 (0x0604AE4A), with the three
 * selected async MoveNext identities retained in the method inventory.
 * Omitted-members node: ChangeSubLevelInternal, other controller members,
 * assistants, and provider bodies remain external; no behavior stubs are
 * supplied by this fragment.
 * `partial` marks reconstruction packaging only; it does not prove the
 * original declaration used the partial modifier.
 *
 * Compiler-generated async lowering is projected once as readable async
 * source. The tuple arrays below are readable projections of observed
 * compiler-generated inline-array/span operands, not actual game allocations.
 * Source assembly: CSharpScript.dll SHA-256
 * e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.
 */

using System;
using System.Collections.Generic;
using Aki.Config;
using Aki.Protocol;
using CSharpScript.Game.Ui;
using CSharpScript.Game.World.Model;
using Cysharp.Threading.Tasks;
using UnrealEngine;

namespace CSharpScript.Game.World.Controller;

public partial class SubLevelController : ControllerWithAssistantBase<SubLevelController>
{
    // MethodDef 0x0602F360; RVA 0xb2f5c0; code size 63.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:159-210.
    // Generated MoveNext 0x0604AE4F; RVA 0x1461cc8; code size 418.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:212-452.
    public async UniTask<bool> CheckLoadSubLevels(SceneInformation sceneInformation)
    {
        Dictionary<string, bool> levelsWithVisible = new Dictionary<string, bool>();
        if (sceneInformation.LoadedSubLevels != null && sceneInformation.LoadedSubLevels.Count > 0)
        {
            foreach (string level in sceneInformation.LoadedSubLevels)
            {
                levelsWithVisible[level] = sceneInformation.HideSubLevels.IndexOf(level) < 0;
            }
        }
        else
        {
            InstanceDungeon? instanceDungeon = ModelBase<GameModeModel>.Instance.InstanceDungeon;
            if (instanceDungeon.HasValue && instanceDungeon.Value.SubLevelsLength > 0)
            {
                foreach (string level in instanceDungeon.Value.SubLevelsIter())
                {
                    levelsWithVisible[level] = true;
                }
            }
        }

        if (levelsWithVisible.Count == 0)
        {
            return true;
        }

        return await LoadSubLevels(levelsWithVisible);
    }

    // MethodDef 0x0602F362; RVA 0xb2f658; code size 63.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:454-504.
    // Generated MoveNext 0x0604AE55; RVA 0x1462488; code size 356.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:506-715.
    public async UniTask<bool> LoadSubLevels(Dictionary<string, bool> levelPath)
    {
        List<UniTask<bool>> tasks = new List<UniTask<bool>>();
        foreach (KeyValuePair<string, bool> level in levelPath)
        {
            if (level.Value)
            {
                tasks.Add(LoadSubLevel(level.Key, level.Value));
            }
        }

        foreach (KeyValuePair<string, bool> level in levelPath)
        {
            if (!level.Value)
            {
                tasks.Add(LoadSubLevel(level.Key, level.Value));
            }
        }

        await UniTask.WhenAll(tasks);
        return true;
    }

    // MethodDef 0x0602F361; RVA 0xb2f60c; code size 63.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:717-768.
    // Generated MoveNext 0x0604AE53; RVA 0x1462090; code size 959.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:770-1149.
    public async UniTask<bool> LoadSubLevel(string levelPath, bool bVisibleAfterLoad)
    {
        SubLevelModel subLevelModel = ModelBase<SubLevelModel>.Instance;
        SubLevel subLevel = subLevelModel.GetPreloadSubLevel(levelPath);
        if (subLevel != null)
        {
            subLevelModel.MovePreloadSubLevelToSubLevel(levelPath);
            if (subLevel.LoadState == ELoadLevelType.Loading)
            {
                await subLevel.LoadPromise.Promise;
            }

            await subLevel.SetLevelVisible(bVisibleAfterLoad, "SubLevelController.LoadSubLevel");
            return true;
        }

        subLevel = subLevelModel.GetSubLevel(levelPath);
        if (subLevel != null && subLevel.LoadState == ELoadLevelType.Done)
        {
            await subLevel.SetLevelVisible(bVisibleAfterLoad, "SubLevelController.LoadSubLevel");
            return true;
        }

        if (subLevel == null)
        {
            subLevel = subLevelModel.AddSubLevel(levelPath, bVisibleAfterLoad);
        }

        if (subLevel.LoadState == ELoadLevelType.None)
        {
            subLevel.LoadState = ELoadLevelType.Loading;
            ULoadMapNotify? loadMapNotify = GlobalData.GameInstance.场景加载通知器;
            FName path = FNameUtil.GetDynamicFName(levelPath) ?? FName.NAME_None;
            int linkId = loadMapNotify.LoadStreamLevel(
                in path,
                subLevel.LoadVisibleParam,
                bShouldBlockOnLoad: false);
            subLevel.LinkId = linkId;
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)3,
                "SubLevelController:加载子关卡",
                new (string, object)[]
                {
                    ("Path", levelPath),
                    ("LinkId", linkId),
                });
        }

        Singleton<UiTimeDilation>.Instance.AddWaitSetTimeDilationTag(
            "SubLevelController.LoadSubLevel" + subLevel.LinkId);
        bool result = await subLevel.LoadPromise.Promise;
        Singleton<UiTimeDilation>.Instance.DeleteWaitSetTimeDilationTag(
            "SubLevelController.LoadSubLevel" + subLevel.LinkId);
        return result;
    }

    // MethodDef 0x0602F365; RVA 0xb2f71c; code size 467.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1151-1381.
    // The compiler-generated callback is 0x0604AE4A, RVA 0x14613b8, code size
    // 145; DIRECT_IL_VERIFIED at evidence/performance/PERF04_DIRECT_CIL.txt:1383-1446.
    public void OnLoadSubLevel(int linkId, string levelPath, ULevelStreaming? level)
    {
        SubLevelModel subLevelModel = ModelBase<SubLevelModel>.Instance;
        SubLevel subLevel = null;

        foreach (SubLevel candidate in subLevelModel.GetAllPreloadSubLevels().Values)
        {
            if (candidate.LinkId == linkId)
            {
                subLevel = candidate;
                break;
            }
        }

        if (subLevel == null)
        {
            foreach (SubLevel candidate in subLevelModel.GetAllSubLevels().Values)
            {
                if (candidate.LinkId == linkId)
                {
                    subLevel = candidate;
                    break;
                }
            }
        }

        if (subLevel == null)
        {
            subLevel = subLevelModel.GetUnloadSubLevel(levelPath);
            if (subLevel != null)
            {
                subLevel.LoadPromise?.SetResult(true);
                FName path = FNameUtil.GetDynamicFName(subLevel.Path) ?? FName.NAME_None;
                subLevel.UnLoadLinkId = GlobalData.GameInstance.场景加载通知器.UnloadStreamLevel(
                    in path,
                    bShouldBlockOnLoad: true);
                return;
            }
        }

        if (subLevel == null)
        {
            Singleton<Log>.Instance.Warn(
                (ELogModule)81,
                (ELogAuthor)3,
                "SubLevelController:加载的子关卡不存在",
                new (string, object)[]
                {
                    ("LinkId", linkId),
                    ("Level", levelPath),
                    ("LevelStreaming", level?.IsValid() ?? false),
                });
            return;
        }

        subLevel.OnLevelLoad(level).ContinueWith(() =>
        {
            Singleton<Log>.Instance.Info(
                (ELogModule)81,
                (ELogAuthor)19,
                "SubLevelController:子关卡加载完成",
                new (string, object)[]
                {
                    ("Level", levelPath),
                    ("LinkId", linkId),
                    ("LevelStreaming", level?.IsValid() ?? false),
                });
        });
    }
}
