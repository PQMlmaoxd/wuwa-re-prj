// ORIGINAL REFERENCE MODEL -- synthetic, not reconstructed or decompiled game source.
// This is an independently authored teaching model of selected skill-decision,
// cooldown-availability, and charge-restoration behavior. No original method body,
// proprietary string literal, game binary, or private corpus file is copied.
// BCL-only. No clock, timer, filesystem, process, network, native bridge, server
// state, animation window, or real callback delivery is represented here.
// Evidence anchors and the research limitations are stated in README.md.

using System;
using System.Collections.Generic;

namespace SkillDecisionReference;

// Reference identity is deliberate. Two objects with the same numeric fields are
// not interchangeable for selection; only the same object reference participates.
public sealed class Skill
{
    public Skill(int id, int group, int priority)
    {
        Id = id;
        Group = group;
        Priority = priority;
    }

    public int Id { get; }
    public int Group { get; }
    public int Priority { get; }
}

// The decision state is small on purpose. It is not a game component and does not
// model activation flags, filtering, sorting, a scheduler, or a cooldown component.
public sealed class SkillDecisions
{
    public Dictionary<int, List<Skill>?> ActiveGroups { get; } = new();

    public bool EasyInterrupt { get; set; }

    public bool ReadyEnd { get; set; } = true;

    // Null means debug mode was not read. When present, the callback is evaluated
    // for a current group-1 skill even when priority alone would accept.
    public Func<bool>? ReadDebug { get; set; }

    // Null means the cooldown component is absent and its answer is false.
    public Func<int, bool>? ReadCooldown { get; set; }

    // API preconditions: incoming is required. Null output/reasons arguments are
    // local throwaway lists; they are not returned or reassigned. These are model
    // boundaries, not claims about a recovered validator or exception contract.
    public bool Check(Skill incoming, List<Skill>? output = null, List<string>? reasons = null)
    {
        ArgumentNullException.ThrowIfNull(incoming);

        List<Skill> toInterrupt = output ?? new List<Skill>();
        List<string> failureReasons = reasons ?? new List<string>();

        if (incoming.Group == 1)
        {
            // Only the first entry is considered, and only that entry is compared.
            // Missing, null, or empty state declines nothing and changes no list.
            Skill? current = ActiveGroups.TryGetValue(1, out List<Skill>? group)
                && group != null
                && group.Count > 0
                ? group[0]
                : null;

            if (current != null)
            {
                bool higherPriority = current.Priority < incoming.Priority;
                bool equalPriorityEasy = current.Priority == incoming.Priority && EasyInterrupt;
                bool readyEnd = ReadyEnd;
                Func<bool>? readDebug = ReadDebug;
                bool debugMode = readDebug != null && readDebug();

                if (higherPriority || equalPriorityEasy || readyEnd || debugMode)
                {
                    // Preserve existing entries and do not de-duplicate.
                    toInterrupt.Add(current);
                }
                else
                {
                    failureReasons.Add(current.Priority.ToString());
                    failureReasons.Add(incoming.Priority.ToString());
                    failureReasons.Add(EasyInterrupt.ToString());
                    failureReasons.Add(ReadyEnd.ToString());
                    toInterrupt.Clear();
                    return false;
                }
            }
        }
        else
        {
            // A missing, null, or empty group declines nothing and changes no list.
            List<Skill>? otherGroup = ActiveGroups.TryGetValue(incoming.Group, out List<Skill>? value)
                ? value
                : null;

            if (otherGroup != null)
            {
                Func<int, bool>? readCooldown = ReadCooldown;
                foreach (Skill item in otherGroup)
                {
                    // Cooldown is read before reference selection. A hit rejects the
                    // whole call even when the group entry is not the incoming object.
                    if (readCooldown != null && readCooldown(item.Id))
                    {
                        failureReasons.Add($"Skill {item.Id} is on cooldown.");
                        toInterrupt.Clear();
                        return false;
                    }

                    if (ReferenceEquals(item, incoming))
                    {
                        toInterrupt.Add(item);
                    }
                }
            }
        }

        return true;
    }
}

// A caller-owned charge snapshot. RemainingCount and LimitCount are plain integers.
// No timer, finish stamp, queue, logging, or event state is inferred here.
public sealed class ChargeState
{
    public ChargeState(int remainingCount, int limitCount)
    {
        RemainingCount = remainingCount;
        LimitCount = limitCount;
    }

    public int RemainingCount { get; set; }

    public int LimitCount { get; set; }
}

// A null map represents an uninitialized component, not an empty component.
public sealed class CooldownAvailability
{
    private readonly IReadOnlyDictionary<long, ChargeState?>? _groups;

    public CooldownAvailability(IReadOnlyDictionary<long, ChargeState?>? groups)
    {
        _groups = groups;
    }

    public bool IsUnavailable(int id)
    {
        if (_groups == null)
        {
            throw new NullReferenceException("The cooldown component is uninitialized.");
        }

        if (!_groups.TryGetValue(id, out ChargeState? group))
        {
            return false;
        }

        if (group == null)
        {
            throw new NullReferenceException($"The cooldown group for skill {id} is null.");
        }

        return group.RemainingCount <= 0;
    }
}

// The pending work is an explicit port. The model does not claim that an original
// queue was FIFO; the authored test double is a controlled input assumption only.
public interface IPendingCooldowns
{
    int SkillIdCount { get; }

    int DurationCount { get; }

    int PopSkillId();

    float PopDuration();
}

public static class ChargeCompletion
{
    // elapsed is a caller-supplied model input. This guard is an explicit adapter
    // boundary, not a recovered validation rule. Durations themselves are not
    // validated: zero, negative, and NaN values are used exactly as supplied.
    public static void Complete(
        ChargeState state,
        float elapsed,
        IPendingCooldowns pending,
        Action notify,
        Action<int, float> schedule)
    {
        ArgumentNullException.ThrowIfNull(state);
        ArgumentNullException.ThrowIfNull(pending);
        ArgumentNullException.ThrowIfNull(notify);
        ArgumentNullException.ThrowIfNull(schedule);

        if (float.IsNaN(elapsed) || float.IsInfinity(elapsed) || elapsed < 0f)
        {
            throw new ArgumentOutOfRangeException(
                nameof(elapsed),
                elapsed,
                "Elapsed time must be finite and non-negative.");
        }

        unchecked
        {
            state.RemainingCount++;
        }

        (int? nextId, float? nextDuration) = ReadNextPair(pending);
        while (nextDuration.HasValue && elapsed >= nextDuration.Value)
        {
            unchecked
            {
                state.RemainingCount++;
            }

            elapsed -= nextDuration.Value;
            (nextId, nextDuration) = ReadNextPair(pending);
        }

        // Cap only the upper side. A negative or otherwise low count is not repaired.
        if (state.RemainingCount > state.LimitCount)
        {
            state.RemainingCount = state.LimitCount;
        }

        notify();

        if (nextId.HasValue && nextDuration.HasValue)
        {
            schedule(nextId.Value, nextDuration.Value - elapsed);
        }
    }

    private static (int? SkillId, float? Duration) ReadNextPair(IPendingCooldowns pending)
    {
        if (pending.SkillIdCount <= 0 || pending.DurationCount <= 0)
        {
            return (null, null);
        }

        // The id is removed first. If the duration read throws, the id stays removed.
        int skillId = pending.PopSkillId();
        float duration = pending.PopDuration();
        return (skillId, duration);
    }
}
