// Authored minimal host adapter for RE-SKILL-INTERRUPTION01.
// The four slices between BEGIN/END EXACT markers are exact byte substrings of
// src/reconstructed/managed/full/CSharpScript/shards/c5eb641d6bd91ab09f1282c38ccf7625bab6b7c9e1a98d50573abd6d7c01158d/source/_global/BaseSkillComponent.cs
// (SHA256 0392810d5c956b2b7ccb9360660190c266be414611b617e64a6a4c126e805696; LF only).
// They are not rewritten or normalized. This is not a whole-class reconstruction
// and does not emulate EntityComponent.
using System.Collections.Generic;

internal partial class BaseSkillComponentHost
{
	internal readonly Dictionary<int, List<Skill>> ActivatedSkillGroup = new Dictionary<int, List<Skill>>();

	internal bool IsMainSkillEasyInterrupt;

	internal bool IsMainSkillReadyEnd = true;

	internal CharacterSkillCdComponent? SkillCdComp;

// BEGIN EXACT CurrentSkill canonical lines 569-579
	public Skill? CurrentSkill
	{
		get
		{
			if (!ActivatedSkillGroup.TryGetValue(1, out List<Skill> value) || value == null || value.Count <= 0)
			{
				return null;
			}
			return value[0];
		}
	}
// END EXACT CurrentSkill

// BEGIN EXACT CheckSkillCanInterrupt canonical lines 1646-1651
	private bool CheckSkillCanInterrupt(Skill skill, List<Skill> skillsToInterrupt, List<string> reasons)
	{
		int groupId = skill.SkillInfo.GroupId;
		int interruptLevel = skill.InterruptLevel;
		return DoCheckInterrupt(groupId, interruptLevel, skillsToInterrupt, reasons, skill);
	}
// END EXACT CheckSkillCanInterrupt

// BEGIN EXACT DoCheckInterrupt canonical lines 1653-1712
	protected bool DoCheckInterrupt(int group, int interruptLevel, List<Skill>? skillsToInterrupt = null, List<string>? reasons = null, Skill? skill = null)
	{
		if (skillsToInterrupt == null)
		{
			skillsToInterrupt = new List<Skill>();
		}
		if (reasons == null)
		{
			reasons = new List<string>();
		}
		bool flag = true;
		if (group == 1)
		{
			Skill currentSkill = CurrentSkill;
			if (currentSkill != null)
			{
				bool num = currentSkill.InterruptLevel < interruptLevel;
				bool flag2 = currentSkill.InterruptLevel == interruptLevel && IsMainSkillEasyInterrupt;
				bool isMainSkillReadyEnd = IsMainSkillReadyEnd;
				bool flag3 = ModelBase<SkillCdModel>.Instance?.SkillDebugMode ?? false;
				if (num || flag2 || isMainSkillReadyEnd || flag3)
				{
					skillsToInterrupt.Add(currentSkill);
				}
				else
				{
					flag = false;
					reasons.Add(currentSkill.InterruptLevel.ToString());
					reasons.Add(interruptLevel.ToString());
					reasons.Add(IsMainSkillEasyInterrupt.ToString());
					reasons.Add(IsMainSkillReadyEnd.ToString());
				}
			}
		}
		else
		{
			List<Skill> list = (ActivatedSkillGroup.ContainsKey(group) ? ActivatedSkillGroup[group] : null);
			if (list != null)
			{
				foreach (Skill item in list)
				{
					if (IsSkillInCd(item.SkillId))
					{
						flag = false;
						reasons.Add($"技能{item.SkillId}处于CD中");
						break;
					}
					if (item == skill)
					{
						skillsToInterrupt.Add(item);
					}
				}
			}
		}
		if (!flag)
		{
			skillsToInterrupt.Clear();
		}
		return flag;
	}
// END EXACT DoCheckInterrupt

// BEGIN EXACT IsSkillInCd canonical lines 1973-1980
	public bool IsSkillInCd(int skillId)
	{
		if (SkillCdComp == null)
		{
			return false;
		}
		return SkillCdComp.IsSkillInCd(skillId);
	}
// END EXACT IsSkillInCd


	// Authored test-only wrapper. The canonical CheckSkillCanInterrupt is private;
	// the exact method above is the code that reads GroupId/InterruptLevel and
	// forwards the incoming skill reference.
	public bool CheckSkillCanInterruptForTest(Skill skill, List<Skill> skillsToInterrupt, List<string> reasons)
	{
		return CheckSkillCanInterrupt(skill, skillsToInterrupt, reasons);
	}

	// Authored test-only wrapper. The canonical DoCheckInterrupt is protected.
	public bool DoCheckInterruptForTest(int group, int interruptLevel, List<Skill>? skillsToInterrupt = null, List<string>? reasons = null, Skill? skill = null)
	{
		return DoCheckInterrupt(group, interruptLevel, skillsToInterrupt, reasons, skill);
	}
}
