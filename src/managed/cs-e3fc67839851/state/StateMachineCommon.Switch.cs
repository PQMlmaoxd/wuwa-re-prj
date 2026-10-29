/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source, original file
 * structure, a standalone-compilation claim, or semantic approval. The
 * metadata type is public, abstract, nonpartial, and generic arity two; the
 * source-only overlay uses `partial` only to disclose its composition seam.
 * No generic constraints, fields, constructors, helpers, getters, stubs,
 * base-class declarations, or dependency method bodies are introduced here.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original full type: CSharpScript.Core.Utils.StateMachine.StateMachineCommon`2<TObject,TState>
 * Original type: public abstract, nonpartial, generic arity 2, no method generic arity.
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/7d83dcfb7d0d91a0b591380471198ea186d64db500a33252949f27daed462de6/source/CSharpScript/Core/Utils/StateMachine/StateMachineCommon_g2.cs
 * Canonical source SHA256: ba478c1f50ebc432e9fb8210dd7f44069ddc3f1d79df2e049c18dfc39508f679
 * Selected canonical lines: 211-243, copied as the complete Switch method.
 * Selected method: 0x060461d9; dependencies 0x060461c9 Enter and 0x060461cb
 * Exit are retained as evidence-only IL artifacts and are not reconstructed
 * as source bodies in this overlay.
 */

using System;
using System.Collections.Generic;
using CSharpScript.Core.Utils.StateMachine;

namespace CSharpScript.Core.Utils.StateMachine
{
	public abstract partial class StateMachineCommon<TObject, TState>
	{
	public bool Switch(TState state, bool triggerEvent = true, bool triggerSubNode = true, params object[] pairs)
	{
		if (CurrentNode == null)
		{
			Log instance = Singleton<Log>.Instance;
			(string, object) reference = ("state", state);
			instance.Error((ELogModule)26, (ELogAuthor)14, "状态机没有启动", new ReadOnlySpan<(string, object)>(ref reference));
			return false;
		}
		StateMachineCommon<TObject, TState> state2 = GetState(state);
		if (state2 == null)
		{
			Log instance2 = Singleton<Log>.Instance;
			(string, object) reference2 = ("state", state);
			instance2.Error((ELogModule)26, (ELogAuthor)14, "状态机切换失败，目标节点不存在", new ReadOnlySpan<(string, object)>(ref reference2));
			return false;
		}
		if (EqualityComparer<TState>.Default.Equals(state, CurrentNode.State))
		{
			if (!CurrentNode.CanReEnter())
			{
				return false;
			}
			CurrentNode.ReEnter();
			return true;
		}
		StateMachineCommon<TObject, TState> currentNode = CurrentNode;
		currentNode.Exit(state2, triggerEvent, triggerSubNode, pairs);
		CurrentNode = state2;
		state2.Enter(currentNode, triggerEvent, triggerSubNode, pairs);
		OnSwitchState(currentNode, state2);
		return true;
	}
	}
}
