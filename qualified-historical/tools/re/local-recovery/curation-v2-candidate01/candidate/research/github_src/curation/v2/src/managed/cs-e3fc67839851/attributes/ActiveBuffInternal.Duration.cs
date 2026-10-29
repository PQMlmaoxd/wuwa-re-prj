/*
 * CANDIDATE_ONLY / RECONSTRUCTED source-only view.
 * Method bodies are exact source-byte spans from the selected static evidence.
 * Activation: NOT_APPLIED; equivalence: NOT_ESTABLISHED; publication: DISABLED.
 */
public partial class ActiveBuffInternal
{
	// MethodDef 0x06018700; exact source lines 397-472.
	public void SetDuration(float? duration = null)
	{
		BuffDefinition config = Config;
		float num = 0.034f;
		if (config.DurationPolicy == EBuffDurationType.Infinite)
		{
			num = -1f;
		}
		else if (duration.HasValue)
		{
			num = duration.Value;
		}
		else if (config.DurationMagnitude.Length == 0 || config.DurationCalculationPolicy.Length == 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity? owner = GetOwner();
			string message = $"Buff 配置为hasDuration 但未配置DurationMagnitude或DurationCalculationPolicy, 强制将时间设为{0.034f}";
			(string, object) reference = ("BuffId", Id);
			instance.Error(CombatLog.EDebugModule.Buff, owner, message, new ReadOnlySpan<(string, object)>(ref reference));
			num = 0.034f;
		}
		else
		{
			float levelValue = AbilityUtils.GetLevelValue(Config.DurationMagnitude, Level, 0f);
			float num2 = OwnerBuffCompInternal?.CalculateDurationRate(Id, GetInstigatorBuffComponent()) ?? 1f;
			int[] durationCalculationPolicy = config.DurationCalculationPolicy;
			switch ((durationCalculationPolicy != null) ? new int?(durationCalculationPolicy[0]) : null)
			{
			case 1:
			{
				if (durationCalculationPolicy.Length < 4)
				{
					CombatLog instance2 = Singleton<CombatLog>.Instance;
					Entity? owner2 = GetOwner();
					string message2 = $"Buff 配置为hasDuration 但未配置DurationMagnitude或DurationCalculationPolicy, 将被重设为{0.034f}";
					global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("BuffId", Id);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", Handle);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("释放者", InstigatorId);
					instance2.Error(CombatLog.EDebugModule.Buff, owner2, message2, global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
					num = 0.034f;
					break;
				}
				float levelValue2 = AbilityUtils.GetLevelValue(config.DurationMagnitude2, Level, 0f);
				int attrId = durationCalculationPolicy[1];
				EBuffAttributeCaptureSource eBuffAttributeCaptureSource = (EBuffAttributeCaptureSource)durationCalculationPolicy[2];
				EAttributeBasedFloatCalculationType attrCalcPolicy = (EAttributeBasedFloatCalculationType)durationCalculationPolicy[3];
				BaseAttributeComponent baseAttributeComponent = ((eBuffAttributeCaptureSource == EBuffAttributeCaptureSource.Instigator) ? GetInstigatorAttributeSet() : GetOwnerAttributeSet());
				if (baseAttributeComponent == null)
				{
					CombatLog instance3 = Singleton<CombatLog>.Instance;
					Entity? owner3 = GetOwner();
					string message3 = $"Buff 找不到周期计算属性来源, 周期将被重设为{0.034f}";
					global::<>y__InlineArray4<(string, object)> buffer2 = default(global::<>y__InlineArray4<(string, object)>);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 0) = ("BuffId", Id);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 1) = ("handle", Handle);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 3) = ("释放者", InstigatorId);
					instance3.Error(CombatLog.EDebugModule.Buff, owner3, message3, global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer2, 4));
					num = 0.034f;
				}
				else
				{
					num = Math.Max((AbilityUtils.GetAttrValue(baseAttributeComponent, (EAttributeType)attrId, attrCalcPolicy) * levelValue * 0.0001f + levelValue2) * num2, 0.034f);
				}
				break;
			}
			default:
				num = levelValue * num2;
				break;
			}
		}
		DurationInternal = num;
		ResetDurationTimer(num);
	}
}
