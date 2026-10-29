/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020031ee (global::RoleElementComponent)
 * Canonical source: .../a945abfe7fe9e21c894c44a58ce979ba79b1d0e2061cbe5ac269307448ba47ce/source/_global/RoleElementComponent.cs
 * Canonical raw methods: .../a945abfe7fe9e21c894c44a58ce979ba79b1d0e2061cbe5ac269307448ba47ce/evidence/raw/methods.jsonl
 * Canonical raw types: .../a945abfe7fe9e21c894c44a58ce979ba79b1d0e2061cbe5ac269307448ba47ce/evidence/raw/types.jsonl
 * Scope: selected whole source-level bodies for R22 rows 210-216 only.
 * RoleElementType, RoleElementEnergy, RoleElementEnergyMax, OnEnd, and
 * unresolved callback/lifecycle groups remain external without implementations.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using Aki.Protocol.Summon;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;

public partial class RoleElementComponent : EntityComponent
{
    // Field tokens: ActorComponent 0x0400d652; AttributeComponent 0x0400d653;
    // BuffComponent 0x0400d654; SummonedEntity 0x0400d655;
    // TagComponent 0x0400d656; NeedCheckEnergy 0x0400d657;
    // TriggerEnergy 0x0400d658; QteBuffId 0x0400d659.
    private CharacterActorComponent? ActorComponent;
    private BaseAttributeComponent? AttributeComponent;
    private CharacterBuffComponent? BuffComponent;
    private EntityHandle? SummonedEntity;
    private BaseTagComponent? TagComponent;
    private bool NeedCheckEnergy;
    public float TriggerEnergy;
    private long QteBuffId;

    // Property token 0x170023ec; getter MethodDef 0x0601a852; raw row 861.
    // Setter MethodDef 0x0601a851; raw row 860. One private property owns both selected accessors.
    private bool ElementActivated
    {
        get
        {
            return TagComponent?.HasExactTag(GameplayTagDefine.EGameplayTagId["功能.功能制作.QTE.激活QTE"]) ?? false;
        }
        set
        {
            if (ElementActivated == value)
            {
                return;
            }
            CharacterActorComponent? actorComponent = ActorComponent;
            if (actorComponent == null || !actorComponent.IsAutonomousProxy)
            {
                return;
            }
            if (value)
            {
                bool flag = ControllerBase<FormationDataController>.Instance.IsBattleMulti();
                QteBuffId = (flag ? 3028 : 3029);
                BuffComponent?.AddBuff(QteBuffId, new AddBuffParam
                {
                    InstigatorId = BuffComponent.CreatureDataId,
                    Reason = "RoleElementComponent获取激活QTE的Tag"
                });
                if (flag)
                {
                    MultiQteGuide();
                }
            }
            else
            {
                BuffComponent?.RemoveBuff(QteBuffId, -1, "RoleElementComponent移除激活QTE的Tag");
            }
        }
    }

    // MethodDef 0x0601a84f; protected virtual override; source lines 70-91; raw row 858.
    protected override bool OnStart()
    {
        ActorComponent = base.Entity.GetComponent<CharacterActorComponent>();
        AttributeComponent = base.Entity.GetComponent<BaseAttributeComponent>();
        BuffComponent = base.Entity.GetComponent<CharacterBuffComponent>();
        TagComponent = base.Entity.CheckGetComponent<BaseTagComponent>();
        AttributeComponent?.AddListener(EAttributeType.ElementEnergy, OnElementEnergyChanged, "RoleElementComponent");
        Singleton<EventSystem>.Instance.AddWithTarget<Entity, bool>(base.Entity, EEventName.RoleOnStateInherit, OnStateInherit);
        Singleton<EventSystem>.Instance.AddWithTarget(base.Entity, EEventName.CharOnRevive, OnRoleRevive);
        Singleton<EventSystem>.Instance.AddWithTarget(base.Entity, EEventName.CharOnRoleDeadTargetSelf, OnRoleDead);
        Singleton<EventSystem>.Instance.Add(EEventName.OnBattleStateChanged, (Delegate)new Action<bool>(OnBattleStateChanged));
        Singleton<EventSystem>.Instance.AddWithTarget<HitInformation, HitContext>(base.Entity, EEventName.CharHitLocal, CheckEnergyOnce);
        Singleton<EventSystem>.Instance.AddWithTarget<HitInformation, HitContext>(base.Entity, EEventName.CharBeHitLocal, CheckEnergyOnce);
        SummonedEntity = PhantomUtil.GetSummonedEntity(base.Entity, ESummonType.ConcomitantCustom);
        WorldEntity worldEntity = SummonedEntity?.Entity;
        if (worldEntity != null && !Singleton<EventSystem>.Instance.HasWithTarget((object)worldEntity, EEventName.CharHitLocal, (Delegate)new Action<HitInformation, HitContext>(CheckEnergyOnce)))
        {
            Singleton<EventSystem>.Instance.AddWithTarget<HitInformation, HitContext>(worldEntity, EEventName.CharHitLocal, CheckEnergyOnce);
        }
        NeedCheckEnergy = true;
        return true;
    }

    // Getter MethodDef 0x0601a852; private; source lines 28-31; raw row 861.
    // Setter MethodDef 0x0601a851; private; source lines 32-61; raw row 860.
    // The getter/setter are represented together by the one ElementActivated property above.

    // MethodDef 0x0601a856; private; source lines 112-118; raw row 865.
    private void OnElementEnergyChanged(EAttributeType attributeId, float newValue, float oldValue)
    {
        CheckEnergyFull(newValue);
        EElementType roleElementType = RoleElementType;
        Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.CharOnElementEnergyChanged, roleElementType, newValue, oldValue);
        Singleton<EventSystem>.Instance.Emit(EEventName.CharOnElementEnergyChanged, roleElementType, newValue, oldValue);
    }

    // MethodDef 0x0601a857; private; source lines 120-142; raw row 866.
    private void CheckEnergyFull(float energy)
    {
        if (energy >= TriggerEnergy - float.Epsilon)
        {
            SceneTeamModel instance = ModelBase<SceneTeamModel>.Instance;
            bool flag = instance != null && (instance.GetTeamItem(base.Entity.Id, new GetTeamItemOptions
            {
                ParamType = ETeamParamType.EntityId
            })?.IsControl()).GetValueOrDefault();
            if (!ElementActivated && flag && ControllerBase<FormationDataController>.Instance.GlobalIsInFight)
            {
                ElementActivated = true;
                BuffComponent?.TriggerEvents(EBuffTriggerType.ForWhenOneElementActivated, BuffComponent, new Partial_RequirementPayload
                {
                    ElementType = RoleElementType
                });
            }
        }
        else
        {
            ElementActivated = false;
        }
    }

    // MethodDef 0x0601a85b; public; source lines 170-179; raw row 870.
    public void TriggerEvents(Entity newRole)
    {
        RoleElementComponent component = newRole.GetComponent<RoleElementComponent>();
        Partial_RequirementPayload payload = new Partial_RequirementPayload
        {
            ElementType = RoleElementType
        };
        BuffComponent?.TriggerEvents(EBuffTriggerType.ForWhenElementFusionGoDown, component?.BuffComponent, payload);
        component?.BuffComponent?.TriggerEvents(EBuffTriggerType.ForWhenElementFusionGoBattle, BuffComponent, payload);
    }

    // MethodDef 0x0601a85e; public; source lines 199-210; raw row 873.
    // Optional parameter constant: consumeBuffId = 0L.
    public void ClearElementEnergy(Entity instigator, long consumeBuffId = 0L)
    {
        if (consumeBuffId == 0L)
        {
            consumeBuffId = 3027L;
        }
        BuffComponent?.AddBuff(consumeBuffId, new AddBuffParam
        {
            InstigatorId = (instigator.GetComponent<CreatureDataComponent>()?.GetCreatureDataId() ?? 0),
            Reason = "ClearElementEnergy消耗元素能量"
        });
    }
}