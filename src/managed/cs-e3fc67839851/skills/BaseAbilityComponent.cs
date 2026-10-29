/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e50 (global::BaseAbilityComponent)
 * Scope: four selected whole method bodies only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using CSharpScript.Game.Common.Event;
using UnrealEngine;

public partial class BaseAbilityComponent : EntityComponent, IStaticVariableResetter
{
    // Observed members used by the selected view. The surrounding class and
    // all non-selected lifecycle members remain outside this overlay.
    private UBaseAbilitySystemComponent? TsAbilityComponentInternal;
    protected readonly Dictionary<int, HashSet<TGameplayEventCallback>> GameplayEventCallbacks = new Dictionary<int, HashSet<TGameplayEventCallback>>();

    // Token 0x060184a9; original public, non-virtual.
    public UAsyncTaskWaitGameplayEvent CreateGameplayEventTask(Action<FGameplayTag, FGameplayEventData> callback)
    {
        UAsyncTaskWaitGameplayEvent? task = UAsyncTaskWaitGameplayEvent.ListenForGameplayEvent(TsAbilityComponentInternal);
        task.EventReceived.Add(callback);
        return task;
    }

    // Token 0x060184aa; original private, non-virtual.
    private void GameplayEventCallback(FGameplayTag eventTag, FGameplayEventData payload)
    {
        int tagId = eventTag.TagId();
        if (tagId == 0 || !GameplayEventCallbacks.TryGetValue(tagId, out HashSet<TGameplayEventCallback> listeners))
        {
            return;
        }

        string tagName = GameplayTagUtils.GetNameByTagId(tagId);
        // This list is a membership snapshot; the payload itself is not copied.
        foreach (TGameplayEventCallback listener in new List<TGameplayEventCallback>(listeners))
        {
            try
            {
                listener(tagId, payload);
            }
            catch (Exception exception)
            {
                Log logger = Singleton<Log>.Instance;
                ReadOnlySpan<(string, object)> details =
                [
                    ("gameplayEvent", tagName),
                    ("error", exception.Message)
                ];
                logger.ErrorWithStack((ELogModule)10, (ELogAuthor)29, "gameplayEvent事件回调执行异常", exception, details);
            }
        }
    }

    // Token 0x060184ab; original public, non-virtual.
    public void AddGameplayEventListener(int tagId, TGameplayEventCallback callback)
    {
        if (callback == null)
        {
            Log logger = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> details =
            [
                ("gameplayEvent", GameplayTagUtils.GetNameByTagId(tagId)),
                ("callbackName", callback?.Method.Name)
            ];
            logger.Error((ELogModule)29, (ELogAuthor)29, "回调函数添加失败", details);
            return;
        }

        if (!GameplayEventCallbacks.TryGetValue(tagId, out HashSet<TGameplayEventCallback> listeners))
        {
            listeners = new HashSet<TGameplayEventCallback>();
            GameplayEventCallbacks[tagId] = listeners;
        }

        if (listeners.Contains(callback))
        {
            Log logger = Singleton<Log>.Instance;
            ReadOnlySpan<(string, object)> details =
            [
                ("gameplayEvent", GameplayTagUtils.GetNameByTagId(tagId)),
                ("callbackName", callback.Method.Name)
            ];
            logger.Warn((ELogModule)29, (ELogAuthor)29, "重复添加回调函数", details);
        }
        else
        {
            listeners.Add(callback);
        }
    }

    // Token 0x060184ac; original public, non-virtual.
    public void RemoveGameplayEventListener(int tagId, TGameplayEventCallback callback)
    {
        if (GameplayEventCallbacks.TryGetValue(tagId, out HashSet<TGameplayEventCallback> listeners))
        {
            listeners.Remove(callback);
        }
    }
}
