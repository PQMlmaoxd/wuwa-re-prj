# Source Index

Public-copy context: this index preserves the curated baseline and later scoped
checkpoint records below. The additional
[qualified historical source](../examples/mechanics/gameplay-showcase/README.md#qualified-historical-source-map)
is not silently promoted into that accepted baseline. Original whole-file hashes
remain historical identities; six authored provenance headers were redacted in
publication derivatives without changing method bodies. The
[preparation provenance](PUBLICATION_PROVENANCE.json) records those mappings, not
a new original-binary authentication or a reseal of later documentation edits.

The 18 historical files are separate from the 131 curated paths. Five additional
`SOURCE_ONLY` method excerpts are in the [damage snapshot view](../qualified-research/damage-snapshots/BaseDamageComponent.SourceOnly.cs):
`AllocateSnapshot`, `ReplaceAttacker`, `ProcessWhenCounter`, `ExecDamageEffect`
and `ProcessDamageOptimize` (TypeDef `0x02002E65`; MethodDefs `0x0601864F`,
`0x06018650`, `0x06018651`, `0x06018654`, `0x06018662`). The
[walkthrough](DAMAGE_SNAPSHOT_FLOW.md) and [call-site evidence](../qualified-research/damage-snapshots/evidence.json)
describe checks of ten methods and 24 selected CIL calls, not complete source/CIL
or runtime equivalence. These excerpts do not activate historical candidates.

Current active import authority is the append-only `metadata/curation-batches.jsonl` index (header + `gameplay66` + `performance25`) over the immutable `metadata/imports.v5.json` baseline and `metadata/curation/bulk-curate01.json` campaign catalog. It maps **104 source files / 533 qualified identities**: 246 `DIRECT_CIL_FULL`, 280 `SOURCE_ONLY`, 6 historical `UNKNOWN`, and 1 abstract `METADATA_ONLY`. The 48 associated identities (11 baseline + 37 new) are not additional ordinary C# bodies. The root is not buildable or runtime-verified; C# compilation was not run.

This index is navigation, not a 533-record dump. Per-method identities, roles, proof references, and limitations remain in the typed catalog and accepted evidence. Historical R38 status (8 FULL) is separate from old D Calculation status (6 UNKNOWN) and is unchanged. R54 `StateMachineCommon.Enter` is not a candidate. Formula is a disjoint partial complement with D/T. AsyncTask/Completion are source files, not evidence of a TaskGraph runtime executor.

| File | Record | Subsystem | Ordinary / associated | Evidence (F/S/U) | Known dependency boundary | Related views | Proof catalog |
| --- | --- | --- | ---: | --- | --- | --- | --- |
| [AiLibrary.cs](../src/managed/cs-e3fc67839851/ai/AiLibrary.cs) | `G_AILIBRARY` | ai | 3 / 0 | 3/0/0 | OBSERVED_SINGLETON_CONTEXT: Info (source line 43); OBSERVED_SINGLETON_CONTEXT: Log (source line 46); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 29), BlackboardC... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [ActiveBuffInternal.cs](../src/managed/cs-e3fc67839851/attributes/ActiveBuffInternal.cs) | `G_ACTIVEBUFFINTERNAL` | attributes | 9 / 0 | 0/9/0 | OBSERVED_BASE: IActiveBuff, IStaticVariableResetter (source line 17); OBSERVED_CONTEXT_TYPES: BuffDefinition (line 23), Stat (line 38); 13 expression-bodied source-context prope... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseAttributeComponent.cs](../src/managed/cs-e3fc67839851/attributes/BaseAttributeComponent.cs) | `G_BASEATTRIBUTECOMPONENT` | attributes | 11 / 0 | 0/11/0 | OBSERVED_BASE: EntityComponent, IAttributeSet, IStaticVariableResetter (source line 16); OBSERVED_SINGLETON_CONTEXT: Log (source line 242); OBSERVED_MEMBER_OR_HELPER_REFERENCE: ... | G_BASEATTRIBUTESTORAGE | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseAttributeStorage.cs](../src/managed/cs-e3fc67839851/attributes/BaseAttributeStorage.cs) | `G_BASEATTRIBUTESTORAGE` | attributes | 7 / 0 | 6/1/0 | OBSERVED_BASE: EntityComponent, IAttributeSet, IStaticVariableResetter (source line 20); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 226); OBSERVED_SINGLETON_CONTEXT: Log... | G_BASEATTRIBUTECOMPONENT | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterAttributeComponent.cs](../src/managed/cs-e3fc67839851/attributes/CharacterAttributeComponent.cs) | `G_CHARACTERATTRIBUTECOMPONENT` | attributes | 4 / 0 | 4/0/0 | OBSERVED_BASE: BaseAttributeComponent (source line 21); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 69); OBSERVED_MEMBER_OR_HELPER_REFERENCE: ApplyModToAttributeToAbili... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [LockValue.cs](../src/managed/cs-e3fc67839851/attributes/LockValue.cs) | `G_LOCKVALUE` | attributes | 3 / 0 | 0/3/0 | OBSERVED_BASE: BuffEffect (source line 14); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseAttributeComponent (line 45), CheckGetComponent (line 45); UNKNOWN_GAME_CONTEXT: unselected ... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseBuffComponent.cs](../src/managed/cs-e3fc67839851/buffs/BaseBuffComponent.cs) | `G_BASEBUFFCOMPONENT` | buffs | 7 / 0 | 0/7/0 | OBSERVED_BASE: EntityComponent, IBuffComponent, IStaticVariableResetter (source line 18); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 35); OBSERVED_SINGLETON_CONTEXT: Inf... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BuffController.cs](../src/managed/cs-e3fc67839851/buffs/BuffController.cs) | `G_BUFFCONTROLLER` | buffs | 1 / 0 | 1/0/0 | OBSERVED_BASE: ControllerBase<BuffController> (source line 24); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 44); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BuffModel (line 32),... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RoleKuroFastCollisionComponent.cs](../src/managed/cs-e3fc67839851/collision/RoleKuroFastCollisionComponent.cs) | `G_ROLEKUROFASTCOLLISIONCOMPONENT` | collision | 9 / 0 | 9/0/0 | OBSERVED_BASE: BaseKuroFastCollisionComponent (source line 16); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseKuroFastCollisionComponent (line 8), BaseTagComponent (line 19), BulletM... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseDamageComponent.cs](../src/managed/cs-e3fc67839851/damage/BaseDamageComponent.cs) | `G_BASEDAMAGECOMPONENT` | damage | 10 / 4 | 12/2/0 | OBSERVED_BASE: EntityComponent (source line 27); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 82); OBSERVED_SINGLETON_CONTEXT: CombatNet (source line 207); OBSERVED_MEMBER... | G_BASEDAMAGECOMPONENT_EXECDAMAGEEVENT | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseDamageComponent.ExecDamageEvent.cs](../src/managed/cs-e3fc67839851/damage/BaseDamageComponent.ExecDamageEvent.cs) | `G_BASEDAMAGECOMPONENT_EXECDAMAGEEVENT` | damage | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent (source line 26); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 34); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseDamageComponent (line 9), Cre... | G_BASEDAMAGECOMPONENT | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterHitComponent.cs](../src/managed/cs-e3fc67839851/damage/CharacterHitComponent.cs) | `G_CHARACTERHITCOMPONENT` | damage | 27 / 1 | 13/15/0 | OBSERVED_BASE: BaseHitComponent, IStaticVariableResetter (source line 37); OBSERVED_SINGLETON_CONTEXT: BulletConstant (source line 350); OBSERVED_SINGLETON_CONTEXT: CombatLog (s... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterStatisticsComponent.DamageCombat.cs](../src/managed/cs-e3fc67839851/damage/CharacterStatisticsComponent.DamageCombat.cs) | `G_CHARACTERSTATISTICSCOMPONENT_DAMAGECOMBAT` | damage | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent, IStaticVariableResetter (source line 29); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterStatisticsComponent (line 3), EntityComponent (line 29), ... | G_CHARACTERSTATISTICSCOMPONENT_DAMAGERECEIVER, G_CHARACTERSTATISTICSCOMPONENT_LISTVIEWFILTER | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterStatisticsComponent.DamageReceiver.cs](../src/managed/cs-e3fc67839851/damage/CharacterStatisticsComponent.DamageReceiver.cs) | `G_CHARACTERSTATISTICSCOMPONENT_DAMAGERECEIVER` | damage | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent, IStaticVariableResetter (source line 31); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterStatisticsComponent (line 3), DamageData (line 36), Entit... | G_CHARACTERSTATISTICSCOMPONENT_DAMAGECOMBAT, G_CHARACTERSTATISTICSCOMPONENT_LISTVIEWFILTER | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterStatisticsComponent.ListViewFilter.cs](../src/managed/cs-e3fc67839851/damage/CharacterStatisticsComponent.ListViewFilter.cs) | `G_CHARACTERSTATISTICSCOMPONENT_LISTVIEWFILTER` | damage | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent, IStaticVariableResetter (source line 29); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterStatisticsComponent (line 3), EntityComponent (line 29); ... | G_CHARACTERSTATISTICSCOMPONENT_DAMAGECOMBAT, G_CHARACTERSTATISTICSCOMPONENT_DAMAGERECEIVER | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RoleElementComponent.cs](../src/managed/cs-e3fc67839851/elements/RoleElementComponent.cs) | `G_ROLEELEMENTCOMPONENT` | elements | 7 / 0 | 3/4/0 | OBSERVED_BASE: EntityComponent (source line 23); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 86); OBSERVED_MEMBER_OR_HELPER_REFERENCE: ActorComponent (line 25), Attribu... | G_ROLEELEMENTRESOURCE | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RoleElementResource.cs](../src/managed/cs-e3fc67839851/elements/RoleElementResource.cs) | `G_ROLEELEMENTRESOURCE` | elements | 2 / 0 | 2/0/0 | OBSERVED_BASE: EntityComponent (source line 16); OBSERVED_MEMBER_OR_HELPER_REFERENCE: AttributeComponent (line 10), EntityComponent (line 16), RoleElementComponent (line 8); UNK... | G_ROLEELEMENTCOMPONENT | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Entity.cs](../src/managed/cs-e3fc67839851/entity/Entity.cs) | `P_ENTITY` | entity | 1 / 0 | 0/1/0 | ObjectBase; IGameBudgetManagedObject; IStaticVariableResetter; Stat; TickComponentManager | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [EntityComponent.cs](../src/managed/cs-e3fc67839851/entity/EntityComponent.cs) | `P_ENTITYCOMPONENT` | entity | 1 / 0 | 0/1/0 | IComponentDependency; IEntityComponentCleaner; Entity; EEntityDisableKey; Log | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [EntitySystem.cs](../src/managed/cs-e3fc67839851/entity/EntitySystem.cs) | `P_ENTITYSYSTEM` | entity | 3 / 0 | 0/3/0 | Singleton; ObjectSystem; Entity; TickEntityManager; GameBudgetInterfaceController | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RangeComponent.cs](../src/managed/cs-e3fc67839851/entity/RangeComponent.cs) | `P_RANGECOMPONENT` | entity | 11 / 2 | 13/0/0 | EntityComponent; EventSystem; TEmitEventParams; AActor; EntityHandle; ITriggerShape | P_RANGECOMPONENT_LIFECYCLERECOVERY; RangePendingEvents excluded backlog | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RangeComponent.LifecycleRecovery.cs](../src/managed/cs-e3fc67839851/entity/RangeComponent.LifecycleRecovery.cs) | `P_RANGECOMPONENT_LIFECYCLERECOVERY` | entity | 2 / 0 | 2/0/0 | RangeComponent; EntityComponent; EventSystem; ResourceSystem; UObjectPool | P_RANGECOMPONENT; RangePendingEvents excluded backlog | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TickComponentInfo.cs](../src/managed/cs-e3fc67839851/entity/TickComponentInfo.cs) | `P_TICKCOMPONENTINFO` | entity | 1 / 0 | 0/1/0 | EntityComponent | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TickComponentManager.cs](../src/managed/cs-e3fc67839851/entity/TickComponentManager.cs) | `P_TICKCOMPONENTMANAGER` | entity | 5 / 0 | 0/5/0 | TickComponentInfo; EntityComponent; GameBudgetInterfaceController | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TickEntityGroup.cs](../src/managed/cs-e3fc67839851/entity/TickEntityGroup.cs) | `P_TICKENTITYGROUP` | entity | 1 / 0 | 0/1/0 | Entity | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TickEntityManager.cs](../src/managed/cs-e3fc67839851/entity/TickEntityManager.cs) | `P_TICKENTITYMANAGER` | entity | 8 / 0 | 0/8/0 | Entity; TickEntityGroup; Stat; GameBudgetInterfaceController | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Event.Emit.Enum2.DirectCIL.cs](../src/managed/cs-e3fc67839851/event/Event.Emit.Enum2.DirectCIL.cs) | `E18` | event | 1 / 0 | 1/0/0 | Event<TName,TType> context and generic constraints; EventArgs<T1,T2> constructor MethodDef 0x06000457; IEventArgs; EmitInternal<T1> MethodDef 0x06000434 | historical baseline | metadata/imports.v5.json; metadata/migration.json |
| [Event.EmitInternal.DirectCIL.cs](../src/managed/cs-e3fc67839851/event/Event.EmitInternal.DirectCIL.cs) | `E17` | event | 1 / 0 | 1/0/0 | Event<TName,TType> context; IEventArgs; ConditionListener<T1>; Stat, Log, inline-array helpers, and mutation helpers | historical baseline | metadata/imports.v5.json; metadata/migration.json |
| [Event.ExecuteHandlesInternal.cs](../src/managed/cs-e3fc67839851/event/Event.ExecuteHandlesInternal.cs) | `E13` | event | 1 / 0 | 1/0/0 | Event<TName,TType> context and retained Enum or Delegate constraints; IEventArgs.Invoke; ConditionListener<T1>; Stat, Log, and compiler-generated inline-array helpers | historical baseline | metadata/imports.v5.json; metadata/migration.json |
| [EventArgs.Invoke.Enum2.cs](../src/managed/cs-e3fc67839851/event/EventArgs.Invoke.Enum2.cs) | `P_EVENTARGS_INVOKE_ENUM2` | event | 1 / 0 | 1/0/0 | EventArgs<T1,T2>; Action<T1,T2>; GenericEventHandler; CALL_METHOD_06000450 unresolved | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [EventSystem.EmitWithTarget.Enum2.cs](../src/managed/cs-e3fc67839851/event/EventSystem.EmitWithTarget.Enum2.cs) | `P_EVENTSYSTEM_EMITWITHTARGET_ENUM2` | event | 1 / 0 | 1/0/0 | EventSystem; Event<EEventName,Delegate>; GetTargetEvent; Log | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [GameplayEventTrigger.cs](../src/managed/cs-e3fc67839851/events/GameplayEventTrigger.cs) | `G_GAMEPLAYEVENTTRIGGER` | events | 4 / 0 | 0/4/0 | OBSERVED_BASE: Trigger (source line 17); OBSERVED_MEMBER_OR_HELPER_REFERENCE: ActorUtils (line 49), BaseAbilityComponent (line 32), FGameplayEventData (line 42), GameplayTagUtil... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TagSwitchedTask.cs](../src/managed/cs-e3fc67839851/events/TagSwitchedTask.cs) | `G_TAGSWITCHEDTASK` | events | 2 / 0 | 0/2/0 | OBSERVED_BASE: ITagTask (source line 12); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 16), TagComponent (line 16); UNKNOWN_GAME_CONTEXT: unselected members, call... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Trigger.cs](../src/managed/cs-e3fc67839851/events/Trigger.cs) | `G_TRIGGER` | events | 5 / 0 | 0/5/0 | OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 57); OBSERVED_SINGLETON_CONTEXT: Time (source line 90); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterTriggerComponent (line 2... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Calculation.cs](../src/managed/cs-e3fc67839851/gameplay/Calculation.cs) | `T` | gameplay | 1 / 0 | 0/1/0 | Calculation context and IStaticVariableResetter; AttributeSnapshot; EAttributeType operands with independently unproved numeric identities | historical baseline | metadata/imports.v5.json; metadata/migration.json |
| [Calculation.DamageFormula.cs](../src/managed/cs-e3fc67839851/gameplay/Calculation.DamageFormula.cs) | `D` | gameplay | 8 / 0 | 1/1/6 | Calculation context and IStaticVariableResetter; Aki.Config and Aki.Protocol; SnapshotPayload, AttributeSnapshot, and external calculation helpers | historical baseline | metadata/imports.v5.json; metadata/migration.json |
| [Calculation.FormulaOneInvocation.cs](../src/managed/cs-e3fc67839851/gameplay/Calculation.FormulaOneInvocation.cs) | `G_CALCULATION_FORMULAONEINVOCATION` | gameplay | 1 / 1 | 2/0/0 | RELATED_SOURCE: D (same root TypeDef 0x02002EB2; 8 current keys; selected keys disjoint from Formula); R... | D | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterInputCommands.cs](../src/managed/cs-e3fc67839851/input/CharacterInputCommands.cs) | `G_CHARACTERINPUTCOMMANDS` | input | 2 / 0 | 2/0/0 | OBSERVED_BASE: EntityComponent, IInputHandler, IStaticVariableResetter (source line 43); OBSERVED_SINGLETON_CONTEXT: Info (source line 61); OBSERVED_SINGLETON_CONTEXT: Time (sou... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [InputBlueprintFunctionLibrary.cs](../src/managed/cs-e3fc67839851/input/InputBlueprintFunctionLibrary.cs) | `G_INPUTBLUEPRINTFUNCTIONLIBRARY` | input | 1 / 0 | 1/0/0 | OBSERVED_BASE: UBlueprintFunctionLibrary, IUnrealUObject, IUnrealObject (source line 38); UNKNOWN_GAME_CONTEXT: unselected members, callers, and external runtime effects are not... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [InputFunctionCommon.cs](../src/managed/cs-e3fc67839851/input/InputFunctionCommon.cs) | `G_INPUTFUNCTIONCOMMON` | input | 4 / 0 | 4/0/0 | OBSERVED_BASE: IStaticVariableResetter (source line 29); OBSERVED_SINGLETON_CONTEXT: EntitySystem (source line 92); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseAttributeComponent (... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [InputFunctionVisionSkill2.cs](../src/managed/cs-e3fc67839851/input/InputFunctionVisionSkill2.cs) | `G_INPUTFUNCTIONVISIONSKILL2` | input | 2 / 0 | 2/0/0 | OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 43), BlackboardController (line 118), CharacterActorComponent (line 38), CharacterVisionComponent (line 85), EInputSt... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [GameModePromise.cs](../src/managed/cs-e3fc67839851/loading/GameModePromise.cs) | `P` | loading | 2 / 0 | 2/0/0 | AkiUniTask<bool>; Cysharp.Threading.Tasks.UniTask; GlobalData, Net, and Log; external task completion, cancellation, and scheduling effects | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [LoadMapController.cs](../src/managed/cs-e3fc67839851/loading/LoadMapController.cs) | `P_LOADMAPCONTROLLER` | loading | 1 / 1 | 1/1/0 | ControllerBase; SceneInformation; SubLevelController; LoadingController; GameModeController | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SubLevel.cs](../src/managed/cs-e3fc67839851/loading/SubLevel.cs) | `P_SUBLEVEL` | loading | 3 / 1 | 2/2/0 | GameModePromise; ULevelStreaming; UKuroSubLevelVisibleSubsystem; GlobalData; DelegateUtils | P_SUBLEVEL_VISIBILITY | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SubLevel.Visibility.cs](../src/managed/cs-e3fc67839851/loading/SubLevel.Visibility.cs) | `P_SUBLEVEL_VISIBILITY` | loading | 5 / 4 | 9/0/0 | SubLevel fields; CustomPromise<bool>; UKuroSubLevelVisibleSubsystem; delegates; GlobalData | P_SUBLEVEL | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SubLevelController.Execution.cs](../src/managed/cs-e3fc67839851/loading/SubLevelController.Execution.cs) | `P_SUBLEVELCONTROLLER_EXECUTION` | loading | 8 / 3 | 4/7/0 | ControllerWithAssistantBase; SubLevelProcess; AsyncTask; CustomPromise<bool>; TaskSystem; SubLevelModel; GameModeModel | P_SUBLEVELCONTROLLER_STREAMING | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SubLevelController.Streaming.cs](../src/managed/cs-e3fc67839851/loading/SubLevelController.Streaming.cs) | `P_SUBLEVELCONTROLLER_STREAMING` | loading | 4 / 4 | 4/4/0 | SubLevelModel; GameModeModel; SceneInformation; UniTask; UiTimeDilation | P_SUBLEVELCONTROLLER_EXECUTION | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [WorldGlobal.cs](../src/managed/cs-e3fc67839851/loading/WorldGlobal.cs) | `P_WORLDGLOBAL` | loading | 2 / 0 | 2/0/0 | ULoadMapNotify; GlobalData; SubLevelController; delegates | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterFightStateComponent.cs](../src/managed/cs-e3fc67839851/movement/CharacterFightStateComponent.cs) | `G_CHARACTERFIGHTSTATECOMPONENT` | movement | 9 / 0 | 9/0/0 | OBSERVED_BASE: EntityComponent (source line 30); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 145); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterUnifiedStateComponent (li... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterGlideComponent.cs](../src/managed/cs-e3fc67839851/movement/CharacterGlideComponent.cs) | `G_CHARACTERGLIDECOMPONENT` | movement | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent (source line 35); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CharacterActorComponent (line 20), EnterGlideState (line 41), EntityComponent (line 35), Se... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterMoveComponent.cs](../src/managed/cs-e3fc67839851/movement/CharacterMoveComponent.cs) | `G_CHARACTERMOVECOMPONENT` | movement | 2 / 0 | 2/0/0 | OBSERVED_BASE: BaseMoveComponent, IComponentDependency, IStaticVariableResetter (source line 39); OBSERVED_SINGLETON_CONTEXT: Time (source line 84); OBSERVED_MEMBER_OR_HELPER_RE... | G_CHARACTERMOVECOMPONENT_JUMPHANDOFF, G_CHARACTERMOVECOMPONENT_LANDING | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterMoveComponent.JumpHandoff.cs](../src/managed/cs-e3fc67839851/movement/CharacterMoveComponent.JumpHandoff.cs) | `G_CHARACTERMOVECOMPONENT_JUMPHANDOFF` | movement | 6 / 0 | 6/0/0 | OBSERVED_BASE: BaseMoveComponent, IComponentDependency, IStaticVariableResetter (source line 44); OBSERVED_SINGLETON_CONTEXT: Net (source line 202); OBSERVED_SINGLETON_CONTEXT: ... | G_CHARACTERMOVECOMPONENT, G_CHARACTERMOVECOMPONENT_LANDING | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterMoveComponent.Landing.cs](../src/managed/cs-e3fc67839851/movement/CharacterMoveComponent.Landing.cs) | `G_CHARACTERMOVECOMPONENT_LANDING` | movement | 2 / 0 | 2/0/0 | OBSERVED_BASE: BaseMoveComponent, IComponentDependency, IStaticVariableResetter (source line 36); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 45); OBSERVED_SINGLETON_CO... | G_CHARACTERMOVECOMPONENT, G_CHARACTERMOVECOMPONENT_JUMPHANDOFF | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterUnifiedStateComponent.Landing.cs](../src/managed/cs-e3fc67839851/movement/CharacterUnifiedStateComponent.Landing.cs) | `G_CHARACTERUNIFIEDSTATECOMPONENT_LANDING` | movement | 5 / 0 | 5/0/0 | OBSERVED_BASE: BaseUnifiedStateComponent, IComponentDependency (source line 35); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 144); OBSERVED_SINGLETON_CONTEXT: EventSystem... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletCollisionSystem.cs](../src/managed/cs-e3fc67839851/projectiles/BulletCollisionSystem.cs) | `G_BULLETCOLLISIONSYSTEM` | projectiles | 13 / 0 | 6/7/0 | OBSERVED_BASE: BulletSystemBase (source line 31); OBSERVED_SINGLETON_CONTEXT: BulletConstant (source line 304); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 204); OBSERV... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletController.cs](../src/managed/cs-e3fc67839851/projectiles/BulletController.cs) | `G_BULLETCONTROLLER` | projectiles | 7 / 0 | 0/7/0 | OBSERVED_BASE: ControllerBase<BulletController> (source line 25); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 106); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 3... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletDataLogic.cs](../src/managed/cs-e3fc67839851/projectiles/BulletDataLogic.cs) | `G_BULLETDATALOGIC` | projectiles | 4 / 0 | 0/4/0 | OBSERVED_FIELD_TYPE: BulletLogicType_C? Data (source line 22) is the selected getter cache source; selected getters read masked fields of that external data type.; OBSERVED_MEMB... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletHitCountUtil.cs](../src/managed/cs-e3fc67839851/projectiles/BulletHitCountUtil.cs) | `G_BULLETHITCOUNTUTIL` | projectiles | 1 / 0 | 1/0/0 | OBSERVED_SINGLETON_CONTEXT: BulletConstant (source line 23); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BulletInfo (line 17), CharacterFollowComponent (line 29), GetComponent (line 29... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletModel.cs](../src/managed/cs-e3fc67839851/projectiles/BulletModel.cs) | `G_BULLETMODEL` | projectiles | 1 / 0 | 1/0/0 | OBSERVED_BASE: ModelBase<BulletModel> (source line 33); OBSERVED_SINGLETON_CONTEXT: CombatNet (source line 43); OBSERVED_SINGLETON_CONTEXT: MathUtils (source line 41); OBSERVED_... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BulletUtil.cs](../src/managed/cs-e3fc67839851/projectiles/BulletUtil.cs) | `G_BULLETUTIL` | projectiles | 1 / 0 | 1/0/0 | OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 43), GetComponent (line 38), LevelTagComponent (line 38); UNKNOWN_GAME_CONTEXT: unselected members, callers, and exte... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [AsyncTask.cs](../src/managed/cs-e3fc67839851/scheduling/AsyncTask.cs) | `P_ASYNCTASK` | scheduling | 2 / 1 | 2/1/0 | TaskBase; UniTask<bool>; TaskInterruptOptions | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Completion.cs](../src/managed/cs-e3fc67839851/scheduling/Completion.cs) | `P_COMPLETION` | scheduling | 7 / 0 | 4/3/0 | UniTaskCompletionSource<T>; EPromiseState; CancellationToken; AkiUniTask<T> | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [PriorityQueue.cs](../src/managed/cs-e3fc67839851/scheduling/PriorityQueue.cs) | `PSCHED_HEAP` | scheduling | 15 / 0 | 5/10/0 | Comparison<T>, List<T>, Dictionary<T,int>, IClearable, Log; comparer and heap populations unresolved | historical baseline | metadata/imports.v5.json; metadata/scheduling-source.v1.json |
| [Queue.cs](../src/managed/cs-e3fc67839851/scheduling/Queue.cs) | `PEXEC_QUEUE` | scheduling | 19 / 0 | 4/15/0 | T, IClearable, Log; external queue populations unresolved; six explicit Enumerator members count as ordinary | historical baseline | metadata/imports.v5.json; metadata/task-execution-source.v1.json |
| [TaskBase.cs](../src/managed/cs-e3fc67839851/scheduling/TaskBase.cs) | `PEXEC_BASE` | scheduling | 12 / 1 (+1 abstract) | 6/7/0 | TaskInterruptOptions, Func<bool>, Action<bool>, UniTaskCompletionSource<bool>, Log; runtime completion unresolved | historical baseline | metadata/imports.v5.json; metadata/task-execution-source.v1.json |
| [TaskGraph.cs](../src/managed/cs-e3fc67839851/scheduling/TaskGraph.cs) | `PSCHED_GRAPH` | scheduling | 6 / 6 | 5/7/0 | IGraphNode, IGraphNodeWithDependencies, IRunOptions, UniTask, CompositeError, PriorityQueue<string>, TaskUtils, Log | historical baseline | metadata/imports.v5.json; metadata/scheduling-source.v1.json |
| [TaskSystem.cs](../src/managed/cs-e3fc67839851/scheduling/TaskSystem.cs) | `PEXEC_SYSTEM` | scheduling | 14 / 4 | 13/5/0 | TaskBase, Queue<TaskBase>, CustomPromise, UniTask, TimerSystem.GameplayTimeInstance, Log; thread placement unresolved | historical baseline | metadata/imports.v5.json; metadata/task-execution-source.v1.json |
| [TaskUtils.cs](../src/managed/cs-e3fc67839851/scheduling/TaskUtils.cs) | `PSCHED_UTILS` | scheduling | 6 / 0 | 2/4/0 | IGraphNodeWithDependencies, OneOf, graph result types; dictionary ordering and callers unresolved | historical baseline | metadata/imports.v5.json; metadata/scheduling-source.v1.json |
| [BaseAbilityComponent.cs](../src/managed/cs-e3fc67839851/skills/BaseAbilityComponent.cs) | `G_BASEABILITYCOMPONENT` | skills | 4 / 0 | 0/4/0 | OBSERVED_BASE: EntityComponent, IStaticVariableResetter (source line 17); OBSERVED_SINGLETON_CONTEXT: Log (source line 51); OBSERVED_MEMBER_OR_HELPER_REFERENCE: EntityComponent ... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseSkillCdComponent.cs](../src/managed/cs-e3fc67839851/skills/BaseSkillCdComponent.cs) | `G_BASESKILLCDCOMPONENT` | skills | 1 / 0 | 0/1/0 | OBSERVED_BASE: EntityComponent (source line 15); OBSERVED_MEMBER_OR_HELPER_REFERENCE: EntityComponent (line 15), PassiveSkillCdInfo (line 18); UNKNOWN_GAME_CONTEXT: unselected m... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseSkillComponent.cs](../src/managed/cs-e3fc67839851/skills/BaseSkillComponent.cs) | `G_BASESKILLCOMPONENT` | skills | 25 / 3 | 6/22/0 | OBSERVED_BASE: EntityComponent (source line 41); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 262); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 339); OBSERVED_MEM... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterPassiveSkillComponent.cs](../src/managed/cs-e3fc67839851/skills/CharacterPassiveSkillComponent.cs) | `G_CHARACTERPASSIVESKILLCOMPONENT` | skills | 9 / 2 | 0/11/0 | OBSERVED_BASE: EntityComponent (source line 19); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 68); OBSERVED_SINGLETON_CONTEXT: Log (source line 148); OBSERVED_MEMBER_OR_HE... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterSkillCdComponent.cs](../src/managed/cs-e3fc67839851/skills/CharacterSkillCdComponent.cs) | `G_CHARACTERSKILLCDCOMPONENT` | skills | 1 / 0 | 0/1/0 | OBSERVED_BASE: BaseSkillCdComponent (source line 14); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseSkillCdComponent (line 14), GroupSkillCdInfo (line 17); UNKNOWN_GAME_CONTEXT: unse... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterSkillComponent.JumpHandoff.cs](../src/managed/cs-e3fc67839851/skills/CharacterSkillComponent.JumpHandoff.cs) | `G_CHARACTERSKILLCOMPONENT_JUMPHANDOFF` | skills | 1 / 0 | 1/0/0 | OBSERVED_BASE: BaseSkillComponent, IStaticVariableResetter (source line 33); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseSkillComponent (line 33), CharacterSkillComponent (line 9);... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [GroupSkillCdInfo.cs](../src/managed/cs-e3fc67839851/skills/GroupSkillCdInfo.cs) | `G_GROUPSKILLCDINFO` | skills | 7 / 0 | 0/7/0 | OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 126); OBSERVED_SINGLETON_CONTEXT: Log (source line 68); OBSERVED_MEMBER_OR_HELPER_REFERENCE: EventSystem (line 126), SkillCd... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [PassiveSkillCdInfo.cs](../src/managed/cs-e3fc67839851/skills/PassiveSkillCdInfo.cs) | `G_PASSIVESKILLCDINFO` | skills | 4 / 0 | 0/4/0 | OBSERVED_SINGLETON_CONTEXT: Log (source line 76); OBSERVED_SINGLETON_CONTEXT: Time (source line 33); UNKNOWN_GAME_CONTEXT: unselected members, callers, and external runtime effe... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Skill.cs](../src/managed/cs-e3fc67839851/skills/Skill.cs) | `G_SKILL` | skills | 4 / 1 | 4/1/0 | OBSERVED_SINGLETON_CONTEXT: CharacterNameDefines (source line 128); OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 112); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseSkillCompon... | G_SKILLEFFECTS | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SkillBehaviorAction.cs](../src/managed/cs-e3fc67839851/skills/SkillBehaviorAction.cs) | `G_SKILLBEHAVIORACTION` | skills | 5 / 0 | 1/4/0 | OBSERVED_SINGLETON_CONTEXT: CombatLog (source line 101); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BlackboardController (line 147), BulletController (line 132), CapsuleComponent (lin... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SkillBehaviorMisc.cs](../src/managed/cs-e3fc67839851/skills/SkillBehaviorMisc.cs) | `G_SKILLBEHAVIORMISC` | skills | 1 / 0 | 1/0/0 | OBSERVED_BASE: IStaticVariableResetter (source line 16); UNKNOWN_GAME_CONTEXT: unselected members, callers, and external runtime effects are not exhaustively mapped; this list i... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SkillEffects.cs](../src/managed/cs-e3fc67839851/skills/SkillEffects.cs) | `G_SKILLEFFECTS` | skills | 4 / 0 | 4/0/0 | OBSERVED_SINGLETON_CONTEXT: EffectSystem (source line 62); OBSERVED_MEMBER_OR_HELPER_REFERENCE: CombatMessageModel (line 137), EffectHandle (line 53), EffectSystem (line 62), En... | G_SKILL | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TsAnimNotifyStateCounterAttack.cs](../src/managed/cs-e3fc67839851/skills/TsAnimNotifyStateCounterAttack.cs) | `G_TSANIMNOTIFYSTATECOUNTERATTACK` | skills | 2 / 1 | 3/0/0 | OBSERVED_BASE: TsAnimNotifyStateBase, IUnrealUObject, IUnrealObject (source line 24); OBSERVED_SINGLETON_CONTEXT: Log (source line 36); OBSERVED_SINGLETON_CONTEXT: ResourceSyste... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TsTaskSelectSkill.cs](../src/managed/cs-e3fc67839851/skills/TsTaskSelectSkill.cs) | `G_TSTASKSELECTSKILL` | skills | 2 / 0 | 2/0/0 | OBSERVED_BASE: TsTaskAbortImmediatelyBase, IUnrealUObject, IUnrealObject (source line 30); OBSERVED_SINGLETON_CONTEXT: Log (source line 60); OBSERVED_MEMBER_OR_HELPER_REFERENCE:... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TsTaskUseSkill.cs](../src/managed/cs-e3fc67839851/skills/TsTaskUseSkill.cs) | `G_TSTASKUSESKILL` | skills | 2 / 1 | 2/1/0 | OBSERVED_BASE: TsTaskAbortImmediatelyBase, IUnrealUObject, IUnrealObject (source line 30); OBSERVED_SINGLETON_CONTEXT: Log (source line 51); OBSERVED_MEMBER_OR_HELPER_REFERENCE:... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [StateMachineCommon.Switch.cs](../src/managed/cs-e3fc67839851/state/StateMachineCommon.Switch.cs) | `G_STATEMACHINECOMMON_SWITCH` | state | 1 / 0 | 1/0/0 | OBSERVED_SINGLETON_CONTEXT: Log (source line 35); OBSERVED_MEMBER_OR_HELPER_REFERENCE: GetState (line 40), OnSwitchState (line 61); UNKNOWN_GAME_CONTEXT: unselected members, cal... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [BaseLockOnComponent.cs](../src/managed/cs-e3fc67839851/targeting/BaseLockOnComponent.cs) | `G_BASELOCKONCOMPONENT` | targeting | 1 / 0 | 1/0/0 | OBSERVED_BASE: EntityComponent (source line 48); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 97); OBSERVED_SINGLETON_CONTEXT: Time (source line 91); OBSERVED_MEMBER_OR_... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CharacterLockOnComponent.cs](../src/managed/cs-e3fc67839851/targeting/CharacterLockOnComponent.cs) | `G_CHARACTERLOCKONCOMPONENT` | targeting | 2 / 0 | 2/0/0 | OBSERVED_BASE: BaseLockOnComponent, IComponentDependency (source line 32); OBSERVED_SINGLETON_CONTEXT: MathUtils (source line 47); OBSERVED_SINGLETON_CONTEXT: Time (source line ... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [LockOnInfo.cs](../src/managed/cs-e3fc67839851/targeting/LockOnInfo.cs) | `G_LOCKONINFO` | targeting | 1 / 0 | 1/0/0 | OBSERVED_MEMBER_OR_HELPER_REFERENCE: EntityHandle (line 21); UNKNOWN_GAME_CONTEXT: unselected members, callers, and external runtime effects are not exhaustively mapped; this li... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [LockOnUtils.cs](../src/managed/cs-e3fc67839851/targeting/LockOnUtils.cs) | `G_LOCKONUTILS` | targeting | 1 / 0 | 1/0/0 | OBSERVED_BASE: IStaticVariableResetter (source line 25); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 48), CreatureDataComponent (line 39), EntityHandle (line 28)... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CooperationController.cs](../src/managed/cs-e3fc67839851/teams/CooperationController.cs) | `G_COOPERATIONCONTROLLER` | teams | 2 / 0 | 1/1/0 | OBSERVED_BASE: ControllerBase<CooperationController> (source line 23); OBSERVED_SINGLETON_CONTEXT: Log (source line 30); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (l... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [CooperationModel.cs](../src/managed/cs-e3fc67839851/teams/CooperationModel.cs) | `G_COOPERATIONMODEL` | teams | 1 / 0 | 0/1/0 | OBSERVED_BASE: ModelBase<CooperationModel> (source line 17); OBSERVED_MEMBER_OR_HELPER_REFERENCE: ICooperationHandler (line 20), LinkCooperationHandler (line 27), QteCooperation... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [QteCooperationHandler.cs](../src/managed/cs-e3fc67839851/teams/QteCooperationHandler.cs) | `G_QTECOOPERATIONHANDLER` | teams | 1 / 0 | 1/0/0 | OBSERVED_BASE: ICooperationHandler (source line 20); OBSERVED_SINGLETON_CONTEXT: Log (source line 27); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 37), EntityHan... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RoleQteComponent.cs](../src/managed/cs-e3fc67839851/teams/RoleQteComponent.cs) | `G_ROLEQTECOMPONENT` | teams | 10 / 0 | 7/3/0 | OBSERVED_BASE: EntityComponent (source line 28); OBSERVED_SINGLETON_CONTEXT: CombatNet (source line 257); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 65); OBSERVED_MEMB... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [RoleTeamComponent.cs](../src/managed/cs-e3fc67839851/teams/RoleTeamComponent.cs) | `G_ROLETEAMCOMPONENT` | teams | 16 / 3 | 9/10/0 | OBSERVED_BASE: EntityComponent (source line 31); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 217); OBSERVED_SINGLETON_CONTEXT: Log (source line 96); OBSERVED_MEMBER_OR_... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SceneTeamController.cs](../src/managed/cs-e3fc67839851/teams/SceneTeamController.cs) | `G_SCENETEAMCONTROLLER` | teams | 2 / 1 | 0/3/0 | OBSERVED_BASE: ControllerBase<SceneTeamController> (source line 30); OBSERVED_SINGLETON_CONTEXT: CombatNet (source line 117); OBSERVED_SINGLETON_CONTEXT: Log (source line 72); O... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SceneTeamCooperationHandler.cs](../src/managed/cs-e3fc67839851/teams/SceneTeamCooperationHandler.cs) | `G_SCENETEAMCOOPERATIONHANDLER` | teams | 1 / 0 | 1/0/0 | OBSERVED_BASE: ICooperationHandler (source line 18); OBSERVED_SINGLETON_CONTEXT: Log (source line 36); OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseTagComponent (line 33), CheckGetC... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SceneTeamItem.cs](../src/managed/cs-e3fc67839851/teams/SceneTeamItem.cs) | `G_SCENETEAMITEM` | teams | 1 / 0 | 0/1/0 | OBSERVED_MEMBER_OR_HELPER_REFERENCE: BaseDeathComponent (line 26), EntityHandle (line 19), GetComponent (line 26); UNKNOWN_GAME_CONTEXT: unselected members, callers, and externa... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [SceneTeamModel.cs](../src/managed/cs-e3fc67839851/teams/SceneTeamModel.cs) | `G_SCENETEAMMODEL` | teams | 3 / 0 | 1/2/0 | OBSERVED_BASE: ModelBase<SceneTeamModel> (source line 25); OBSERVED_SINGLETON_CONTEXT: EventSystem (source line 44); OBSERVED_SINGLETON_CONTEXT: Log (source line 51); OBSERVED_M... | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Core.cs](../src/managed/cs-e3fc67839851/tick/Core.cs) | `P_CORE` | tick | 8 / 2 | 0/10/0 | Singleton; Time; TickSystem; Net; EntitySystem; TimerSystem; ConfigStatementRegister; Stat | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Ticker.cs](../src/managed/cs-e3fc67839851/tick/Ticker.cs) | `P_TICKER` | tick | 1 / 0 | 0/1/0 | Stat; ETickingGroup | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [TickSystem.cs](../src/managed/cs-e3fc67839851/tick/TickSystem.cs) | `P_TICKSYSTEM` | tick | 7 / 1 | 0/8/0 | Ticker; Time; UKuroTickManager; delegates; PerfSight | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Time.cs](../src/managed/cs-e3fc67839851/tick/Time.cs) | `P_TIME` | tick | 15 / 0 | 0/15/0 | KuroTime; NetInfo; Singleton | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |
| [Timer.cs](../src/managed/cs-e3fc67839851/timer/Timer.cs) | `PTIMER` | timer | 2 / 0 | 2/0/0 | TTimerAction, TimerHandle, EState, Stat, Log; native clock and cadence unresolved | historical baseline | metadata/imports.v5.json; metadata/performance-source.v1.json |
| [TimerSystemInstance.cs](../src/managed/cs-e3fc67839851/timer/TimerSystemInstance.cs) | `PTIMER_INSTANCE` | timer | 4 / 0 | 4/0/0 | Timer, PriorityQueue<Timer>, FinalizationRegistry, StatWeakMap, ConcurrentQueue, Log; clock units and lifetime unresolved | historical baseline | metadata/imports.v5.json; metadata/performance-source.v1.json |
| [TimerSystemInstance.PendingEventBoundaries.cs](../src/managed/cs-e3fc67839851/timer/TimerSystemInstance.PendingEventBoundaries.cs) | `P_PENDINGEVENTBOUNDARIES` | timer | 2 / 0 | 2/0/0 | TimerSystemInstance; TimerHandle; Timers; Get; DoRemove; EventSystem comments | none | metadata/curation/bulk-curate01.json; metadata/curation-batches.jsonl |

## Preserved outside active import

- `H`: historical read-only source view, excluded because its two identities overlap active E18/E17; not copied. Its supporting bytes are not in this repository and were not independently reproduced here. **H is not an active key duplicate.**
- `PERF_RANGECOMPONENT_PENDINGEVENTS`: historical read-only source view, excluded because two lifecycle excerpts overlap the complete LifecycleRecovery bodies `0x06030B43`/`0x06030BA4`; four unique methods `0x06030B3D`/`0x06030B84`/`0x06030B87`/`0x06030BA8` remain preserved in the recovery backlog. Its 93-candidate qualification manifest is not included or independently reproduced here.

Those four backlog identities were subsequently represented by `IMP-08-RANGE-PENDING-DRAIN`
in the [supplement below](#supplementary-recovered-source-closure-01): three ordinary
bodies and one embedded callback. The earlier backlog note is historical.

## Scope notes

- The table totals 484 ordinary selected identities, 48 associated identities, and the one abstract TaskBase.OnRun signature. Abstract and associated identities do not add ordinary implementation bodies; explicit Queue.Enumerator members remain ordinary nested members. These categories do not establish buildability or binary equivalence.
- ActiveBuffInternal preserves 13 expression-bodied source-context properties (lines 79-91) but they are not selected MethodDefs and are not new bodies. The selected identity count is 9.
- The six exact Performance gap bodies retain accepted CIL15/CIL16/CIL12/PERF11 proof; inferred source mappings remain `SOURCE_ONLY`/`INFERRED_CONDITIONAL` with raw labels retained.
- The old 12-file C#12/syntax observations are historical baseline claims only, not proof that the new 104-file tree is buildable.


<!-- BEGIN RE-RECOVERED-SOURCE-CLOSURE01 SUPPLEMENT -->
## Supplementary recovered source closure 01

This historical scoped-closure supplement is not a
replacement for the original 104-file / 533-identity baseline above and not a
whole-program final, semantic PASS, build claim, or publication approval. Typed
navigation is `metadata/source-closure.v1.json`; the census report is
`metadata/source-closure.coverage.v1.json` (report only, not self-proving
authority). Full commands, limits, graph join, and the 34-row
disposition accounting are not reproduced in this index.

Eight qualified source files are added:

| Import | Destination | Bytes | SHA-256 | Class |
| --- | --- | ---: | --- | --- |
| `IMP-01-DATALAYER-TS` | `src/scripts/reconstructed/transition/DataLayerTransition.ts` | 28648 | `fd3474339f07d49dac4a1cc3f3aa3f78d53e1528a460190464126e24aa39342c` | `IMPLEMENTATION_BEARING_PARTIAL_VIEW` |
| `IMP-02-EVENT-JS` | `src/scripts/reconstructed/event/Event.js` | 10409 | `a5873f3fb774d92c2e98f8dfa5f4a3fb9666719ab6b4723c685594bf3062e921` | `RECONSTRUCTED_JS_VIEW` |
| `IMP-03-EVENT-BRIDGE-JS` | `src/scripts/reconstructed/event/EventCSharpBridge.js` | 3173 | `4887cc3d17c94281742282275ddb89587df85b2ea1a124da7e4095193821be54` | `RECONSTRUCTED_JS_VIEW` |
| `IMP-04-CORECLR-NATIVE-THREAD` | `src/models/native/CoreclrNativeThread.cpp` | 1641 | `16048310ef7e6539e2b20b20f14f7aad7267e3de7c46c077b2bad6cd651450bb` | `SOURCE_LIKE_MODEL` |
| `IMP-05-CORECLR-THREAD-ORIGIN` | `src/models/native/CoreclrThreadOrigin.cpp` | 17301 | `51ade140ac0d2910e5185c9c209b5f01e107bd1daf55e24c8c1c070f1bfa4ce7` | `SOURCE_LIKE_MODEL` |
| `IMP-06-LIBXELL-HELPER-TARGET` | `src/models/native/LibxellHelperTarget.cpp` | 3335 | `67962955c2a27846264bd7042fccb0ef5aabc80fd9fe9f3082864ce63a3e772c` | `SOURCE_LIKE_MODEL` |
| `IMP-07-LIBXELL-SLEEP-IMPLEMENTATION` | `src/models/native/LibxellSleepImplementation.cpp` | 4262 | `9000bd4fb062dfc10d149089987db032df8153cb31274189e133da899ae51114` | `SOURCE_LIKE_MODEL` |
| `IMP-08-RANGE-PENDING-DRAIN` | `src/managed/cs-e3fc67839851/entity/RangeComponent.PendingEventDrain.cs` | 9839 | `2efdad631c62af834db0d2d3f17e540da4e11803f7daba50ec9ac329eda83fe8` | `RECONSTRUCTED_VIEW` (3 bodies + 1 embedded callback) |

The R15 alternate bridge view, R27 CharacterHit, and R28 RoleElement files are
`HISTORICAL_ALTERNATIVE`, not primary imports. The PERF03B checker is
`INTERNAL_ONLY`. The nine headers and twelve config schema files are
`DECLARATION_ONLY`. The four C++ files remain `CURATED_MODEL` /
`SOURCE_LIKE_MODEL`, not validated native implementations. TS/JS/CPP imports
have no managed MethodDef.

The C# source-like view is mechanically selected from the PERF11 projection
(input lines 1-176 and 250-251; omitted overlap 177-249), not a byte import.
Its four new qualified keys are `0x06030B3D`, `0x06030B84`, `0x06030B87`, and
the associated generated callback `0x06030BA8`. The old 104/533 authority and
key hash remain pinned; the candidate supplemented total is 105/537. No v7
ledger is created. The three ordinary method proofs and callback proof are
scoped to the accepted PERF11 evidence; their pins are not new authentication of
an original binary or a semantic revalidation.

Production trust is authorized for exact manifest SHA-256
`42f70e404a13bfbe5545ba81b946630d60a5ec15f2218d004abedff6a95e1b1d`;
`--manifest` cannot override the compiled pin. Receipts report
`PRODUCTION_PIN_AUTHORIZED` with `semantic_status: NOT_CHECKED`. The approval is
scoped to this source-like candidate and is not a whole-program final.

Checks: `source-status` structural; `source-verify --research-root ...` byte,
proof, deterministic-selection, family, old-key-pin, and bounded proof-range
checks; and `source-inventory --research-root ...` one streaming graph join.
The seven non-CS imports carry 60 finite typed function selectors. The accepted
533 classes remain 246 `DIRECT_CIL_FULL` / 280 `SOURCE_ONLY` / 6 `UNKNOWN` /
1 `METADATA_ONLY`; graph declaration axes are separate and not semantic PASS.
Any required surveyed source missing/size-changed, new research file, or
unexpected curated file now fails nonzero. Semantic status remains `NOT_CHECKED`;
runtime, native execution, CIL extraction, binary inspection, builds, and
publication were outside that checkpoint. Its checks are not new execution or
independent review of the publication copy.
<!-- END RE-RECOVERED-SOURCE-CLOSURE01 SUPPLEMENT -->


<!-- BEGIN RE-MULTILANG-SRC01 INTEGRATION -->
## Multilang source integration 01: historical scoped candidate

This appendix records the historical scoped multilang candidate, not a final, semantic PASS,
build claim, or publication approval. The prior candidate manifest and prior
review matrix remain preserved; this D01 candidate replaces only the current
working labels and adds typed predecessor-derived fields.

Predecessor authority remains unchanged:
[metadata/source-closure.v1.json](../metadata/source-closure.v1.json),
198579 bytes, SHA-256
`42f70e404a13bfbe5545ba81b946630d60a5ec15f2218d004abedff6a95e1b1d`.
The D01 successor is
[metadata/source-closure.multilang.v1.json](../metadata/source-closure.multilang.v1.json),
112697 bytes,
SHA-256 `3e6a4ae50c0443b5ed20156cc0e7e4cb317bb13bcaf8328bb96e23927d6bbf8b`, status `PRIMARY_APPROVED_SCOPED_MULTILANG_CANDIDATE`,
`not_final: true`. Expected curated set remains **131** (112 predecessor + 19
new). The compiled successor pin is `3e6a4ae50c0443b5ed20156cc0e7e4cb317bb13bcaf8328bb96e23927d6bbf8b`; production source commands are activated on the frozen candidate bytes.

### Normalized origin classes (finite)

`RECOVERED_IMPLEMENTATION`, `RECOVERED_SOURCE_PROJECTION`, `SHIPPED_SCRIPT`,
`INDEPENDENT_RESEARCH_MODEL`, `DECLARATION_OR_INTERFACE`,
`HISTORICAL_ALTERNATIVE`, `METADATA_ONLY`, `UNRESOLVED`.

No row claims `SHIPPED_SCRIPT` or `RECOVERED_IMPLEMENTATION`. The exact old
label is retained separately as `predecessor_origin_class` and checked against
the predecessor.

### Final dispositions (finite)

`IMPORTED_SOURCE_PROJECTION`, `IMPORTED_PRIMARY_SOURCE`,
`DECLARATION_ONLY_NOT_IMPORTED`, `REFERENCE_ONLY`,
`PRESERVED_HISTORICAL_ALTERNATIVE`.

`HISTORICAL_ALTERNATIVE` is an origin class, not a final disposition.
FEventSystem and KuroKVStore retain normalized origin class `METADATA_ONLY`, but
their final disposition is `REFERENCE_ONLY` because they contain only comments
and `#pragma once`.

### Physical implementation / projection / declaration files (23)

The seven existing imports remain at their original paths and were not
re-copied. The fourteen new non-historical copies were exact bytes at import,
before the later header-only privacy derivatives described above. The 12 config
declaration files are listed under the included
[config declaration directory](../src/scripts/typescript/declarations/config/).

| Origin | Predecessor label | Normalized class | Final disposition | Actual destination | Bytes | SHA-256 (prefix) | Import |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| `SRC-HPP-EVIDENCE-LABELS` | `RECONSTRUCTED` | `INDEPENDENT_RESEARCH_MODEL` | `IMPORTED_PRIMARY_SOURCE` | [evidence_labels.hpp](../src/native/include/research/evidence_labels.hpp) | 437 | `d15408e960b6` | new exact |
| `SRC-CPP-CORECLR-NATIVE-THREAD` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [CoreclrNativeThread.cpp](../src/models/native/CoreclrNativeThread.cpp) | 1641 | `16048310ef7e` | existing |
| `SRC-CPP-CORECLR-THREAD-ORIGIN` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [CoreclrThreadOrigin.cpp](../src/models/native/CoreclrThreadOrigin.cpp) | 17301 | `51ade140ac0d` | existing |
| `SRC-CPP-LIBXELL-HELPER-TARGET` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [LibxellHelperTarget.cpp](../src/models/native/LibxellHelperTarget.cpp) | 3335 | `67962955c2a2` | existing |
| `SRC-CPP-LIBXELL-SLEEP-IMPLEMENTATION` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [LibxellSleepImplementation.cpp](../src/models/native/LibxellSleepImplementation.cpp) | 4262 | `9000bd4fb062` | existing |
| `SRC-HPP-STREAMING-COMPLETION-BRIDGE` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [StreamingCompletionBridge.hpp](../src/projections/native/include/StreamingCompletionBridge.hpp) | 8541 | `9ef853036faf` | new exact |
| `SRC-HPP-WORLD-PARTITION-BRIDGE` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [WorldPartitionBridge.hpp](../src/projections/native/include/WorldPartitionBridge.hpp) | 8756 | `6b10a10f1c0a` | new exact |
| `SRC-JS-EVENT-CORE` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [Event.js](../src/scripts/reconstructed/event/Event.js) | 10409 | `a5873f3fb774` | existing |
| `SRC-JS-EVENT-CSHARP-BRIDGE` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [EventCSharpBridge.js](../src/scripts/reconstructed/event/EventCSharpBridge.js) | 3173 | `4887cc3d17c9` | existing |
| `SRC-TS-CONFIG-BOSSPILINGBUFF` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [BossPilingBuff.ts](../src/scripts/typescript/declarations/config/BossPilingBuff.ts) | 1342 | `c3ae3a7d2e28` | new exact |
| `SRC-TS-CONFIG-BOSSRUSHBUFF` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [BossRushBuff.ts](../src/scripts/typescript/declarations/config/BossRushBuff.ts) | 1695 | `fb7bed56fca9` | new exact |
| `SRC-TS-CONFIG-COMBOTEACHING` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [ComboTeaching.ts](../src/scripts/typescript/declarations/config/ComboTeaching.ts) | 1724 | `68005cb9f6b5` | new exact |
| `SRC-TS-CONFIG-FLAGCHALLENGEROLEBUFF` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [FlagChallengeRoleBuff.ts](../src/scripts/typescript/declarations/config/FlagChallengeRoleBuff.ts) | 1535 | `71644f37fee6` | new exact |
| `SRC-TS-CONFIG-FLOWTEXT` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [FlowText.ts](../src/scripts/typescript/declarations/config/FlowText.ts) | 1318 | `291b1f2b5bc1` | new exact |
| `SRC-TS-CONFIG-INTERACTDATA` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [InteractData.ts](../src/scripts/typescript/declarations/config/InteractData.ts) | 1577 | `783a9fe13705` | new exact |
| `SRC-TS-CONFIG-KUROTATOGENERALGROWTH` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [KurotatoGeneralGrowth.ts](../src/scripts/typescript/declarations/config/KurotatoGeneralGrowth.ts) | 1194 | `249ee20398c6` | new exact |
| `SRC-TS-CONFIG-PAYSHOPTAB` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [PayShopTab.ts](../src/scripts/typescript/declarations/config/PayShopTab.ts) | 1247 | `414c7980b34b` | new exact |
| `SRC-TS-CONFIG-ROVERROGUEROOM` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [RoverRogueRoom.ts](../src/scripts/typescript/declarations/config/RoverRogueRoom.ts) | 1467 | `9c4383dd8902` | new exact |
| `SRC-TS-CONFIG-SKILLTREE` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [SkillTree.ts](../src/scripts/typescript/declarations/config/SkillTree.ts) | 2207 | `1bd9a0e77db6` | new exact |
| `SRC-TS-CONFIG-SORTRULE` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [SortRule.ts](../src/scripts/typescript/declarations/config/SortRule.ts) | 1246 | `75d04322bea1` | new exact |
| `SRC-TS-CONFIG-TETRISMAP` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `IMPORTED_SOURCE_PROJECTION` | [TetrisMap.ts](../src/scripts/typescript/declarations/config/TetrisMap.ts) | 1613 | `dc80e80843e5` | new exact |
| `SRC-TS-DATALAYER-TRANSITION` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [DataLayerTransition.ts](../src/scripts/reconstructed/transition/DataLayerTransition.ts) | 28648 | `fd3474339f07` | existing |
| `SRC-TS-STREAMING-COMPLETION-TRANSITION` | `RECONSTRUCTED` | `RECOVERED_SOURCE_PROJECTION` | `IMPORTED_SOURCE_PROJECTION` | [StreamingCompletionTransition.ts](../src/projections/typescript/StreamingCompletionTransition.ts) | 17036 | `fb9fafd01635` | new exact |

### Preserved historical alternatives (3, inactive)

These historical imports have normalized origin class `HISTORICAL_ALTERNATIVE` and
final disposition `PRESERVED_HISTORICAL_ALTERNATIVE`, with `active=false`.

| Origin | Predecessor label | Normalized class | Final disposition | Actual destination | Bytes | SHA-256 (prefix) | Import |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| `SRC-JS-CHARACTER-HIT-R27` | `HISTORICAL` | `HISTORICAL_ALTERNATIVE` | `PRESERVED_HISTORICAL_ALTERNATIVE` | [CharacterHitComponent.historical.reconstructed.js](../src/scripts/javascript/historical/gameplay-r27/CharacterHitComponent.historical.reconstructed.js) | 8184 | `743ea45f3673` | new exact |
| `SRC-JS-EVENT-CSHARP-BRIDGE-R15` | `RECONSTRUCTED` | `HISTORICAL_ALTERNATIVE` | `PRESERVED_HISTORICAL_ALTERNATIVE` | [EventCSharpBridge.js](../src/scripts/javascript/historical/bridge-r15/EventCSharpBridge.js) | 3129 | `67443871c698` | new exact |
| `SRC-JS-ROLE-ELEMENT-R28` | `HISTORICAL` | `HISTORICAL_ALTERNATIVE` | `PRESERVED_HISTORICAL_ALTERNATIVE` | [RoleElementComponent.historical.reconstructed.js](../src/scripts/javascript/historical/gameplay-r28/RoleElementComponent.historical.reconstructed.js) | 3939 | `9ac8c91588af` | new exact |

### Reviewed but not imported (8)

This is the historical curated-import disposition. Seven reference files are
now also included outside that curated set in the [qualified historical source map](../examples/mechanics/gameplay-showcase/README.md#qualified-historical-source-map).
Their inclusion does not turn declaration shells into implementations.

No physical import or import mode is allowed for these rows. The two
comment-only headers are final `REFERENCE_ONLY`; the four one-forward-declaration
headers are final `DECLARATION_ONLY_NOT_IMPORTED`; LoadingPipeline and the
internal checker remain references.

| Origin | Predecessor label | Normalized class | Final disposition | Reason |
| --- | --- | --- | --- | --- |
| `SRC-HPP-FEVENTSYSTEM` | `RECONSTRUCTED` | `METADATA_ONLY` | `REFERENCE_ONLY` | Only comments and #pragma once; no actual declarations or bodies; not imported. |
| `SRC-HPP-KUROKVSTORE` | `RECONSTRUCTED` | `METADATA_ONLY` | `REFERENCE_ONLY` | Only comments and #pragma once; no actual declarations or bodies; not imported. |
| `SRC-HPP-RESOURCE-INTERFACES` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `DECLARATION_ONLY_NOT_IMPORTED` | One forward declaration plus metadata; placeholder only, not a useful API; not imported. |
| `SRC-HPP-STREAMING-INTERFACES` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `DECLARATION_ONLY_NOT_IMPORTED` | One forward declaration plus metadata; placeholder only, not a useful API; not imported. |
| `SRC-HPP-TRANSITION-INTERFACES` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `DECLARATION_ONLY_NOT_IMPORTED` | One forward declaration plus metadata; placeholder only, not a useful API; not imported. |
| `SRC-HPP-WORLD-PARTITION-INTERFACES` | `RECONSTRUCTED` | `DECLARATION_OR_INTERFACE` | `DECLARATION_ONLY_NOT_IMPORTED` | One forward declaration plus metadata; placeholder only, not a useful API; not imported. |
| `SRC-JS-CHECK-PERF03B-SEMANTICS` | `INTERNAL` | `INDEPENDENT_RESEARCH_MODEL` | `REFERENCE_ONLY` | Internal author checker; not game implementation or copied dependency; not imported. |
| `SRC-TS-LOADING-PIPELINE` | `RECONSTRUCTED` | `UNRESOLVED` | `REFERENCE_ONLY` | Empty load skeleton and comments only; UNKNOWN ABI; retained as reference, not imported. |

### Typed predecessor fields and exact route policy

Every row carries typed `authority`, exact parent `identity_scheme`, and exact
parent `evidence_references` (the accepted parent proof anchors). Code compares
these semantically to the predecessor; it does not re-hash evidence files in
the successor or accept self-asserted authority. The exact 34-origin route table
in code binds each origin to its reviewed destination, import mode, category,
representation, normalized class, final disposition, and tier. Counts alone
cannot authorize a balanced substitution.

### Derived counts

- Origins: **34**; physical: **26** (19 new + 7 existing).
- Final: **1 primary**, **22 imported projections**, **3 preserved historical**,
  **4 declaration-only not imported**, **4 reference-only**.
- Not imported: **8** (`4 declaration-only + 4 reference-only`).
- Normalized classes: `{DECLARATION_OR_INTERFACE: 18, HISTORICAL_ALTERNATIVE: 3, INDEPENDENT_RESEARCH_MODEL: 2, METADATA_ONLY: 2, RECOVERED_SOURCE_PROJECTION: 8, UNRESOLVED: 1}`.
- D01 records: the included [implementation checkpoint](../tools/re/local-recovery/multilang-src01/d01/reports/IMPLEMENTATION_CHECKPOINT_D01.json) and [review matrix](../tools/re/local-recovery/multilang-src01/d01/reports/SOURCE_REVIEW_MATRIX_D01.json) record historical scoped candidate evidence; neither is a current review or final approval.
- D01 route policy: the accepted historical route policy is not included or independently reproduced here.
- Exact per-file authority/identity/evidence equality was recorded in an accepted
  typed-field proof that is not included or independently reproduced here.

<!-- END RE-MULTILANG-SRC01 INTEGRATION -->


## Indexed access workflow (RE-CORPUS-ACCESS01)

The navigation index above is still the accepted 104/533 authority. For
repeatable whole-graph lookup without rescanning the 764197857-byte
`methods.jsonl` per key, build the separate derivative:

When separately authorized, set `$env:RESEARCH_ROOT` to the required research
corpus. It is not included in this repository.

```powershell
cargo run --manifest-path tools/re/Cargo.toml --bin corpus-registry -- `
  source-access-index --curated-root . --research-root $env:RESEARCH_ROOT `
  --manifest metadata/source-closure.v1.json
```

Output is CreateNew only and defaults to
`tools/re/local-index/source-access.methods.v1.idx`. It must be an explicit
fresh relative path under the curated root. The builder streams the accepted
graph exactly once, validates every strict-JSON row, recomputes every full-key
`method_id`, requires 305856 unique rows and the accepted graph SHA/size, then
publishes a sorted fixed-width index. Existing index files are never replaced.
The DEV index is 23,245,116 bytes; it stores no source/content/provenance
copies. The included [final source-hash report](../tools/re/local-recovery/corpus-access01/reports/final/SOURCE131_HASHES_FINAL.json) records historical identities of the 131 source files, not the index payload or later publication derivatives.

`source-locate` is the maintainer query. It accepts the accepted authority id
`MANAGED-CS-E3FC67839851` or the exact assembly SHA, and a proper `0x02`/`0x06`
eight-hex TypeDef/MethodDef pair:

```powershell
cargo run --manifest-path tools/re/Cargo.toml --bin corpus-registry -- `
  source-locate --authority MANAGED-CS-E3FC67839851 `
  --type 0x020000A3 --method 0x0600042E `
  --research-root $env:RESEARCH_ROOT --curated-root .
```

Exit/status contract:

- `0` `FOUND` or `FOUND_WITH_LIMITATIONS`: the selected row and every present
  selected location were checked. `BODY_UNKNOWN`/`BODY_ABSENT` views are
  allowed and are not promoted.
- `1` `PARTIAL_MISSING_BACKING`: at least one declared location is missing;
  other good views remain in the output. `NOT_INDEXED` is also `exit 1` for a
  well-formed full key absent from the pinned historical graph snapshot.
- `2` blocked/unsafe/invalid: bad authority, token, manifest, index hash,
  index bounds, reparse ancestry, or artifact drift. `TRUST_INDEX_PIN_UNAVAILABLE`
  applies when the required compiled index pin is absent; the historical
  production pin recorded below is not a CLI override.

Representation classes are `PRIMARY_CURATED`, `SUPPLEMENTARY`,
`CANONICAL_GENERATED`, `HISTORICAL_ALTERNATIVE`, `EVIDENCE_ONLY`, `MISSING`,
and `UNRESOLVED`. Raw/proof references never claim a source body. Canonical
source association does not prove a constructor/method body. The accepted
`UNKNOWN` (6) and abstract `METADATA_ONLY` (1) classes remain unpromoted.
Associated callbacks are `ASSOCIATED_OR_EMBEDDED`, not extra bodies.

### Reconcile the 131 curated files

`source-lookup` artifact mode resolves the exact closed set:

- 104 active receipt files;
- 8 predecessor import destinations (7 non-C# exact copies + 1 deterministic
  C# supplement);
- 26 successor physical imports (7 existing + 19 new exact copies).

The union is exactly **131** paths: `104 + 1 + 26`; the 7 existing successor
imports are already counted in the predecessor 7, not added twice. A
post-snapshot path such as
`src/scripts/typescript/declarations/config/SkillTree.ts` is accepted without
requiring membership in the frozen 41456-row snapshot:

```powershell
cargo run --manifest-path tools/re/Cargo.toml --bin corpus-registry -- `
  source-lookup --artifact-root-id CURATED_ROOT `
  --artifact-path src/scripts/typescript/declarations/config/SkillTree.ts `
  --research-root $env:RESEARCH_ROOT --curated-root .
```

Arbitrary curated paths remain `SOURCE_NOT_FOUND`. Missing paths are `1`;
drift, traversal, and reparse ancestry are blocked/`2`.

### Historical lookup examples

These describe prior checks, not a new test run or a public-input reproduction.

- Event: `0x020000A3` / `0x0600042E`: active `DIRECT_CIL_FULL`, body available.
- Calculation: `0x02002EB2` / `0x06018868` (`UNKNOWN`) and `/ 0x06018867`
  (`DIRECT_CIL_FULL`): classes remain distinct.
- Timer: `0x02000BEC` / `0x06003267`; TaskGraph `0x020034D4` / `0x0601C917`;
  TaskSystem `0x020034D8` / `0x0601C933`.
- Gameplay: `0x02002E71` / `0x06018708`: source-only accepted view plus
  supplementary graph references.
- Supplement: `0x020048E7` / `0x06030B87`, callback `0x06030BA8`: destination
  and source selectors are explicit; the callback is embedded.
- Abstract: `0x020034D3` / `0x0601C913`: `METADATA_ONLY`/body absent.
- Canonical: `0x02000022` / `0x0600009A`: `CANONICAL_GENERATED`, body unknown.
- Multiple views: `0x02000039` / `0x060000F4`: canonical plus supplementary
  performance/semantic views remain separate.
- Historical JS origins R15/R27/R28 remain reachable through
  `source-lookup --origin-id ...`; source-locate does not fabricate historical
  managed artifacts.

### Limits

The index is a derivative accelerator, not a new authority. `source-locate`
verifies the whole index hash once per process, then seeks and verifies only
the selected graph row; the other 305855 rows are explicitly not rechecked.
A missing key is absent from the indexed historical snapshot, not proof about
later bytes. Whole-file hashes do not prove method/body semantics. No
buildability, ABI, runtime behavior, publication approval, or original-source
claim is made.
### D01 correction notes

- `source-locate` retains every exact current match, not only one selected
  record. The Event query for `0x0600042E`/`0x06000434` now keeps E18/E17 as
  active primary records and H as a separate `HISTORICAL_ALTERNATIVE` source
  view. H is `HISTORICAL_ONLY`, superseded by E18/E17, and has its own record
  hash. Its supporting bytes are not included or independently reproduced here.
  Exact historical fullkeys are reported in `exact_source_views` even when the
  same record already supplies the active source.
- The 48 associated identities are classified from accepted method coverage and
  actual-vs-root TypeDef evidence: **37 generated associations + 11 associated
  declaring-type methods**. `DIRECT_CIL_FULL` remains an evidence class, but an
  associated FULL identity is `ASSOCIATED_OR_EMBEDDED`, not an independent
  ordinary body. `P_ASYNCTASK` `0x02009689/0x0604A4E5` is generated association
  with `SOURCE_ONLY` evidence and body unknown; `G_BASEDAMAGECOMPONENT`
  `0x02002E65/0x06018666` is generated association with FULL evidence and body
  embedded.
- Graph-declared ranges are emitted as `declared_source_range` / `declared_semantic_range` only, with
  `navigation_range` null unless the bytes/range are actually validated. Legacy
  incorrect-anchor paths are quarantined. Graph body hashes are
  `declared_body_hash_verified`, and P1 `DECOMPILED_BODY_PRESENT` is a declared
  snapshot fact, not a fresh semantic-identity proof. Graph-declared body
  placement stays `BODY_UNKNOWN` unless separately verified.
- Every declared representation source/semantic/raw/proof reference and row
  source/raw reference is post-emission checked and, if a branch did not emit it,
  retained through a role-specific fallback. `representation_id`, declared kind,
  and reference role remain visible in views.
- Exact accepted CURATED imports resolve from the manifest/active/successor
  refs before the frozen snapshot or 54MB link registry, but successor loading
  first byte-checks private research originals. This is not a public-only shortcut.
  Those historical joins
  are explicitly `NOT_REQUESTED` for this hot path; unknown CURATED paths still
  require the frozen registry. The four supplement selectors use
  `SOURCE_CLOSURE_ACCEPTED_SCOPED` with verifier `NOT_RECHECKED_THIS_LOOKUP`;
  they are not "new unvalidated" methods.
### Historical compiled index pin

The recorded DEV access index for the frozen
RE-CORPUS-ACCESS01 candidate: **23245116 bytes / SHA-256
`42102369cc43a9ff6c60cab75bb00d9ade8a149ce186b19338790607de81cdbe`**, with
header magic `WUWAIDX1`, version 1, graph methods SHA-256
`86441818f198af7bb220f3675098aaeb48f6f3ed6c47f2e1baa99b6aa20d4c03`, size
764197857, and 305856 rows. The compiled production pin now uses this exact
value; the CLI still has no hash override.

Production `source-locate` hashes the whole 23MB index once per process, seeks
and verifies only the selected graph row, and explicitly reports the other
305855 rows as `NOT_RECHECKED`. The final source-inventory gate performs one
fresh full graph stream and must match this frozen pin; it is not an index
rebuild. Fixture missing-pin tests use an explicit
`IndexTrust { expected_sha256: None }`, not a production-None assumption.

This is a scoped historical pin, not a new acceptance, final freeze,
semantic/body claim, or buildability/ABI/runtime proof.

### Reproducibility and private-input boundaries

See [REPRODUCIBILITY.md](REPRODUCIBILITY.md) for the per-operation status of
builds, portable CLI roots, fixture tests, mandatory private-corpus inputs, and
backup coverage. The curated source pages above remain self-contained
navigation. Supporting private evidence is not redistributed; stable identities
and limitations are retained, but unavailable bytes were not independently
reproduced here.
