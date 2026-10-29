// Authored test scaffolding for RE-SKILL-INTERRUPTION01.
// This file contains NO recovered method body. The four exact canonical
// BaseSkillComponent slices live in the private fixture.
using System;
using System.Collections.Generic;

// Minimal authored plain-reference Skill shape. Canonical Skill.cs:13 declares
// a plain class; line 153 SkillId, line 159 SkillInfo, and lines 167-177
// InterruptLevel over a backing field. No equality or comparison operator is
// authored, so `item == skill` in the exact DoCheckInterrupt body remains
// reference equality.
public sealed class Skill
{
    public int SkillId { get; set; }

    public SkillInfo SkillInfo { get; set; } = new SkillInfo();

    public int InterruptLevel { get; set; }

    // Test-only marker. The exact CurrentSkill getter does not read it; group 1
    // uses it to prove no Active predicate is applied.
    public bool Active { get; set; } = true;
}

public sealed class SkillInfo
{
    public int GroupId { get; set; }
}

// Authored controlled dependency for the exact IsSkillInCd forwarding body.
// The delegate is the CD oracle; Calls is an observation trace. A throwing
// delegate is used to test exception ordering.
public sealed class CharacterSkillCdComponent
{
    public Func<int, bool>? IsSkillInCdImpl { get; set; }

    public List<int> Calls { get; } = new List<int>();

    public bool IsSkillInCd(int skillId)
    {
        Calls.Add(skillId);
        return IsSkillInCdImpl?.Invoke(skillId) ?? false;
    }
}

// Authored controlled dependency for the exact DebugModelBase expression.
// SkillDebugModeGetter, when set, is evaluated on every property read and may
// throw; this is how the eager-read-before-OR discriminator is tested.
public sealed class SkillCdModel
{
    private bool _skillDebugMode;

    public Func<bool>? SkillDebugModeGetter { get; set; }

    public bool SkillDebugMode
    {
        get => SkillDebugModeGetter?.Invoke() ?? _skillDebugMode;
        set => _skillDebugMode = value;
    }
}

// Authored minimal static holder matching the observed ModelBase<T>.Instance
// access. It is a test double, not a reconstruction of the whole model layer.
public class ModelBase<T> where T : class
{
    public static T? Instance { get; set; }
}
