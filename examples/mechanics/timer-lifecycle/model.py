"""Authored timer lifecycle model; source checks and scenarios, not a game oracle."""

import hashlib
import heapq
import json
from pathlib import Path
import stat
import sys


if not sys.dont_write_bytecode:
    raise SystemExit("Run with python -B; this model does not write bytecode.")


SOURCE_BINDINGS = (
    (
        "src/managed/cs-e3fc67839851/timer/Timer.cs",
        2779,
        "c5d437c8f4c80de3a163fbb80fc6c4968b3467cb280d8bf4c1f9d310e09ecbfa",
        (
            (
                34,
                72,
                (
                    "double next = Next;",
                    "Now = next;",
                    "Next = next +",
                    "Count++;",
                    "Handle = null;",
                    "Action!.Invoke(delta);",
                    "if (Loop != 0)",
                ),
            ),
        ),
    ),
    (
        "src/managed/cs-e3fc67839851/timer/TimerSystemInstance.cs",
        4931,
        "2607a87cd72841399c0f6159ea08a9bd2587317c57e7b92327dee1f819b12201",
        (
            (
                85,
                129,
                (
                    "queue.Pop();",
                    "if (top.State != EState.Normal)",
                    "if (top.Do())",
                    "queue.Push(top);",
                    "while (!PendingRemoveQueue.IsEmpty)",
                ),
            ),
        ),
    ),
    (
        "src/managed/cs-e3fc67839851/timer/TimerSystemInstance.cs",
        4931,
        "2607a87cd72841399c0f6159ea08a9bd2587317c57e7b92327dee1f819b12201",
        (
            (
                136,
                142,
                (
                    "Registry.Unregister(timer);",
                    "timer.State = EState.Trash;",
                    "timer.Clear();",
                    "Timers.Remove(timer.Id);",
                ),
            ),
        ),
    ),
    (
        "src/managed/cs-e3fc67839851/scheduling/PriorityQueue.cs",
        7853,
        "48d2102543b23a10077b86b6621b0f805f60c1dc8b3773b85a0baf2b46b1bb4d",
        (
            (
                84,
                90,
                (
                    "public void Push(T item)",
                    "_Heap.Add(item);",
                    "_Hash[item] = index;",
                    "Up(index);",
                ),
            ),
        ),
    ),
)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def read_plain(path):
    for entry in (path, *path.parents):
        info = entry.lstat()
        require(
            not getattr(info, "st_file_attributes", 0) & 0x400,
            f"Reparse input: {entry}",
        )
    require(stat.S_ISREG(path.stat().st_mode), f"Not a regular input: {path}")
    return path.read_bytes()


def check_source_bindings(root):
    for relative, size, expected_hash, windows in SOURCE_BINDINGS:
        path = root / relative
        data = read_plain(path)
        require(len(data) == size, f"Size mismatch: {relative}")
        digest = hashlib.sha256(data).hexdigest()
        require(digest == expected_hash, f"SHA-256 mismatch: {relative}")
        lines = data.decode("utf-8").splitlines()
        for first, last, fragments in windows:
            require(1 <= first <= last <= len(lines), f"Line range: {relative}")
            text = "\n".join(lines[first - 1 : last])
            offset = 0
            for fragment in fragments:
                found = text.find(fragment, offset)
                require(found >= 0, f"Missing ordered fragment in {relative}: {fragment}")
                offset = found + len(fragment)


class TimerModel:
    # Assumptions: one owner/thread/timer, Normal initially, positive unit
    # interval/dilation, and nonthrowing callback. No timing precision claim.
    # The omitted public Remove/Get lookup is not modeled; the callback below
    # assumes DoRemove is reached for this same timer.
    def __init__(self, loop=0):
        self.now = 0
        self.next = 1
        self.count = 0
        self.loop = loop
        self.state = "Normal"
        self.handle = object()
        self.action = lambda owner: None

    def do(self, owner):
        self.now = self.next
        self.next += 1
        self.count += 1
        self.handle = None
        self.action(owner)
        return self.loop == 0 or self.count < self.loop


class OwnerModel:
    # Authored heap and dictionary stand-ins, not recovered containers/runtime.
    def __init__(self, timer):
        self.now = 0
        self.timer = timer
        self.members = {1: timer}
        self.queue = [(timer.next, timer)]
        self.pending = []

    def remove_same(self):
        self.timer.state = "Trash"
        self.timer.action = None
        self.timer.handle = None
        self.members.pop(1, None)

    def tick(self, delta, post_callback_guard=False):
        self.now += delta
        while self.queue and self.queue[0][0] <= self.now:
            _, timer = heapq.heappop(self.queue)
            if timer.state != "Normal":
                continue
            if timer.do(self):
                # Counterfactual negative control; no such guard is shown.
                if not post_callback_guard or timer.state == "Normal":
                    heapq.heappush(self.queue, (timer.next, timer))
            else:
                self.remove_same()
        while self.pending:
            self.pending.pop()
            self.remove_same()


def run_scenarios():
    results = []

    one = OwnerModel(TimerModel(loop=1))
    one.tick(1)
    require(one.timer.count == 1 and not one.members and not one.queue, "one-shot")
    results.append({"name": "one-shot", "count": 1, "queued": False})

    repeat = OwnerModel(TimerModel())
    repeat.tick(3)
    require(repeat.timer.count == 3 and repeat.queue[0][0] == 4, "repeat catch-up")
    results.append({"name": "repeat-catch-up", "count": 3, "next": 4})

    removed = OwnerModel(TimerModel())
    removed.timer.action = lambda owner: owner.remove_same()
    removed.tick(1)
    require(removed.timer.count == 1 and not removed.members, "self-remove membership")
    require(len(removed.queue) == 1 and removed.timer.state == "Trash", "self-remove requeue")
    removed.tick(0.5)
    require(len(removed.queue) == 1 and removed.timer.count == 1, "not due yet")
    removed.tick(0.5)
    require(not removed.queue and removed.timer.count == 1, "trash due pop")
    results.append({"name": "self-remove-trash-until-due", "count": 1, "queued_after_pop": False})

    control = OwnerModel(TimerModel())
    control.timer.action = lambda owner: owner.remove_same()
    control.tick(1, post_callback_guard=True)
    require(not control.queue, "counterfactual post-callback guard")
    results.append({"name": "counterfactual-post-callback-guard", "queued": False})

    pending = OwnerModel(TimerModel())
    pending.pending.append(1)
    pending.tick(1)
    require(pending.timer.count == 1 and not pending.members, "pending removal order")
    require(len(pending.queue) == 1 and pending.timer.state == "Trash", "pending removal requeue")
    results.append({"name": "pending-remove-after-due-loop", "count": 1, "trash_queued": True})

    before_due = OwnerModel(TimerModel())
    before_due.remove_same()
    before_due.tick(1)
    require(before_due.timer.count == 0 and not before_due.queue, "pre-removed timer")
    results.append({"name": "pre-removed-skip", "count": 0, "queued": False})

    return results


def main():
    script = Path(__file__).absolute()
    root = script.parents[3]
    check_source_bindings(root)
    scenarios = run_scenarios()
    print(
        json.dumps(
            {
                "classification": "AUTHORED_CONDITIONAL_MODEL_NOT_GAME_ORACLE",
                "source_binding_files": len({binding[0] for binding in SOURCE_BINDINGS}),
                "source_bindings": "PASS",
                "scenarios": scenarios,
                "game_runtime_equivalence": "NOT_CLAIMED",
            },
            separators=(",", ":"),
        )
    )


if __name__ == "__main__":
    main()
