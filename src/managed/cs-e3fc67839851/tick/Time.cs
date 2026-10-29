// PERF07: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x02000054.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/3e1facc4e6f297ee4666b05aa99bf905c9c0f54eaf0502dac773a1d9bf1a3721/source/_global/Time.cs
// Source boundary: observed field context plus exactly the 15 selected
// MethodDefs recorded in PERF07_DIRECT_CIL.txt. Unselected Time members are
// intentionally omitted, not replaced with no-op methods.
// CSharpScript.Typing.KuroTime is external context only; no new KuroTime body
// or native implementation is reconstructed here.

using System;
using CSharpScript.Typing;

public partial class Time : Singleton<Time>
{
    // OBSERVED field context. Field initializers are retained as source
    // context; they are not a claim about constructor CIL.
    private const float MsToSecond = 0.001f;

    private int FrameInternal;

    private double FrameNowInternal;

    private double FramePlayerTimeInternal;

    private double FrameWorldTimeInternal;

    private double FramePlayerWorldTimeInternal;

    private double FrameFlowTimeInternal;

    private float DeltaTimeMsInternal;

    private double SystemNowInternal;

    private double SystemStart;

    private double ServerTimeStampOffsetInternal;

    private double ServerFlowTimeStampOffsetInternal;

    private double ServerStopTimeStampOffsetInner;

    private double ServerTimeOffsetInternal;

    private double ServerCombatTimeOffsetInternal;

    private float WorldTimeDilationInternal = 1f;

    private float FlowTimeDilationInternal = 1f;

    private float InverseSelfCenteredTimeDilationInternal = 1f;

    public float OriginTimeDilation = 1f;

    public int LastPauseTimeFrame;

    public int LastResumeTimeFrame;

    public double LastServerTime;

    private int AfterTickFrameInternal;

    // MethodDef 0x06000190; RVA 0x9f32; code size 13.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body/source metadata observes the
    // KuroTime call and offset addition (metadata field access offset 6).
    public double ServerTimeStamp => KuroTime.GetMilliseconds64() + ServerTimeStampOffsetInternal;

    // MethodDef 0x06000194; RVA 0x9fd5; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns the world dilation
    // field (metadata field access offset 1).
    public float TimeDilation => WorldTimeDilationInternal;

    // MethodDef 0x06000195; RVA 0x9fdd; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns the flow dilation
    // field (metadata field access offset 1).
    public float FlowTimeDilation => FlowTimeDilationInternal;

    // MethodDef 0x06000197; RVA 0xa006; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns the inverse
    // self-centered dilation field (metadata field access offset 1).
    public float InverseSelfCenteredTimeDilation => InverseSelfCenteredTimeDilationInternal;

    // MethodDef 0x060001a1; RVA 0xa0b2; code size 13.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body uses the observed float
    // factor 0.001f after loading DeltaTimeMsInternal (metadata constant
    // offset 6).
    public float DeltaTimeSeconds => DeltaTimeMsInternal * 0.001f;

    // MethodDef 0x060001a2; RVA 0xa0c0; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns FrameInternal
    // (metadata field access offset 1).
    public int Frame => FrameInternal;

    // MethodDef 0x060001a3; RVA 0xa0c8; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns FrameNowInternal
    // (metadata field access offset 1).
    public double Now => FrameNowInternal;

    // MethodDef 0x060001a4; RVA 0xa0d0; code size 17.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body retains the literal
    // double factor 0.0010000000474974513 (metadata constant offset 6).
    public double NowSeconds => FrameNowInternal * 0.0010000000474974513;

    // MethodDef 0x060001ab; RVA 0xa130; code size 7.
    // SOURCE_VERIFIED_ONLY: official direct-CIL attempts 1 and 2 failed; no
    // raw CIL section is claimed. Canonical body returns FrameFlowTimeInternal
    // (metadata field access offset 1).
    public double FlowTime => FrameFlowTimeInternal;

    // MethodDef 0x06000191; RVA 0x9f40; code size 58.
    // DIRECT_IL_VERIFIED; raw section "Time::SetServerTimeStamp". Critical
    // offsets: IL_0001 NetInfo.Instance, IL_0006 RttMs, IL_0010 float divide,
    // IL_0011 conv.r8, IL_0012 add, IL_0013 KuroTime.GetMilliseconds64,
    // IL_0018 sub, IL_001c offset stfld, IL_0030 bne.un.s, IL_0034 conditional
    // LastServerTime stfld. The float divide and operation order are retained.
    public void SetServerTimeStamp(double serverTimeStampMs)
    {
        double lastServerTime = ServerTimeStampOffsetInternal =
            serverTimeStampMs + (double)(Singleton<NetInfo>.Instance.RttMs / 2f) - KuroTime.GetMilliseconds64();
        if (LastServerTime == 0.0)
        {
            LastServerTime = lastServerTime;
        }
    }

    // MethodDef 0x0600019d; RVA 0xa08f; code size 8.
    // DIRECT_IL_VERIFIED; raw section "Time::SetTimeDilation". Critical
    // offsets: IL_0001 ldarg.1, IL_0002 direct WorldTimeDilationInternal
    // stfld; no validation branch or exception region is observed.
    public void SetTimeDilation(float timeDilation)
    {
        WorldTimeDilationInternal = timeDilation;
    }

    // MethodDef 0x0600019e; RVA 0xa098; code size 8.
    // DIRECT_IL_VERIFIED; raw section "Time::SetFlowTimeDilation". Critical
    // offsets: IL_0001 ldarg.1, IL_0002 direct FlowTimeDilationInternal
    // stfld; no validation branch or exception region is observed.
    public void SetFlowTimeDilation(float timeDilation)
    {
        FlowTimeDilationInternal = timeDilation;
    }

    // MethodDef 0x0600019f; RVA 0xa0a1; code size 8.
    // DIRECT_IL_VERIFIED; raw section "Time::SetInverseSelfCenteredTimeDilation".
    // Critical offsets: IL_0001 ldarg.1, IL_0002 direct
    // InverseSelfCenteredTimeDilationInternal stfld; no validation branch or
    // exception region is observed.
    public void SetInverseSelfCenteredTimeDilation(float timeDilation)
    {
        InverseSelfCenteredTimeDilationInternal = timeDilation;
    }

    // MethodDef 0x060001b0; RVA 0xa170; code size 132.
    // DIRECT_IL_VERIFIED; raw section "Time::Initialize". Critical offsets:
    // IL_0002/0011/0020/002f/003a/0049 reset Frame, Now, World, Flow, Delta,
    // and SystemNow; IL_004f/0057/005c capture UTC Unix milliseconds into
    // SystemStart; IL_0063/006e/0079 restore Origin, World, and Flow scales.
    // No reset of PlayerTime, PlayerWorldTime, inverse scale, LastServerTime,
    // offsets, or AfterTickFrameInternal is added.
    public void Initialize()
    {
        FrameInternal = 0;
        FrameNowInternal = 0.0;
        FrameWorldTimeInternal = 0.0;
        FrameFlowTimeInternal = 0.0;
        DeltaTimeMsInternal = 0f;
        SystemNowInternal = 0.0;
        SystemStart = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
        OriginTimeDilation = 1f;
        WorldTimeDilationInternal = 1f;
        FlowTimeDilationInternal = 1f;
    }

    // MethodDef 0x060001b1; RVA 0xa200; code size 172.
    // DIRECT_IL_VERIFIED; raw section "Time::Tick". Critical offsets:
    // IL_0002 Delta stfld; IL_000f unchecked Frame add / IL_0010 stfld;
    // IL_001d conv.r8 then IL_001e add for Now; IL_0033/0034 player;
    // IL_004a/004b world; IL_0061/0062 and IL_0069/006a PlayerWorld;
    // IL_0080/0081 and IL_0088/0089 Flow; IL_0091/0099/009e/00a5/00a6
    // UTC Unix-millisecond SystemNow update. No branch, clamp, pause guard,
    // or exception region is observed. Float scales are converted before
    // multiplication and compound products retain their left-associative
    // order.
    public void Tick(float delta)
    {
        unchecked
        {
            DeltaTimeMsInternal = delta;
            FrameInternal++;
            FrameNowInternal += delta;
            FramePlayerTimeInternal += (double)delta * (double)InverseSelfCenteredTimeDilationInternal;
            FrameWorldTimeInternal += (double)delta * (double)WorldTimeDilationInternal;
            FramePlayerWorldTimeInternal +=
                (double)delta * (double)WorldTimeDilationInternal * (double)InverseSelfCenteredTimeDilationInternal;
            FrameFlowTimeInternal +=
                (double)delta * (double)WorldTimeDilationInternal * (double)FlowTimeDilationInternal;
            SystemNowInternal = (double)DateTimeOffset.UtcNow.ToUnixTimeMilliseconds() - SystemStart;
        }
    }
}
