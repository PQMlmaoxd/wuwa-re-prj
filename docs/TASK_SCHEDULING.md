# TaskGraph Scheduling and TaskSystem Execution Source Views

Status: `RECONSTRUCTED / SELECTED METHODS`, imported from exact research bytes. Runtime is `PARKED`; publication is `DISABLED`; build is `NOT_BUILDABLE_WITH_CURRENT_CONTEXT`.

This note covers two separate, coherent source slices: the scheduling dependency, priority, and ready-queue slice (`TaskGraph`, `TaskUtils`, and `PriorityQueue<T>`), and the execution slice (`TaskSystem`, `TaskBase`, and the custom FIFO `Queue<T>`). They share accepted PERF evidence and TypeDef neighborhoods but are not joined at runtime by this note. The execution slice is not a claim that the executor is complete, that the Timer frame path is reconstructed, or that a standalone project exists.

> **Current navigation (Phase C):** active authority is `metadata/curation-batches.jsonl` plus `metadata/curation/bulk-curate01.json`, covering **104 files / 533 identities** (246 FULL / 280 SOURCE / 6 historical UNKNOWN / 1 abstract METADATA_ONLY). `AsyncTask.cs` and `Completion.cs` are now imported Performance source files; they are not a TaskGraph runtime executor or a persistence guarantee. The v5 13-file/102-key paragraphs below are historical baseline context, not the current active count. Start from [SOURCE_INDEX.md](SOURCE_INDEX.md).

## Import closure

The current active ledger is the append-only `metadata/curation-batches.jsonl` index; the v5 receipt below is the immutable historical baseline, not the current 104-file campaign count. It is `metadata/imports.v5.json`, 61237 bytes, SHA-256 `7c990cc73ea4050f7b20b00b7c8de2aef3f51bd179c516f12e0102b4bc5d11fd`, with 13 exact imported files and 102 distinct qualified method keys. Historical v1-v4 receipts remain immutable and are not combined with v5 counts.

The thirteen-file v5 closure has 45 distinct `DIRECT_CIL_FULL` keys, 50 `SOURCE_ONLY` keys, 6 historical `UNKNOWN` keys, and 1 `METADATA_ONLY` abstract signature. The execution three contribute 51 keys: 23 `DIRECT_CIL_FULL`, 27 `SOURCE_ONLY`, and 1 `METADATA_ONLY`. This note's two mapping tables contain 84 identity rows: 33 graph plus 51 execution, including the one abstract signature.

### Scheduling import closure

Three exact source files:

| Destination | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/managed/cs-e3fc67839851/scheduling/TaskGraph.cs` | 10715 | `2a6b17d8e10c287437b450d8fa87f717c008fa6571fcd3bd26c61eb603b5373a` |
| `src/managed/cs-e3fc67839851/scheduling/TaskUtils.cs` | 9108 | `f156234df333c606e4d3ea92445f39954f7beb9ed1797f44f55f70751edfc4fd` |
| `src/managed/cs-e3fc67839851/scheduling/PriorityQueue.cs` | 7853 | `48d2102543b23a10077b86b6621b0f805f60c1dc8b3773b85a0baf2b46b1bb4d` |

Typed record: `metadata/scheduling-source.v1.json`, 51985 bytes, SHA-256 `2d6ab3fbff603761c82dfcadeb2c32e37e82d1e85523de21e12bf157edab7c54`.

The scheduling three contribute 33 keys: 12 `DIRECT_CIL_FULL` and 21 `SOURCE_ONLY`. Historical v4 is 10 files / 51 keys; it is predecessor evidence, not the current count.

### Execution import closure

Three exact source files, imported under `RE-SOURCE03`:

| Destination | Bytes | SHA-256 | Selected keys |
| --- | ---: | --- | ---: |
| `src/managed/cs-e3fc67839851/scheduling/TaskSystem.cs` | 15075 | `0567e1ec4cff3885fc5add61ff4288fd1f19e69894445630f7dc0f1afce792d9` | 18 |
| `src/managed/cs-e3fc67839851/scheduling/TaskBase.cs` | 5418 | `50dd7ff2779ce004601ee862ffbf5b578c0dc46823df61f68f8d00fbf76771c3` | 14 |
| `src/managed/cs-e3fc67839851/scheduling/Queue.cs` | 10163 | `439ce33738812997ad56772218f108eb82873f34f105f6ed677f4b9cb6b4ad3a` | 19 |

Typed record: `metadata/task-execution-source.v1.json`, 96117 bytes, SHA-256 `46e312246a5d5d870da293bb95c500e2c03e8d4960ed6cfb29d952a193bd509e`. The record-level statuses are `UNKNOWN` evidence, `SOURCE_ONLY` source, and `MIXED` representation for all three records. The per-file counts are `TaskSystem` 13 FULL + 5 SOURCE, `TaskBase` 6 FULL + 7 SOURCE + 1 abstract META, and `Queue` 4 FULL + 15 SOURCE. The 51 keys are 23 FULL + 27 SOURCE + 1 META.

All six scheduling/execution source files are non-standalone; the execution files are not a whole `TaskSystem`/`TaskBase`/`Queue` class recovery and are not compile-ready.


## Authority and associated types

Authority is managed `MANAGED-CS-E3FC67839851`, assembly SHA-256 `e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db`, MVID `2d6d37cb-76e5-4075-8220-3eeca41471e3`. Namespace is observed global. The record root TypeDefs remain `0x020034D4` (Graph), `0x020034D9` (Utils), and `0x0200006B` (Heap).

Graph uses a catalog-scoped associated-type map because methods span proven nested declaring types:

| Declaring TypeDef | Parent TypeDef | Association evidence |
| --- | --- | --- |
| `0x0200968E` | `0x020034D4` | raw main `types.jsonl` row 196; source `TaskGraph.cs` lines 117-179 |
| `0x0200968F` | `0x020034D4` | raw main `types.jsonl` row 197; source `TaskGraph.cs` lines 95-231 |
| `0x0200CE0E` | `0x0200968E` | raw main `types.jsonl` row 296; source `TaskGraph.cs` lines 117-179 |

The historically false `compiler_generated` flag on `0x0200CE0E` is not used as association evidence. The hierarchy is the evidence. Every method below retains its actual declaring TypeDef; the Graph root TypeDef is never forced onto an associated method.

### Execution authority and associated types

Execution roots are `0x020034D8` (`TaskSystem`), `0x020034D3` (`TaskBase`), and `0x02000071` (`Queue<T>`). Their source and destination identities are listed in the execution import closure. Execution associated types are:

| Declaring TypeDef | Parent TypeDef | Meaning | Association evidence |
| --- | --- | --- | --- |
| `0x02009691` | `0x020034D8` | `TaskSystem` display class | `types.jsonl` row 219; `TaskSystem.cs` lines 225-243 |
| `0x02009692` | `0x020034D8` | `TaskSystem.Run` state machine | `types.jsonl` row 220; `TaskSystem.cs` lines 66-101 |
| `0x02009693` | `0x020034D8` | `TaskSystem.RunInternal` state machine | `types.jsonl` row 221; `TaskSystem.cs` lines 245-400 |
| `0x02009694` | `0x020034D8` | `TaskSystem.YieldFrame` state machine | `types.jsonl` row 222; `TaskSystem.cs` lines 225-243 |
| `0x0200968D` | `0x020034D3` | `TaskBase.Run` state machine | `types.jsonl` row 241; `TaskBase.cs` lines 93-135 |
| `0x02007184` | `0x02000071` | nested `Queue<T>.Enumerator` | `types.jsonl` row 96; `Queue.cs` lines 272-314 |

The `TaskSystem` and `TaskBase` associated types carry five compiler-lowered/display method identities (`0x0604A4F7`, `0x0604A4F8`, `0x0604A4FA`, `0x0604A4FC`, `0x0604A4ED`). The nested Queue type carries six `Enumerator` identities. The 11 associated identities are not independent top-level original C# declarations. Root TypeDefs are never forced onto associated methods.


## Method mapping

`FULL` means a selected complete persisted CIL body and `DIRECT_CIL_FULL`. `SOURCE` means selected source-level representation with `SOURCE_ONLY` and no complete persisted CIL body for that method. Raw locator keys are defined after the table.

| Token | Method | Root record | Declaring TypeDef | Status | Source anchor | Evidence locator |
| --- | --- | --- | --- | --- | --- | --- |
| `0x0601C916` | `.cctor` | PSCHED_GRAPH | `0x020034D4` | SOURCE | `TaskGraph.cs:20-25` | main-methods 909 |
| `0x0601C917` | `Compare` | PSCHED_GRAPH | `0x020034D4` | FULL | `TaskGraph.cs:27-35` | critical 9-38 |
| `0x0601C918` | `.ctor` | PSCHED_GRAPH | `0x020034D4` | SOURCE | `TaskGraph.cs:37-93` | main-methods 911 |
| `0x0601C919` | `Run` | PSCHED_GRAPH | `0x020034D4` | SOURCE | `TaskGraph.cs:95-231` | main-methods 912 |
| `0x0601C91A` | `CreateStaticDefaultValue` | PSCHED_GRAPH | `0x020034D4` | SOURCE | `TaskGraph.cs:233-237` | main-methods 913 |
| `0x0601C91B` | `ResetStaticDefaultValue` | PSCHED_GRAPH | `0x020034D4` | SOURCE | `TaskGraph.cs:239-245` | main-methods 914 |
| `0x0604A4F0` | `ScheduleTask` local wrapper | PSCHED_GRAPH | `0x0200968E` | SOURCE | `TaskGraph.cs:117-179` | main-methods 2319 |
| `0x0604A4F1` | `TrySchedulingTasks` local function | PSCHED_GRAPH | `0x0200968E` | FULL | `TaskGraph.cs:181-227` | critical 606-726 |
| `0x0604A4F2` | success continuation | PSCHED_GRAPH | `0x0200968E` | SOURCE | `TaskGraph.cs:203-210` | main-methods 2321 |
| `0x0604A4F3` | error continuation | PSCHED_GRAPH | `0x0200968E` | FULL | `TaskGraph.cs:203-223` | critical 727-777 |
| `0x0604A4F4` | Run state-machine `MoveNext` | PSCHED_GRAPH | `0x0200968F` | FULL | `TaskGraph.cs:95-231` | critical 778-1002 |
| `0x060506E6` | ScheduleTask state-machine `MoveNext` | PSCHED_GRAPH | `0x0200CE0E` | FULL | `TaskGraph.cs:117-179` | critical 1003-1307 |
| `0x0601C940` | `GetNodesWithNoDependencies` | PSCHED_UTILS | `0x020034D9` | SOURCE | `TaskUtils.cs:10-25` | helper-methods 1131 |
| `0x0601C941` | `GetNewDependsOnMap` | PSCHED_UTILS | `0x020034D9` | SOURCE | `TaskUtils.cs:27-39` | helper-methods 1132 |
| `0x0601C942` | `TopologicalSort` | PSCHED_UTILS | `0x020034D9` | FULL | `TaskUtils.cs:41-67` | critical 382-481 |
| `0x0601C943` | `GetNodeCumulativePriorities` | PSCHED_UTILS | `0x020034D9` | FULL | `TaskUtils.cs:69-103` | critical 482-602 |
| `0x0601C944` | `SearchForCycleDFS` | PSCHED_UTILS | `0x020034D9` | SOURCE | `TaskUtils.cs:105-195` | helper-methods 1135 |
| `0x0601C945` | `GraphHasCycles` | PSCHED_UTILS | `0x020034D9` | SOURCE | `TaskUtils.cs:197-225` | helper-methods 1136 |
| `0x0600026B` | `.ctor` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:44-49` | heap-methods 66 |
| `0x0600026C` | `Clone` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:51-65` | heap-methods 67 |
| `0x0600026D` | `get_Size` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:16-18` | heap-methods 68 |
| `0x0600026E` | `get_Empty` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:20-22` | heap-methods 69 |
| `0x0600026F` | `get_Top` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:24-42` | heap-methods 70 |
| `0x06000270` | `Clear` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:67-73` | heap-methods 71 |
| `0x06000271` | `Has` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:75-80` | heap-methods 72 |
| `0x06000272` | `Push` | PSCHED_HEAP | `0x0200006B` | FULL | `PriorityQueue.cs:82-90` | critical 42-77 |
| `0x06000273` | `Pop` | PSCHED_HEAP | `0x0200006B` | FULL | `PriorityQueue.cs:92-118` | critical 78-168 |
| `0x06000274` | `Remove` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:120-148` | heap-methods 75 |
| `0x06000275` | `Update` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:150-170` | heap-methods 76 |
| `0x06000276` | `Heapify` | PSCHED_HEAP | `0x0200006B` | SOURCE | `PriorityQueue.cs:172-180` | heap-methods 77 |
| `0x06000277` | `Up` | PSCHED_HEAP | `0x0200006B` | FULL | `PriorityQueue.cs:182-202` | critical 169-230 |
| `0x06000278` | `Down` | PSCHED_HEAP | `0x0200006B` | FULL | `PriorityQueue.cs:204-231` | critical 231-327 |
| `0x06000279` | `Swap` | PSCHED_HEAP | `0x0200006B` | FULL | `PriorityQueue.cs:233-242` | critical 328-381 |

The six generated associations (local function, continuations, and state-machine `MoveNext` methods) are represented as one source-level graph of behavior, not six independent original C# declarations. State-machine syntax is not claimed.

`0x0604A4EF` (display-class constructor), `0x0604A4F5` (Run `SetStateMachine`), and `0x060506E7` (ScheduleTask `SetStateMachine`) have no source representation and are explicitly excluded. They are not counted in 33, imported, or body-recovered. The 28-pin `identity_and_selection.json` does not list `0x0601C916`, `0x0601C91A`, `0x0601C91B`, `0x0604A4F2`, or `0x0604A4F3`; archived raw metadata and accepted reports establish their identities. Only accepted persisted evidence and archived metadata are used here.

### Execution method mapping

`FULL`, `SOURCE`, and `META` below mean the same selected-method proof levels used in the execution catalog. Source anchors identify line ranges in the imported files listed above; every row retains its actual declaring TypeDef, including associated types, and the root TypeDef is not substituted.

| Token | Signature | Root record | Actual TypeDef | Status | Source anchor | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `0x0601C931` | `UniTask<bool>? get_Promise()` | PEXEC_SYSTEM | `0x020034D8` | SOURCE | `TaskSystem.cs:34-35; get_Promise` | ts-methods row 698 |
| `0x0601C932` | `bool Initialize()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:59-64; Initialize` | exec-cil 1-18; Initialize |
| `0x0601C933` | `UniTask<bool> Run()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:66-101; Run` | exec-cil 19-55; Run |
| `0x0601C934` | `void AddTask(TaskBase task)` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:103-139; AddTask` | exec-cil 245-335; AddTask |
| `0x0601C935` | `void Clear()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:141-152; Clear` | exec-cil 336-365; Clear |
| `0x0601C936` | `void ClearInternal()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:154-172; ClearInternal` | exec-cil 366-416; ClearInternal |
| `0x0601C937` | `int get_HighestQueuedInterruptPriority()` | PEXEC_SYSTEM | `0x020034D8` | SOURCE | `TaskSystem.cs:37-54; get_HighestQueuedInterruptPriority` | ts-methods row 704 |
| `0x0601C938` | `void IncreaseQueuedCount(ETaskPriority priority)` | PEXEC_SYSTEM | `0x020034D8` | SOURCE | `TaskSystem.cs:174-179; IncreaseQueuedCount` | ts-methods row 705 |
| `0x0601C939` | `void DecreaseQueuedCount(ETaskPriority priority)` | PEXEC_SYSTEM | `0x020034D8` | SOURCE | `TaskSystem.cs:181-186; DecreaseQueuedCount` | ts-methods row 706 |
| `0x0601C93A` | `void TryInterruptCurrent()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:188-208; TryInterruptCurrent` | exec-cil 417-489; TryInterruptCurrent |
| `0x0601C93B` | `void SwapBuffers()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:210-223; SwapBuffers` | exec-cil 490-531; SwapBuffers |
| `0x0601C93C` | `int get_QueuedSize()` | PEXEC_SYSTEM | `0x020034D8` | SOURCE | `TaskSystem.cs:56-57; get_QueuedSize` | ts-methods row 709 |
| `0x0601C93D` | `UniTask YieldFrame()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:225-243; YieldFrame` | exec-cil 532-565; YieldFrame |
| `0x0601C93E` | `UniTask<bool> RunInternal()` | PEXEC_SYSTEM | `0x020034D8` | FULL | `TaskSystem.cs:245-400; RunInternal` | exec-cil 681-718; RunInternal |
| `0x0604A4F7` | `void <YieldFrame>b__0(float _)` | PEXEC_SYSTEM | `0x02009691` | FULL | `TaskSystem.cs:225-243; <YieldFrame>b__0` | exec-cil 1135-1154; <YieldFrame>b__0 |
| `0x0604A4F8` | `void Run.MoveNext()` | PEXEC_SYSTEM | `0x02009692` | FULL | `TaskSystem.cs:66-101; MoveNext` | exec-cil 56-244; MoveNext |
| `0x0604A4FA` | `void RunInternal.MoveNext()` | PEXEC_SYSTEM | `0x02009693` | FULL | `TaskSystem.cs:245-400; MoveNext` | exec-cil 719-1134; MoveNext |
| `0x0604A4FC` | `void YieldFrame.MoveNext()` | PEXEC_SYSTEM | `0x02009694` | FULL | `TaskSystem.cs:225-243; MoveNext` | exec-cil 566-680; MoveNext |
| `0x0601C909` | `TaskBase(string name, Func<bool>? initHandle = null, Action<bool>? finishedCallback = null, TaskInterruptOptions? interruptOptions = null)` | PEXEC_BASE | `0x020034D3` | FULL | `TaskBase.cs:34-46; .ctor` | exec-cil 1155-1212; .ctor |
| `0x0601C90A` | `string get_Name()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:25-26; get_Name` | tb-methods row 962 |
| `0x0601C90B` | `void SetLogPrefix(string? logPrefix = null)` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:48-52; SetLogPrefix` | tb-methods row 963 |
| `0x0601C90C` | `ETaskPriority get_InterruptPriority()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:27-28; get_InterruptPriority` | tb-methods row 964 |
| `0x0601C90D` | `bool get_Interruptible()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:29-30; get_Interruptible` | tb-methods row 965 |
| `0x0601C90E` | `void RequestInterrupt()` | PEXEC_BASE | `0x020034D3` | FULL | `TaskBase.cs:54-68; RequestInterrupt` | exec-cil 1463-1502; RequestInterrupt |
| `0x0601C90F` | `void OnInterrupt()` | PEXEC_BASE | `0x020034D3` | FULL | `TaskBase.cs:70-80; OnInterrupt` | exec-cil 1503-1541; OnInterrupt |
| `0x0601C910` | `bool Init()` | PEXEC_BASE | `0x020034D3` | FULL | `TaskBase.cs:82-91; Init` | exec-cil 1213-1240; Init |
| `0x0601C911` | `UniTask<bool> get_Promise()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:31-32; get_Promise` | tb-methods row 969 |
| `0x0601C912` | `UniTask<bool> Run()` | PEXEC_BASE | `0x020034D3` | FULL | `TaskBase.cs:93-135; Run` | exec-cil 1241-1277; Run |
| `0x0601C913` | `abstract UniTask<bool> OnRun()` | PEXEC_BASE | `0x020034D3` | META | `TaskBase.cs:137-138; OnRun` | tb-types row 64 |
| `0x0601C914` | `bool OnInit()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:140-144; OnInit` | tb-methods row 971 |
| `0x0601C915` | `void OnExit()` | PEXEC_BASE | `0x020034D3` | SOURCE | `TaskBase.cs:146-149; OnExit` | tb-methods row 972 |
| `0x0604A4ED` | `void Run.MoveNext()` | PEXEC_BASE | `0x0200968D` | FULL | `TaskBase.cs:93-135; MoveNext` | exec-cil 1278-1462; MoveNext |
| `0x060002A3` | `Queue(int capacity = 4)` | PEXEC_QUEUE | `0x02000071` | FULL | `Queue.cs:52-57; .ctor` | exec-cil 1720-1745; .ctor |
| `0x060002A4` | `int get_Size()` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:27-28; get_Size` | q-methods row 2 |
| `0x060002A5` | `void Push(T element)` | PEXEC_QUEUE | `0x02000071` | FULL | `Queue.cs:59-77; Push` | exec-cil 1746-1832; Push |
| `0x060002A6` | `T? Pop()` | PEXEC_QUEUE | `0x02000071` | FULL | `Queue.cs:79-107; Pop` | exec-cil 1833-1944; Pop |
| `0x060002A7` | `bool TryPop(out T? result)` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:109-133; TryPop` | q-methods row 5 |
| `0x060002A8` | `T? get_Front()` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:30-47; get_Front` | q-methods row 6 |
| `0x060002A9` | `bool TryGetFront(out T? result)` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:135-146; TryGetFront` | q-methods row 7 |
| `0x060002AA` | `T? Get(int index)` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:148-175; Get` | q-methods row 8 |
| `0x060002AB` | `void SetCapacity(int capacity)` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:177-260; SetCapacity` | q-methods row 9 |
| `0x060002AC` | `bool get_Empty()` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:49-50; get_Empty` | q-methods row 10 |
| `0x060002AD` | `void Clear()` | PEXEC_QUEUE | `0x02000071` | FULL | `Queue.cs:262-270; Clear` | exec-cil 1945-1981; Clear |
| `0x060002AE` | `IEnumerator<T> GetEnumerator()` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:316-317; GetEnumerator` | q-methods row 12 |
| `0x060002AF` | `IEnumerator IEnumerable.GetEnumerator()` | PEXEC_QUEUE | `0x02000071` | SOURCE | `Queue.cs:318-319; System.Collections.IEnumerable.GetEnumerator` | q-methods row 13 |
| `0x0604670C` | `Enumerator(Queue<T> queue)` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:279-286; .ctor` | q-methods row 2166 |
| `0x0604670D` | `bool MoveNext()` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:293-308; MoveNext` | q-methods row 2167 |
| `0x0604670E` | `void Reset()` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:310-311; Reset` | q-methods row 2168 |
| `0x0604670F` | `T IEnumerator<T>.get_Current()` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:288-289; System.Collections.Generic.IEnumerator<T>.get_Current` | q-methods row 2169 |
| `0x06046710` | `object? IEnumerator.get_Current()` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:290-291; System.Collections.IEnumerator.get_Current` | q-methods row 2170 |
| `0x06046711` | `void Dispose()` | PEXEC_QUEUE | `0x02007184` | SOURCE | `Queue.cs:312-313; Dispose` | q-methods row 2171 |

The 51 execution identities are 23 FULL, 27 SOURCE, and 1 META. The abstract `TaskBase.OnRun` signature is `abstract UniTask<bool> OnRun()` and has RVA `0x0`, no IL range, and no body. The 5 compiler-lowered/display identities and 6 nested Queue `Enumerator` identities account for 11 associated identities; they are not 11 top-level declarations. The 46 persisted source-member identities are 45 executable bodies/accessors plus the one abstract signature; the 5 lowering references are not 5 extra declarations. No assertion here turns those 51 identities into 51 independent C# bodies.

## Evidence keys

- The accepted raw evidence and archived metadata below are not distributed in this repository and were not independently reproduced for this editorial note. The labels are stable evidence identities; hashes and row/range references identify historical inputs but do not make their bytes available.
- `critical`: accepted bounded-IL direct-CIL evidence, 139054 bytes, SHA-256 `dc44f6a8044d8e67927833f9783e8918ecabcd9051094ad5dfa1fd68570a48f5`; line ranges are the canonical section ranges.
- `main-methods`: accepted managed method metadata, 18957215 bytes, SHA-256 `8f57b8e7e768e02a05623f0f44a9962202764f5878f97db023af8d55bc928eb9`; row numbers above.
- `main-types`: accepted managed type metadata, 485292 bytes, SHA-256 `5d0dc02d5f95660002177781608d9f9d1d8b3f439bc28fa0d74473ded01cd5c5`; rows 44, 196, 197, and 296 establish the root and nested hierarchy.
- `helper-methods`: accepted helper method metadata, 10194713 bytes, SHA-256 `710853723d0f943cc08f5c87ab69eb0abbad6965f24927c7058762c08675daa6`; rows 1131-1136.
- `helper-types`: accepted helper type metadata, 411069 bytes, SHA-256 `337bd39b6261393c37a61137040558afaaba4e05dffda59727093f25bec33d97`; row 94.
- `heap-methods`: accepted heap method metadata, 19289824 bytes, SHA-256 `3793941c4a7b60c32c5e87bbd45bca1316cd99e72cb22f23ea2ad9579da29863`; rows 66-80.
- `heap-types`: accepted heap type metadata, 592371 bytes, SHA-256 `9179f8a8e1fd3ef3f046a6b5d3a423fa356aec2498a4138ef910038d63edffa2`; row 3.
- Historical reports: `PERF01_VERIFICATION` 7854 bytes / `f3ce66c7ddf8a66d518194ade50e4b811ec105b1447863d20e1b1c348e6f6c14`; `PERF01_THREAD_TASK_ARCHITECTURE` 4772 / `bbd33cf4cc6f724c3b63a48cb0166b8b3654ad2bea95619c5b77dce651b989de`; `PERF_GRAPH01_CUMULATIVE_PRIORITY` 68537 / `7d0e01aad3375822bb329cbe88e10c99d88bf11cbdb0efb43b835ad5e4e7974d`; `PERF_HEAP01_DOWN_RECOVERY` 48443 / `8c41d95d48af200561e8c4fe2da514ea909c79d8105f72adbdd998fe4149b5f1`.
- Supporting metadata: bounded-IL evidence, 1013673 bytes / `27da81c574be0c48af138527737a6262d7930568ab4974fd78e3adbbcf3e1c29`; identity and selection, 6253 / `31ac1c70b8ed20c1caa79a6b83084e8143488e3b7e9c86f87f6272e490f2dd92`.

The historical heap identity error that labeled Up as `0x06000272` and Down as `0x06000273` is not authoritative. Accepted Heap01 evidence and archived metadata establish Push `0x06000272`, Pop `0x06000273`, Up `0x06000277`, and Down `0x06000278`.

Execution evidence keys:

- `exec-cil`: accepted execution direct-CIL evidence, 217268 bytes, SHA-256 `d612572a59f6977f31355de266c08ccdc2b57542e2b6af025f965d4d0a680c13`; line ranges are the accepted section ranges. In the table, `exec-cil 1-18; Initialize` means lines 1-18 of that evidence record.
- `exec-evidence`: accepted execution evidence record, 50879 bytes, SHA-256 `3fe257b8d9c7f6dbeae400cfb4c408cfe08383a6bb98a241d9c62f509d30162c`.
- `exec-manifest`: accepted execution direct-CIL manifest, 15023 bytes, SHA-256 `d32dec70c88ede0c4f37ae417a2d84d2f6e7e7d8c684703f3c4e90b92e508fea`.
- `exec-report`: historical TaskSystem execution report, 31875 bytes, SHA-256 `13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27`.
- `exec-task01`: historical clear/promise lifecycle report, 46431 bytes, SHA-256 `0f278175facc1761e4b851cdeebc0612222e61252c798613557a59175a333902`.
- `ts-methods`: accepted TaskSystem method metadata, 17472586 bytes, SHA-256 `1ce8e64910539417e7af8714bd880a85883e753b07ec119ecfe996bdb8dab7ef`; rows 698-711 and 2311-2316.
- `ts-types`: accepted TaskSystem type metadata, 619458 bytes, SHA-256 `275097e0fcbe81c07845f042dc3984b4fd362f025829ab3e833f8ac9eded873e`; rows 34 and 219-222.
- `tb-methods`: accepted TaskBase method metadata, 19108311 bytes, SHA-256 `21a001ae80b5a8051143760ba7ad017d8e446087477b596827f1a6ddc49b2808`; rows 961-972 and 2344; `0x0601C913` has no method row.
- `tb-types`: accepted TaskBase type metadata, 572787 bytes, SHA-256 `90baf4f114f41872ff1393a173ab06424979d13e6789250aaf5d69c9a38158f5`; rows 64 and 241.
- `q-methods`: accepted Queue method metadata, 20874584 bytes, SHA-256 `e2388fa15ee38066b62b7795ceb638edfbf58bce0d1f61efcc9e96c8b66f36a5`; rows 1-13 and 2166-2171.
- `q-types`: accepted Queue type metadata, 513762 bytes, SHA-256 `c18916e2ad659da316f29d6d05c7ce8856bad9166a13a673c3784ab978ea04ef`; rows 1 and 96.
- `exec-catalog`: `metadata/task-execution-source.v1.json`, 96117 bytes, SHA-256 `46e312246a5d5d870da293bb95c500e2c03e8d4960ed6cfb29d952a193bd509e`.
- `v5-ledger`: `metadata/imports.v5.json`, 61237 bytes, SHA-256 `7c990cc73ea4050f7b20b00b7c8de2aef3f51bd179c516f12e0102b4bc5d11fd`.

## Behavior and dependencies

The selected graph slice builds a node/dependency map, validates endpoints and cycles, computes cumulative priority from dependents, and runs ready nodes through the indexed priority queue with a concurrency gate. `Run` and the local scheduler use source-level async/local-function shapes; generated lowering is retained only in the evidence. The slice is not a joined TaskSystem executor. Current navigation is `TaskGraph -> TaskUtils/PriorityQueue` for the accepted graph calls; it does not imply a runtime join to the execution slice.

Unresolved graph dependencies and boundaries include game `IGraphNode` and `IGraphNodeWithDependencies` context, `UniTask`, `UniTaskCompletionSource`, `CompositeError`, `OneOf`, `StackElement`, logging, callback delegates, scheduler runtime, thread placement, native budget, comparer behavior, actual heap populations, and missing/unselected members. Source-level logging projections are not evidence of original heap allocation.

### Execution behavior and boundaries

The selected execution methods preserve the observed ordering:

- `TaskSystem.RunInternal` runs `task.Run()`, then `TryInterruptCurrent()`, then awaits the task promise. This is the accepted `Run THEN TryInterrupt THEN await` order; it is not reordered into a general cancellation first.
- The initial `Run` buffer swap is not a self-feed. The `SELF_FEED_STREAK_THRESHOLD` value is 100 and applies to batches, not individual tasks.
- Normal `Run` sets the promise result before `ClearInternal` (`Promise.SetResult THEN ClearInternal`), retaining the observed clear-after-publish order.
- `TaskBase.Run` invokes `OnExit()` before `TrySetResult` in its finally path. External exceptions can escape; the `TaskBase` constructor stores `FinishedCallback` (`TaskBase.cs` line 43), and the captured `Run` lines 101-135 do not invoke it.
- Queue `Version` and capacity mutations, grow/shrink thresholds, and the nested enumerator's `Version` checks are preserved from the exact source. The queue is the custom FIFO `Queue<T>`, not `System.Collections.Generic.Queue<T>`.

Navigation within the execution slice is direct: `TaskSystem -> TaskBase/Queue`; the source-only helper members remain helper references, not fresh CIL promotions. The two notes do not claim that `TaskGraph` and `TaskSystem` are runtime-joined, that thread placement is known, or that performance/native budget is established.

`TaskSystem.YieldFrame` reaches `TimerSystem.GameplayTimeInstance.Next` as an accepted callee boundary. This does not establish a complete Timer build, a native frame executor, or a full `TimerSystem` class. The imported Timer selected methods do not fill the unselected `TimerSystem` class.

Dependencies remain per file:

- `TaskSystem`: `Singleton`/`Log`, `ETaskPriority`, `CustomPromise` from the now-imported `Completion.cs` source, `UniTask`, `TimerSystem`, interrupt options, and external task populations. The later source import does not upgrade `CustomPromise.get_Promise` beyond its recorded evidence or establish a runtime executor.
- `TaskBase`: the abstract/concrete subclass `OnRun` implementation is external; `TaskInterruptOptions`, `UniTaskCompletionSource`, callbacks, and logging remain external.
- `Queue`: `IClearable`/`UnrealEngine.Extension`, logging, generic element storage, and external populations remain external.

`CustomPromise.get_Promise` is `UNKNOWN_DIRECT_CIL` outside this batch and is not upgraded by the execution import. The selected clear methods do not prove that every individual task is settled or cancelled; local absence of such a path is not a leak or deadlock proof.


## Quality and build limits

- `EXACT_BYTES`: all six imported scheduling/execution files are byte-equal to their research sources; the three new execution files are 15075, 5418, and 10163 bytes with the hashes above.
- `ACCEPTABLE_AS_IMPORTED`: the three new execution files are acceptable as selected method views. This is not a statement that the original classes or the executor are complete.
- `INTENTIONAL_PRESENTATION`: global namespace, original spelling, artificial partial overlay, source-level async/compiler-lowered shapes, and commentary names are preserved. No fake namespace, wrapper, or original-source claim is added.
- `MISSING_CONTEXT`: constructors and fields beyond the selected views, game interfaces, UniTask, GraphNode, OneOf, StackElement, callbacks, logger, runtime, and native context remain unresolved. `NOT_BUILDABLE_WITH_CURRENT_CONTEXT`.
- `PROOF_BOUNDARY`: the execution 51 are 23 FULL, 27 SOURCE, and 1 META. The 50 new non-abstract identities are body-associated selections; with the abstract signature they total 51 identities, not 51 bodies. The abstract `OnRun` has no body. Of the 46 persisted source-member identities, 45 are executable bodies/accessors and one is the abstract signature; the 5 lowering references are not extra declarations.
- `NO_AESTHETIC_REWRITE`: tabs versus spaces, brace style, and naming taste are not grounds for source changes.

A .NET SDK 9.0.312 was observed in the current environment with `dotnet --version` native exit 0. SOURCE03 did not use the SDK. The later source gate uses the SDK Roslyn parser only for syntax diagnostics; that is not a game build. No `.sln`, C# project, stubs, model, or C# tests were added; C# tests are `NOT_RUN`, and the build state remains `NOT_BUILDABLE_WITH_CURRENT_CONTEXT`.

The earlier Calculation `D` and `T` views are disjoint even though they share TypeDef `0x02002EB2`; the three Event definitions are distinct; the six Timer methods are distinct; the six scheduling/execution source files do not compete with those views. The six frozen `UNKNOWN` Calculation methods are not promoted by this note or by historical header/map wording. The 3 Event, 2 Calculation, 2 Timer, and 6 scheduling files are the complete 13-file imported source index.

## Finite backlog

1. Rights, license, and policy review must be explicit per artifact. Publication readiness is a `BLOCKER` outside technical import status.
2. Actual dependency and build context remains unverified; do not create cosmetic projects to change the build label.
3. The bounded TaskSystem/TaskBase/Queue execution slice is now imported under `RE-SOURCE03`; no further execution import is authorized by this note.
4. Navigate and check legacy competing reports without rewriting historical controls. The accepted Heap01 and current catalog override the old Up/Down labels only for current authority.
5. No automatic follow-on campaign is proposed. Source remains the primary result; further documents are not an objective.

The 46 selected execution source members are 45 executable bodies/accessors plus one abstract signature; the remaining 5 compiler-lowered/display identities are evidence references, not five extra declarations. No TaskSystem/TaskBase/Queue build or runtime join is claimed.
