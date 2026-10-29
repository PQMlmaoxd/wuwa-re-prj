#!/usr/bin/env python3
"""RE-DAMAGE-DISPATCH02 authored runner (stdlib only, fail-closed).

Modes:

  python -B run_dispatch.py --prepare-only
      Static preflight plus exclusive-create fixture generation inside the new
      private tree. Reads a private transformation spec, verifies the canonical
      source slices by line range, absolute offset, length and SHA-256, applies
      exactly one declared span replacement, proves the reverse transform is
      byte-exact, and writes fixtures, manifest, ledger and prepare receipt.
      This mode never invokes dotnet, a compiler, a test, MCP, Git or a network
      client.

  python -B run_dispatch.py --prepare-only --dry-run
      Same verification and planned-output report, no filesystem write.

  python -B run_dispatch.py --execute --manifest manifest.json \
      --expected-manifest-sha <64hex>
      Re-verifies spec, fixtures, public compile inputs, protected baseline and
      toolchain against the frozen manifest, then invokes the pinned csc.dll
      directly (no MSBuild / NuGet / restore / analyzers / compiler server) and
      runs the compiled harness under a scrubbed environment.

All paths are lexical: no resolve()/realpath() is used for a safety decision.
Every existing path component is checked for the Windows reparse attribute.
No recovered method body text is present in this file; it lives only in the
private transformation spec and the generated private fixtures.
"""
import argparse
import base64
import hashlib
import json
import os
import stat
import subprocess
import sys

LEAF = os.path.dirname(os.path.abspath(__file__))
MANAGED = os.path.dirname(LEAF)
TESTS = os.path.dirname(MANAGED)
GITHUB = os.path.dirname(TESTS)
ROOT = os.path.dirname(GITHUB)
PUBLIC = LEAF
PRIVATE = os.path.join(GITHUB, "tools", "re", "local-recovery", "damage-dispatch02")

DOTNET = r"C:\Program Files\dotnet\dotnet.exe"
SDK = r"C:\Program Files\dotnet\sdk\9.0.312"
CSC = os.path.join(SDK, "Roslyn", "bincore", "csc.dll")
CSC_RUNTIMECONFIG = os.path.join(SDK, "Roslyn", "bincore", "csc.runtimeconfig.json")
REFDIR = r"REQUIRED_NET8_REFERENCE_PACK_DIRECTORY"
RUNTIME_DIR = r"C:\Program Files\dotnet\shared\Microsoft.NETCore.App\8.0.11"
REF_COUNT = 163

SPEC = os.path.join(PRIVATE, "transformation-spec.json")
ABSENCE = os.path.join(PRIVATE, "absence-evidence.json")
MANIFEST = os.path.join(PRIVATE, "manifest.json")
FIX01_MANIFEST = os.path.join(PRIVATE, "manifest.fix01.json")
FIXTURES = os.path.join(PRIVATE, "fixtures")
PROVENANCE = os.path.join(PRIVATE, "provenance")
RECEIPTS = os.path.join(PRIVATE, "receipts")
BUILD = os.path.join(PRIVATE, "build")
RUN = os.path.join(PRIVATE, "run")
ENV = os.path.join(PRIVATE, "env")

FIX01_RECEIPT = os.path.join(RECEIPTS, "fix01-receipt.json")
PRIOR_BODIES_FIXTURE = os.path.join(FIXTURES, "project", "src", "Calculation.PriorAcceptedBodies.cs")

CALC_FIXTURE = os.path.join(FIXTURES, "project", "src", "Calculation.RecoveredBodies.cs")
SNAP_FIXTURE = os.path.join(FIXTURES, "project", "src", "AttributeSnapshot.RecoveredGetter.cs")
DISPATCH_FIXTURE = os.path.join(FIXTURES, "project", "src", "Calculation.DispatchTransformed.cs")
ABILITY_FIXTURE = os.path.join(FIXTURES, "project", "src", "AbilityUtils.LevelValue.cs")
RUNTIMECONFIG = os.path.join(FIXTURES, "project", "DispatchRegression.runtimeconfig.json")
DLL = os.path.join(BUILD, "DispatchRegression.dll")

AUTHORED = ("run_dispatch.py", "TestScaffolding.cs", "HarnessTests.cs")
OPTIONAL_PUBLIC = ("README.md",)
PUBLIC_COMPILE = ("TestScaffolding.cs", "HarnessTests.cs")

READ_INPUTS = (
    "src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/source/_global/Calculation.cs",
    "github_src/src/managed/cs-e3fc67839851/gameplay/Calculation.DamageFormula.cs",
    "src/reconstructed/managed/full/CSharpScript/shards/915ffeea66739cd2f9098f048312b97f6dc3410709b2d8622aca7e9dd040ef07/source/_global/AbilityUtils.cs",
    "src/reconstructed/managed/semantic/gameplay/R38_METHOD_EVIDENCE_ADDENDUM.md",
    "src/reconstructed/managed/semantic/gameplay/R53_METHOD_EVIDENCE_ADDENDUM.md",
    "src/reconstructed/managed/semantic/gameplay/R53_R46_RETAINED_EVIDENCE.txt",
    "src/reconstructed/managed/full/CSharpScript/shards/1b7ddbbeb4336b03cd8950cd24cf832978c224681bfc71dea1b9f15d2da0f8d0/evidence/raw/methods.jsonl",
    "src/reconstructed/managed/full/CSharpScript/shards/896b84053f8b7f4804e290a7beecd0b33f33ec95dc38fc380523f642b1e66aee/source/AkiClient/Game/Aki/Protocol/EAttributeType.cs",
)

PRIOR_TREES = (
    "github_src/tests/managed-recovered/damage-formula",
    "github_src/tools/re/local-recovery/damage-formula01",
)

OUTPUT_PATHS = (
    MANIFEST,
    FIXTURES,
    PROVENANCE,
    RECEIPTS,
    BUILD,
    RUN,
    ENV,
)

EXECUTE_OUTPUTS = (
    DLL,
    os.path.join(RECEIPTS, "csc.stdout.txt"),
    os.path.join(RECEIPTS, "csc.stderr.txt"),
    os.path.join(RECEIPTS, "harness.stdout.txt"),
    os.path.join(RECEIPTS, "harness.stderr.txt"),
    os.path.join(RECEIPTS, "execute-receipt.json"),
)

CALC_HEADER = (
    b"// PRIVATE SCRATCH FIXTURE -- generated by run_dispatch.py --prepare-only.\n"
    b"// Authored host wrapper only. TFormula (canonical line 9) and CalculateHeal\n"
    b"// (canonical lines 132-140) are unchanged slices. CalculateFormula is the\n"
    b"// canonical lines 231-276 with exactly one declared span changed: lines\n"
    b"// 257-260 are replaced by the explicit transformation in the private spec;\n"
    b"// lines 231-256 and 261-276 are unchanged. See transformation-spec.json.\n"
    b"using System;\n"
    b"using Aki.Config;\n"
    b"using Aki.Protocol;\n"
    b"using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities;\n"
    b"\npublic partial class Calculation\n{\n"
)

ABILITY_HEADER = (
    b"// PRIVATE SCRATCH FIXTURE -- generated by run_dispatch.py --prepare-only.\n"
    b"// Authored host wrapper only; the method body is an unchanged canonical\n"
    b"// slice of AbilityUtils.cs lines 5-20. See transformation-spec.json.\n"
    b"using Aki.Protocol;\n"
    b"\npublic class AbilityUtils\n{\n"
)

RUNTIMECONFIG_BYTES = (
    b'{\n  "runtimeOptions": {\n    "tfm": "net8.0",\n'
    b'    "framework": { "name": "Microsoft.NETCore.App", "version": "8.0.11" }\n  }\n}\n'
)

FILE_ATTRIBUTE_REPARSE_POINT = 0x0400


def die(message):
    print("REFUSING: " + message, file=sys.stderr)
    raise SystemExit(2)


def lex(path):
    return os.path.abspath(os.fspath(path))


def rel(path):
    return os.path.relpath(lex(path), lex(ROOT)).replace(os.sep, "/")


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def read_bytes(path):
    with open(lex(path), "rb") as handle:
        return handle.read()


def canon_json(obj):
    return (json.dumps(obj, indent=1, sort_keys=True) + "\n").encode("utf-8")


def is_reparse(path):
    st = os.stat(lex(path), follow_symlinks=False)
    return stat.S_ISLNK(st.st_mode) or bool(
        getattr(st, "st_file_attributes", 0) & FILE_ATTRIBUTE_REPARSE_POINT
    )


def assert_no_reparse(path):
    current = lex(path)
    drive, tail = os.path.splitdrive(current)
    parts = [part for part in tail.split(os.sep) if part]
    node = drive + os.sep if drive else os.sep
    chain = [node]
    for part in parts:
        node = os.path.join(node, part)
        chain.append(node)
    for item in chain:
        if not os.path.lexists(item):
            return
        if is_reparse(item):
            die("reparse point in path ancestry: " + item)


def walk_checked(root):
    root = lex(root)
    for dirpath, dirnames, filenames in os.walk(root, topdown=True, followlinks=False):
        if is_reparse(dirpath):
            die("reparse directory encountered: " + dirpath)
        dirnames.sort()
        filenames.sort()
        for name in dirnames:
            if is_reparse(os.path.join(dirpath, name)):
                die("reparse directory encountered: " + os.path.join(dirpath, name))
        yield dirpath, filenames


def assert_tree_no_reparse(root):
    assert_no_reparse(root)
    if not os.path.isdir(lex(root)):
        die("not a directory: " + lex(root))
    for dirpath, filenames in walk_checked(root):
        for name in filenames:
            if is_reparse(os.path.join(dirpath, name)):
                die("reparse file encountered: " + os.path.join(dirpath, name))


def assert_absent(path):
    if os.path.lexists(lex(path)):
        die("path already exists (refusing reuse/overwrite): " + lex(path))


def mkdir_new(path):
    p = lex(path)
    assert_absent(p)
    os.mkdir(p)


def write_new(path, data):
    p = lex(path)
    assert_absent(p)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_BINARY", 0)
    fd = os.open(p, flags, 0o644)
    try:
        os.write(fd, data)
    finally:
        os.close(fd)


def take(data, first, last):
    if b"\r" in data:
        die("source contains CR; LF-only slice contract violated")
    lines = data.split(b"\n")
    if lines[-1] != b"":
        die("source does not end with LF")
    if first < 1 or last < first or last > len(lines) - 1:
        die("line range out of bounds: %d-%d" % (first, last))
    chunk = lines[first - 1:last]
    payload = b"".join(line + b"\n" for line in chunk)
    offset = sum(len(line) + 1 for line in lines[: first - 1])
    if data[offset:offset + len(payload)] != payload:
        die("non-contiguous slice mismatch at lines %d-%d" % (first, last))
    return offset, payload


def tree_digest(root_rel):
    root = lex(os.path.join(ROOT, root_rel))
    assert_tree_no_reparse(root)
    entries = []
    total = 0
    for dirpath, filenames in walk_checked(root):
        for name in filenames:
            full = os.path.join(dirpath, name)
            data = read_bytes(full)
            total += len(data)
            entries.append(
                (os.path.relpath(full, root).replace(os.sep, "/"), len(data), sha256_bytes(data))
            )
    entries.sort(key=lambda e: os.path.normcase(e[0]))
    digest = hashlib.sha256()
    for relpath, size, digest_hex in entries:
        digest.update((relpath + "\0" + str(size) + "\0" + digest_hex + "\n").encode())
    return {
        "root": root_rel,
        "files": [{"path": r, "bytes": s, "sha256": h} for r, s, h in entries],
        "count": len(entries),
        "bytes": total,
        "digest": digest.hexdigest(),
    }


def require_file(path_rel, expected_bytes, expected_sha):
    path = os.path.join(ROOT, path_rel)
    assert_no_reparse(path)
    data = read_bytes(path)
    if len(data) != expected_bytes:
        die("byte count drift for %s: %d != %d" % (path_rel, len(data), expected_bytes))
    if sha256_bytes(data) != expected_sha:
        die("SHA-256 drift for " + path_rel)
    return data


def protected_baseline():
    inputs = []
    for path_rel in READ_INPUTS:
        path = os.path.join(ROOT, path_rel)
        assert_no_reparse(path)
        data = read_bytes(path)
        inputs.append({"path": path_rel, "bytes": len(data), "sha256": sha256_bytes(data)})
    trees = [tree_digest(tree_rel) for tree_rel in PRIOR_TREES]
    body = {"schema": "damage-dispatch02.protected-baseline/1", "inputs": inputs, "trees": trees}
    body["digest"] = sha256_bytes(canon_json(body))
    return body


def toolchain():
    named = {
        "dotnet": DOTNET,
        "csc": CSC,
        "csc_runtimeconfig": CSC_RUNTIMECONFIG,
        "coreclr": os.path.join(RUNTIME_DIR, "coreclr.dll"),
        "corelib": os.path.join(RUNTIME_DIR, "System.Private.CoreLib.dll"),
        "system_runtime": os.path.join(RUNTIME_DIR, "System.Runtime.dll"),
        "hostpolicy": os.path.join(RUNTIME_DIR, "hostpolicy.dll"),
    }
    files = {}
    for label, path in named.items():
        assert_no_reparse(path)
        if not os.path.isfile(path):
            die("missing toolchain file: " + path)
        data = read_bytes(path)
        files[label] = {"path": path, "bytes": len(data), "sha256": sha256_bytes(data)}

    assert_no_reparse(REFDIR)
    names = sorted(
        name for name in os.listdir(REFDIR)
        if name.lower().endswith(".dll") and os.path.isfile(os.path.join(REFDIR, name))
    )
    if len(names) != REF_COUNT:
        die("reference assembly count drift: %d != %d" % (len(names), REF_COUNT))
    refs = []
    for name in names:
        path = os.path.join(REFDIR, name)
        if is_reparse(path):
            die("reparse reference assembly: " + path)
        data = read_bytes(path)
        refs.append({"name": name, "bytes": len(data), "sha256": sha256_bytes(data)})

    body = {
        "schema": "damage-dispatch02.toolchain/1",
        "files": files,
        "refdir": {"path": REFDIR, "count": len(refs), "files": refs},
        "runtime_dir": RUNTIME_DIR,
        "sdk": SDK,
    }
    body["digest"] = sha256_bytes(canon_json(body))
    return body


def load_spec():
    assert_no_reparse(SPEC)
    data = read_bytes(SPEC)
    spec = json.loads(data.decode("utf-8"))
    if spec.get("schema") != "damage-dispatch02.transformation-spec/1":
        die("unexpected spec schema")
    return data, spec


def verify_spec(spec):
    source = require_file(spec["source"]["path"], spec["source"]["bytes"], spec["source"]["sha256"])
    imported = require_file(spec["import"]["path"], spec["import"]["bytes"], spec["import"]["sha256"])
    ability = require_file(
        spec["ability_utils"]["path"],
        spec["ability_utils"]["bytes"],
        spec["ability_utils"]["sha256"],
    )

    method = spec["method"]
    method_offset, method_body = take(source, method["first_line"], method["last_line"])
    if method_offset != method["byte_offset"]:
        die("method byte offset drift")
    if len(method_body) != method["byte_length"] or sha256_bytes(method_body) != method["sha256"]:
        die("method slice identity drift")
    if source[method_offset:method_offset + len(method_body)] != method_body:
        die("method slice is not the contiguous source bytes")

    span = spec["span"]
    if span["byte_offset"] != method_offset + span["method_relative_byte_offset"]:
        die("span absolute/method-relative offset mismatch")
    before = method_body[
        span["method_relative_byte_offset"]:
        span["method_relative_byte_offset"] + span["byte_length"]
    ]
    if len(before) != span["byte_length"] or sha256_bytes(before) != span["before_sha256"]:
        die("declared before span identity drift")
    span_offset, span_slice = take(source, span["first_line"], span["last_line"])
    if span_offset != span["byte_offset"] or span_slice != before:
        die("declared span does not match the canonical line range")

    try:
        after = base64.b64decode(span["after_text_b64"], validate=True)
    except Exception as exc:
        die("after_text_b64 is not strict base64: " + str(exc))
    if len(after) != span["after_bytes"] or sha256_bytes(after) != span["after_sha256"]:
        die("declared after block identity drift")
    if b"\r" in after or not after.endswith(b"\n"):
        die("after block must be LF-only and LF-terminated")
    after_lines = after.split(b"\n")[:-1]
    if len(after_lines) != 4 or any(not line.startswith(b"\t\t\t\t") for line in after_lines):
        die("after block must be four tab-indented lines")
    for token in (
        b'"damageId"',
        b'"formula type"',
        b'"unexpected formula type"',
        b"(ELogModule)35",
        b"(ELogAuthor)20",
    ):
        if after.count(token) != 1:
            die("payload/operand token count is not exactly one: " + repr(token))
    if after.index(b'"damageId"') > after.index(b'"formula type"'):
        die("payload order changed")
    if b"InlineArray" in after or b"global::<>" in after:
        die("compiler-generated inline-array names remain in the after block")

    prefix = method_body[: span["method_relative_byte_offset"]]
    suffix = method_body[span["method_relative_byte_offset"] + span["byte_length"]:]
    if prefix + before + suffix != method_body:
        die("prefix/before/suffix do not reconstruct the canonical method")
    if not prefix.endswith(b"\t\t\t\tLog instance = Singleton<Log>.Instance;\n"):
        die("canonical Log instance line is not preserved at the span boundary")
    if not suffix.startswith(b"\t\t\t\treturn 0.0;\n\t\t\t}\n"):
        die("canonical return 0.0 / close brace is not preserved at the span boundary")

    new_body = prefix + after + suffix
    reverse = new_body[: len(prefix)] + before + new_body[len(prefix) + len(after):]
    if reverse != method_body:
        die("reverse transform does not reproduce the canonical method exactly")
    if new_body[: len(prefix)] != prefix or new_body[len(prefix) + len(after):] != suffix:
        die("bytes outside the declared span changed")
    if len(method_body) - len(before) != len(new_body) - len(after):
        die("unexpected length change outside the declared span")
    other_diff = 0

    extras = {}
    for entry in spec["extra_slices"]:
        offset, payload = take(source, entry["first_line"], entry["last_line"])
        if offset != entry["byte_offset"] or len(payload) != entry["byte_length"]:
            die("extra slice offset/length drift: " + entry["name"])
        if sha256_bytes(payload) != entry["sha256"]:
            die("extra slice identity drift: " + entry["name"])
        extras[entry["name"]] = payload

    ability_utils = spec["ability_utils"]
    ability_offset, ability_slice = take(
        ability, ability_utils["first_line"], ability_utils["last_line"]
    )
    if ability_offset != ability_utils["byte_offset"]:
        die("AbilityUtils slice offset drift")
    if len(ability_slice) != ability_utils["byte_length"]:
        die("AbilityUtils slice length drift")
    if sha256_bytes(ability_slice) != ability_utils["slice_sha256"]:
        die("AbilityUtils slice identity drift")

    copies = {}
    for entry in spec["copies"]:
        copies[entry["role"]] = require_file(entry["path"], entry["bytes"], entry["sha256"])

    return {
        "source": source,
        "import": imported,
        "ability": ability,
        "method_body": method_body,
        "method_offset": method_offset,
        "before": before,
        "after": after,
        "prefix": prefix,
        "suffix": suffix,
        "new_body": new_body,
        "tformula": extras["TFormula"],
        "heal": extras["CalculateHeal"],
        "ability_slice": ability_slice,
        "calc_copy": copies["calculation_bodies"],
        "snap_copy": copies["attribute_getter"],
        "other_diff_bytes": other_diff,
    }


def assemble(derived):
    calc_parts = (
        CALC_HEADER,
        derived["tformula"],
        b"\n",
        derived["heal"],
        b"\n",
        derived["new_body"],
        b"}\n",
    )
    calc = b"".join(calc_parts)
    embedded_offset = len(CALC_HEADER) + len(derived["tformula"]) + 1 + len(derived["heal"]) + 1
    if calc.count(derived["new_body"]) != 1:
        die("transformed method body is not embedded exactly once")
    if calc.index(derived["new_body"]) != embedded_offset:
        die("embedded transformed body offset drift")
    if calc.count(derived["tformula"]) != 1 or calc.count(derived["heal"]) != 1:
        die("extra slice embedding is not exact")
    if b"InlineArray" in calc:
        die("compiler-generated inline-array name leaked into the fixture")

    ability = ABILITY_HEADER + derived["ability_slice"] + b"}\n"
    if ability.count(derived["ability_slice"]) != 1:
        die("AbilityUtils slice embedding is not exact")

    return {
        "calc": calc,
        "snap": derived["snap_copy"],
        "ability": ability,
        "runtimeconfig": RUNTIMECONFIG_BYTES,
        "embedded_body_offset": embedded_offset,
    }


def child_env(roll_forward):
    home = os.path.join(ENV, "home")
    userprofile = os.path.join(home, "userprofile")
    drive, tail = os.path.splitdrive(userprofile)
    env = {
        "SystemRoot": r"C:\Windows",
        "windir": r"C:\Windows",
        "SystemDrive": "C:",
        "ComSpec": r"C:\Windows\System32\cmd.exe",
        "PATH": r"C:\Windows\System32;C:\Windows;C:\Program Files\dotnet",
        "PATHEXT": ".COM;.EXE;.BAT;.CMD",
        "DOTNET_ROOT": r"C:\Program Files\dotnet",
        "DOTNET_CLI_HOME": os.path.join(home, ".dotnet"),
        "DOTNET_CLI_TELEMETRY_OPTOUT": "1",
        "DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE": "1",
        "DOTNET_CLI_DO_NOT_USE_MSBUILD_SERVER": "1",
        "DOTNET_NOLOGO": "1",
        "DOTNET_SKIP_FIRST_TIME_EXPERIENCE": "1",
        "DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK": "1",
        "DOTNET_MULTILEVEL_LOOKUP": "0",
        "DOTNET_ADD_GLOBAL_TOOLS_TO_PATH": "0",
        "DOTNET_GENERATE_ASPNET_CERTIFICATE": "false",
        "DOTNET_EnableDiagnostics": "0",
        "DOTNET_EnableEventPipe": "0",
        "COMPlus_EnableDiagnostics": "0",
        "MSBUILDDISABLENODEREUSE": "1",
        "TEMP": os.path.join(ENV, "temp"),
        "TMP": os.path.join(ENV, "temp"),
        "USERPROFILE": userprofile,
        "HOME": userprofile,
        "HOMEDRIVE": drive,
        "HOMEPATH": tail,
        "APPDATA": os.path.join(home, "appdata"),
        "LOCALAPPDATA": os.path.join(home, "localappdata"),
        "NUGET_PACKAGES": os.path.join(home, "nuget"),
        "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONHASHSEED": "0",
        "PYTHONNOUSERSITE": "1",
    }
    if roll_forward:
        env["DOTNET_ROLL_FORWARD"] = "Disable"
    return env


def execute_plan(fixtures, tc):
    refs = ["/r:" + os.path.join(REFDIR, entry["name"]) for entry in tc["refdir"]["files"]]
    sources = [
        CALC_FIXTURE,
        SNAP_FIXTURE,
        DISPATCH_FIXTURE,
        ABILITY_FIXTURE,
        os.path.join(PUBLIC, "TestScaffolding.cs"),
        os.path.join(PUBLIC, "HarnessTests.cs"),
    ]
    compiler_argv = [
        DOTNET,
        CSC,
        "/noconfig",
        "/nostdlib+",
        "/target:exe",
        "/deterministic+",
        "/checked-",
        "/optimize+",
        "/debug-",
        "/nullable:enable",
        "/langversion:latest",
        "/out:" + DLL,
    ] + refs + sources
    runtime_argv = [
        DOTNET,
        "exec",
        "--runtimeconfig",
        RUNTIMECONFIG,
        DLL,
    ]
    return {
        "compiler_argv": compiler_argv,
        "sources": sources,
        "csc_env": child_env(False),
        "runtime_argv": runtime_argv,
        "run_env": child_env(True),
        "cwd": RUN,
        "timeouts_seconds": {"csc": 300, "harness": 120},
    }


def build_manifest(spec_bytes, spec_doc, absence_bytes, derived, fixtures, baseline, tc):
    public_sources = []
    for name in AUTHORED + OPTIONAL_PUBLIC:
        path = os.path.join(PUBLIC, name)
        if os.path.isfile(path):
            data = read_bytes(path)
            public_sources.append(
                {"path": rel(path), "bytes": len(data), "sha256": sha256_bytes(data)}
            )
    fixture_records = [
        {"path": rel(CALC_FIXTURE), "bytes": len(fixtures["calc"]), "sha256": sha256_bytes(fixtures["calc"])},
        {"path": rel(SNAP_FIXTURE), "bytes": len(fixtures["snap"]), "sha256": sha256_bytes(fixtures["snap"])},
        {"path": rel(DISPATCH_FIXTURE), "bytes": len(fixtures["calc"]), "sha256": sha256_bytes(fixtures["calc"])},
        {"path": rel(ABILITY_FIXTURE), "bytes": len(fixtures["ability"]), "sha256": sha256_bytes(fixtures["ability"])},
        {"path": rel(RUNTIMECONFIG), "bytes": len(fixtures["runtimeconfig"]), "sha256": sha256_bytes(fixtures["runtimeconfig"])},
    ]
    method = spec_doc["method"]
    span = spec_doc["span"]
    return {
        "schema": "damage-dispatch02.manifest/1",
        "task": "RE-DAMAGE-DISPATCH02",
        "phase": "PHASE1_PREPARE_ONLY",
        "status": "PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED",
        "classification": "EXPLICITLY_TRANSFORMED_RECOVERED_SOURCE_FIXTURE",
        "fixture_run_state": "NEVER_RUN",
        "source_authority": spec_doc["source_authority"],
        "spec": {"path": rel(SPEC), "bytes": len(spec_bytes), "sha256": sha256_bytes(spec_bytes)},
        "absence_evidence": {"path": rel(ABSENCE), "bytes": len(absence_bytes), "sha256": sha256_bytes(absence_bytes)},
        "sources": {
            "calculation": {"path": spec_doc["source"]["path"], "bytes": spec_doc["source"]["bytes"], "sha256": spec_doc["source"]["sha256"]},
            "import": {"path": spec_doc["import"]["path"], "bytes": spec_doc["import"]["bytes"], "sha256": spec_doc["import"]["sha256"]},
            "ability_utils": {"path": spec_doc["ability_utils"]["path"], "bytes": spec_doc["ability_utils"]["bytes"], "sha256": spec_doc["ability_utils"]["sha256"]},
        },
        "public_sources": public_sources,
        "fixtures": fixture_records,
        "transformation": {
            "method": method["name"],
            "type_def": method["type_def"],
            "method_def": method["method_def"],
            "raw_method_row": method["raw_method_row"],
            "code_size": method["code_size"],
            "exception_regions": method["exception_regions"],
            "canonical_first_line": method["first_line"],
            "canonical_last_line": method["last_line"],
            "canonical_byte_offset": method["byte_offset"],
            "canonical_byte_length": method["byte_length"],
            "canonical_sha256": method["sha256"],
            "span_first_line": span["first_line"],
            "span_last_line": span["last_line"],
            "span_method_relative_byte_offset": span["method_relative_byte_offset"],
            "span_absolute_byte_offset": span["byte_offset"],
            "span_byte_length": span["byte_length"],
            "span_before_sha256": span["before_sha256"],
            "span_after_bytes": span["after_bytes"],
            "span_after_sha256": span["after_sha256"],
        },
        "execute_plan": execute_plan(fixtures, tc),
        "protected_baseline_digest": baseline["digest"],
        "toolchain_digest": tc["digest"],
    }


def verify_manifest(manifest):
    if sha256_bytes(read_bytes(SPEC)) != manifest["spec"]["sha256"]:
        die("spec drift against manifest")
    if sha256_bytes(read_bytes(ABSENCE)) != manifest["absence_evidence"]["sha256"]:
        die("absence evidence drift against manifest")
    spec = json.loads(read_bytes(SPEC).decode("utf-8"))
    derived = verify_spec(spec)
    fixtures = assemble(derived)
    for entry in manifest["fixtures"]:
        full = os.path.join(ROOT, entry["path"])
        assert_no_reparse(full)
        data = read_bytes(full)
        if len(data) != entry["bytes"] or sha256_bytes(data) != entry["sha256"]:
            die("fixture drift against manifest: " + entry["path"])
    canonical = {
        rel(DISPATCH_FIXTURE): fixtures["calc"],
        rel(SNAP_FIXTURE): fixtures["snap"],
        rel(ABILITY_FIXTURE): fixtures["ability"],
        rel(RUNTIMECONFIG): fixtures["runtimeconfig"],
    }
    recorded = {entry["path"]: entry for entry in manifest["fixtures"]}
    for path, data in canonical.items():
        if path not in recorded:
            die("manifest is missing a required fixture: " + path)
        if recorded[path]["bytes"] != len(data) or recorded[path]["sha256"] != sha256_bytes(data):
            die("derived fixture drift against manifest: " + path)
    for entry in manifest["public_sources"]:
        data = read_bytes(os.path.join(ROOT, entry["path"]))
        if len(data) != entry["bytes"] or sha256_bytes(data) != entry["sha256"]:
            die("public source drift against manifest: " + entry["path"])
    if protected_baseline()["digest"] != manifest["protected_baseline_digest"]:
        die("protected baseline drift against manifest")
    if toolchain()["digest"] != manifest["toolchain_digest"]:
        die("toolchain drift against manifest")
    for name in PUBLIC_COMPILE:
        path = os.path.join(PUBLIC, name)
        if not os.path.isfile(path):
            die("missing public compile input: " + name)
    for source in manifest["execute_plan"]["sources"]:
        assert_no_reparse(source)
        if not os.path.isfile(source):
            die("missing manifest compile source: " + source)
    return derived, fixtures


def public_inventory():
    assert_no_reparse(PUBLIC)
    entries = sorted(os.listdir(PUBLIC))
    files = []
    for name in entries:
        path = os.path.join(PUBLIC, name)
        if not os.path.isfile(path):
            die("unexpected non-file entry in public root: " + path)
        if is_reparse(path):
            die("reparse public file: " + path)
        files.append(name)
    allowed = set(AUTHORED) | set(OPTIONAL_PUBLIC)
    extra = [name for name in files if name not in allowed]
    if extra:
        die("unexpected public files: " + ", ".join(extra))
    for name in AUTHORED:
        if name not in files:
            die("missing authored public file: " + name)
    return files


def prepare(write):
    if os.path.basename(LEAF) != "damage-dispatch-transformed":
        die("unexpected public root")
    if os.path.basename(GITHUB) != "github_src":
        die("unexpected workspace root")
    assert_no_reparse(PUBLIC)
    assert_no_reparse(PRIVATE)
    public_inventory()

    spec_bytes, spec = load_spec()
    absence_bytes = read_bytes(ABSENCE)
    derived = verify_spec(spec)
    fixtures = assemble(derived)
    tc = toolchain()
    baseline = protected_baseline()

    planned = [
        rel(os.path.join(PROVENANCE, "protected-baseline-pre.json")),
        rel(os.path.join(PROVENANCE, "toolchain.json")),
        rel(CALC_FIXTURE),
        rel(SNAP_FIXTURE),
        rel(DISPATCH_FIXTURE),
        rel(ABILITY_FIXTURE),
        rel(RUNTIMECONFIG),
        rel(MANIFEST),
        rel(os.path.join(PROVENANCE, "transformation-ledger.json")),
        rel(os.path.join(RECEIPTS, "prepare-receipt.json")),
    ]
    checks = {
        "source_prefix_unchanged": derived["new_body"][: len(derived["prefix"])] == derived["prefix"],
        "source_suffix_unchanged": derived["new_body"][len(derived["prefix"]) + len(derived["after"]):] == derived["suffix"],
        "reverse_transform_exact": True,
        "other_diff_bytes": derived["other_diff_bytes"],
        "fixture_embedded_body_exact": fixtures["calc"].count(derived["new_body"]) == 1,
        "original_sha256_preserved": sha256_bytes(derived["source"]) == spec["source"]["sha256"],
        "import_sha256_preserved": sha256_bytes(derived["import"]) == spec["import"]["sha256"],
        "ability_sha256_preserved": sha256_bytes(derived["ability"]) == spec["ability_utils"]["sha256"],
        "prior_tree_count": [tree["count"] for tree in baseline["trees"]],
    }
    if not all(
        checks[key]
        for key in (
            "source_prefix_unchanged",
            "source_suffix_unchanged",
            "reverse_transform_exact",
            "fixture_embedded_body_exact",
            "original_sha256_preserved",
            "import_sha256_preserved",
            "ability_sha256_preserved",
        )
    ):
        die("static transform checks did not all hold")

    print("MODE=" + ("PREPARE" if write else "DRY_RUN"))
    print("SOURCE_SHA256=" + spec["source"]["sha256"])
    print("IMPORT_SHA256=" + spec["import"]["sha256"])
    print("METHOD_SHA256=" + spec["method"]["sha256"])
    print("SPAN_BEFORE_SHA256=" + spec["span"]["before_sha256"])
    print("SPAN_AFTER_SHA256=" + spec["span"]["after_sha256"])
    print("SPAN_ABSOLUTE_BYTES=%d..%d" % (spec["span"]["byte_offset"], spec["span"]["byte_offset"] + spec["span"]["byte_length"]))
    print("METHOD_RELATIVE_BYTES=%d..%d" % (spec["span"]["method_relative_byte_offset"], spec["span"]["method_relative_byte_offset"] + spec["span"]["byte_length"]))
    print("SPAN_BEFORE_BYTES=" + str(len(derived["before"])))
    print("SPAN_AFTER_BYTES=" + str(len(derived["after"])))
    print("OTHER_DIFF_BYTES=0")
    print("FIXTURE_DISPATCH_BYTES=" + str(len(fixtures["calc"])))
    print("FIXTURE_DISPATCH_SHA256=" + sha256_bytes(fixtures["calc"]))
    print("FIXTURE_EMBEDDED_BODY_OFFSET=" + str(fixtures["embedded_body_offset"]))
    print("PROTECTED_INPUTS=" + str(len(baseline["inputs"])))
    print("PRIOR_TREES=" + ",".join("%s:%d" % (tree["root"], tree["count"]) for tree in baseline["trees"]))
    print("TOOLCHAIN_REFCOUNT=" + str(tc["refdir"]["count"]))
    print("CLASSIFICATION=EXPLICITLY_TRANSFORMED_RECOVERED_SOURCE_FIXTURE")
    print("FIXTURE_RUN_STATE=NEVER_RUN")
    print("COMPILER_INVOKED=false")
    print("HARNESS_EXECUTED=false")
    print("PLANNED_WRITES=" + ",".join(planned))

    if not write:
        print("RESULT=DRY_RUN_PASS_NO_WRITES")
        return

    for path in OUTPUT_PATHS:
        assert_absent(path)

    for path in (
        FIXTURES,
        os.path.join(FIXTURES, "project"),
        os.path.join(FIXTURES, "project", "src"),
        PROVENANCE,
        RECEIPTS,
        BUILD,
        RUN,
        ENV,
        os.path.join(ENV, "home"),
        os.path.join(ENV, "home", ".dotnet"),
        os.path.join(ENV, "home", "appdata"),
        os.path.join(ENV, "home", "localappdata"),
        os.path.join(ENV, "home", "nuget"),
        os.path.join(ENV, "home", "userprofile"),
        os.path.join(ENV, "temp"),
    ):
        mkdir_new(path)

    baseline_bytes = canon_json(baseline)
    write_new(os.path.join(PROVENANCE, "protected-baseline-pre.json"), baseline_bytes)
    tc_bytes = canon_json(tc)
    write_new(os.path.join(PROVENANCE, "toolchain.json"), tc_bytes)

    write_new(CALC_FIXTURE, fixtures["calc"])
    write_new(SNAP_FIXTURE, fixtures["snap"])
    write_new(DISPATCH_FIXTURE, fixtures["calc"])
    write_new(ABILITY_FIXTURE, fixtures["ability"])
    write_new(RUNTIMECONFIG, fixtures["runtimeconfig"])

    manifest = build_manifest(spec_bytes, spec, absence_bytes, derived, fixtures, baseline, tc)
    manifest_bytes = canon_json(manifest)
    write_new(MANIFEST, manifest_bytes)

    post_baseline = protected_baseline()
    post_checks = {
        "protected_baseline_unchanged": post_baseline["digest"] == baseline["digest"],
        "prior_public_count": post_baseline["trees"][0]["count"],
        "prior_private_count": post_baseline["trees"][1]["count"],
        "public_inventory": public_inventory(),
        "fixture_dispatch_sha256": sha256_bytes(read_bytes(DISPATCH_FIXTURE)),
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "spec_sha256": sha256_bytes(spec_bytes),
        "absence_sha256": sha256_bytes(absence_bytes),
    }
    if not post_checks["protected_baseline_unchanged"]:
        die("protected baseline changed during prepare")

    ledger = {
        "schema": "damage-dispatch02.transformation-ledger/1",
        "task": "RE-DAMAGE-DISPATCH02",
        "status": "PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED",
        "classification": "EXPLICITLY_TRANSFORMED_RECOVERED_SOURCE_FIXTURE",
        "run_state": "NEVER_RUN",
        "source": spec["source"],
        "import": spec["import"],
        "method": spec["method"],
        "method_relative_offsets": {
            "span_start": spec["span"]["method_relative_byte_offset"],
            "span_end_exclusive": spec["span"]["method_relative_byte_offset"] + spec["span"]["byte_length"],
            "prefix_bytes": len(derived["prefix"]),
            "suffix_bytes": len(derived["suffix"]),
        },
        "span": {
            "first_line": spec["span"]["first_line"],
            "last_line": spec["span"]["last_line"],
            "absolute_byte_offset": spec["span"]["byte_offset"],
            "absolute_byte_end_exclusive": spec["span"]["byte_offset"] + spec["span"]["byte_length"],
            "method_relative_byte_offset": spec["span"]["method_relative_byte_offset"],
            "byte_length": spec["span"]["byte_length"],
            "before_text_b64": base64.b64encode(derived["before"]).decode("ascii"),
            "before_sha256": sha256_bytes(derived["before"]),
            "after_text_b64": base64.b64encode(derived["after"]).decode("ascii"),
            "after_sha256": sha256_bytes(derived["after"]),
            "after_bytes": len(derived["after"]),
            "reason": spec["span"]["reason"],
            "preserved": spec["span"]["preserved"],
            "altered": spec["span"]["altered"],
            "lost_or_unestablished": spec["span"]["lost_or_unestablished"],
            "claims": spec["span"]["claims"],
        },
        "fixture": {
            "calculation_bodies": {"path": rel(CALC_FIXTURE), "bytes": len(fixtures["calc"]), "sha256": sha256_bytes(fixtures["calc"])},
            "attribute_getter": {"path": rel(SNAP_FIXTURE), "bytes": len(fixtures["snap"]), "sha256": sha256_bytes(fixtures["snap"])},
            "transformed_dispatch": {"path": rel(DISPATCH_FIXTURE), "bytes": len(fixtures["calc"]), "sha256": sha256_bytes(fixtures["calc"])},
            "ability_utils": {"path": rel(ABILITY_FIXTURE), "bytes": len(fixtures["ability"]), "sha256": sha256_bytes(fixtures["ability"])},
            "runtimeconfig": {"path": rel(RUNTIMECONFIG), "bytes": len(fixtures["runtimeconfig"]), "sha256": sha256_bytes(fixtures["runtimeconfig"])},
            "embedded_method_body_offset": fixtures["embedded_body_offset"],
            "embedded_method_body_bytes": len(derived["new_body"]),
            "embedded_method_body_sha256": sha256_bytes(derived["new_body"]),
        },
        "checks": checks,
        "independent_expected_value_derivations": [
            "T01a 16.0 = 64.0 * 0.5 * 0.5 with float 5000*0.0001f = 0.5f",
            "T01c 127.99999676644802 = (10000*9.9999997473787516e-05)*128, Ceiling 128",
            "T03 10002*float(0.0001) = 1.0002000331878662f, Ceiling 2",
            "T04 9999*float(0.0001) = 0.9998999834060669f, Ceiling 1",
            "T05a 10000.0001*9.999999747378752e-05 = 0.9999999847378749, Ceiling 1",
            "T05b bits 0x401AAAAAA4000003 = 6.666666567325595; (amount*0.10000000149011612)*1.5 = 1.0 exactly",
            "T05c 16777217 forwarded without float narrowing, Ceiling 16777217",
            "T06 3*1.0*0.5 = 1.5, Ceiling 2 then Heal sign -2",
            "T08 10002*float(0.0001) then 10000*float(0.0001)",
        ],
        "not_run": ["dotnet", "csc", "harness", "MCP", "Git", "network", "game", "CIL inspection"],
    }
    ledger_bytes = canon_json(ledger)
    write_new(os.path.join(PROVENANCE, "transformation-ledger.json"), ledger_bytes)

    receipt = {
        "schema": "damage-dispatch02.prepare-receipt/1",
        "task": "RE-DAMAGE-DISPATCH02",
        "phase": "PHASE1_PREPARE_ONLY",
        "status": "PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED",
        "compiler_invoked": False,
        "harness_executed": False,
        "classification": "EXPLICITLY_TRANSFORMED_RECOVERED_SOURCE_FIXTURE",
        "run_state": "NEVER_RUN",
        "spec": {"path": rel(SPEC), "bytes": len(spec_bytes), "sha256": sha256_bytes(spec_bytes)},
        "absence_evidence": {
            "path": rel(ABSENCE),
            "bytes": len(absence_bytes),
            "sha256": sha256_bytes(absence_bytes),
            "both_roots_lexists_false_at_authoring": True,
            "ancestry_reparse_free_at_authoring": True,
        },
        "manifest": {"path": rel(MANIFEST), "bytes": len(manifest_bytes), "sha256": sha256_bytes(manifest_bytes)},
        "ledger": {"path": rel(os.path.join(PROVENANCE, "transformation-ledger.json")), "bytes": len(ledger_bytes), "sha256": sha256_bytes(ledger_bytes)},
        "protected_baseline": {
            "path": rel(os.path.join(PROVENANCE, "protected-baseline-pre.json")),
            "bytes": len(baseline_bytes),
            "sha256": sha256_bytes(baseline_bytes),
            "digest": baseline["digest"],
            "inputs": len(baseline["inputs"]),
            "trees": [{"root": tree["root"], "count": tree["count"], "bytes": tree["bytes"], "digest": tree["digest"]} for tree in baseline["trees"]],
            "post_prepare_unchanged": post_checks["protected_baseline_unchanged"],
        },
        "toolchain": {
            "path": rel(os.path.join(PROVENANCE, "toolchain.json")),
            "bytes": len(tc_bytes),
            "sha256": sha256_bytes(tc_bytes),
            "digest": tc["digest"],
            "refcount": tc["refdir"]["count"],
        },
        "created": [],
        "checks": checks,
        "post_checks": post_checks,
        "expected_groups": 8,
        "planned_assertions": 50,
        "public_files_present": ["run_dispatch.py", "TestScaffolding.cs", "HarnessTests.cs"],
        "readme": "DEFERRED_UNTIL_AFTER_EXECUTION",
        "not_run": ["dotnet", "csc", "harness", "MCP", "Git", "network", "game"],
        "warnings": [
            "No C# compilation was performed; the transformed fixture and harness are text-checked only.",
            "Behavioral equivalence of the transformed log block to the canonical body is NOT_ESTABLISHED.",
            "Real Log.Error and RNG behavior are not observed; both are authored adapters for mechanism tests.",
            "The 143-element attribute arrays and enum numeric associations are authored assumptions preserved from the prior fixture.",
            "The private provider queue is configured per test; no RNG distribution or XorShift equivalence is claimed.",
            "README.md is intentionally deferred until after execution to avoid freezing a NOT_RUN result page.",
        ],
    }
    created = []
    for rel_path in (
        "provenance/protected-baseline-pre.json",
        "provenance/toolchain.json",
        "fixtures/project/src/Calculation.RecoveredBodies.cs",
        "fixtures/project/src/AttributeSnapshot.RecoveredGetter.cs",
        "fixtures/project/src/Calculation.DispatchTransformed.cs",
        "fixtures/project/src/AbilityUtils.LevelValue.cs",
        "fixtures/project/DispatchRegression.runtimeconfig.json",
        "manifest.json",
        "provenance/transformation-ledger.json",
        "receipts/prepare-receipt.json",
    ):
        full = os.path.join(PRIVATE, rel_path)
        if rel_path == "receipts/prepare-receipt.json":
            continue
        data = read_bytes(full)
        created.append({"path": rel(full), "bytes": len(data), "sha256": sha256_bytes(data)})
    receipt["created"] = created
    receipt_bytes = canon_json(receipt)
    write_new(os.path.join(RECEIPTS, "prepare-receipt.json"), receipt_bytes)

    print("PROTECTED_BASELINE_DIGEST=" + baseline["digest"])
    print("TOOLCHAIN_DIGEST=" + tc["digest"])
    print("MANIFEST_SHA256=" + sha256_bytes(manifest_bytes))
    print("RECEIPT_SHA256=" + sha256_bytes(receipt_bytes))
    print("RESULT=PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED")


def repair01(write):
    if not os.path.isfile(MANIFEST):
        die("base manifest is absent; repair requires the prepared tree")
    for path in (FIX01_MANIFEST, FIX01_RECEIPT, PRIOR_BODIES_FIXTURE):
        assert_absent(path)
    base_bytes = read_bytes(MANIFEST)
    base = json.loads(base_bytes.decode("utf-8"))
    if base.get("schema") != "damage-dispatch02.manifest/1":
        die("unexpected base manifest schema")
    spec_bytes, spec = load_spec()
    derived = verify_spec(spec)
    tc = toolchain()
    baseline = protected_baseline()
    if baseline["digest"] != base["protected_baseline_digest"]:
        die("protected baseline drift before repair")
    if tc["digest"] != base["toolchain_digest"]:
        die("toolchain drift before repair")

    argv = base["execute_plan"]["compiler_argv"]
    sources = base["execute_plan"]["sources"]
    if argv.count(CALC_FIXTURE) != 1:
        die("base plan does not reference the defective fixture exactly once")
    if sources.count(CALC_FIXTURE) != 1:
        die("base source list does not reference the defective fixture exactly once")
    new_argv = [PRIOR_BODIES_FIXTURE if item == CALC_FIXTURE else item for item in argv]
    new_sources = [PRIOR_BODIES_FIXTURE if item == CALC_FIXTURE else item for item in sources]
    if len(new_sources) != 6:
        die("repaired source list must contain exactly six C# inputs")
    if sum(1 for item in new_argv if item.startswith("/r:")) != REF_COUNT:
        die("repaired argv reference count drift")

    new_fixtures = [
        {
            "path": rel(PRIOR_BODIES_FIXTURE),
            "bytes": len(derived["calc_copy"]),
            "sha256": sha256_bytes(derived["calc_copy"]),
        }
    ]
    retained = None
    for entry in base["fixtures"]:
        if entry["path"] == rel(CALC_FIXTURE):
            retained = entry
            continue
        new_fixtures.append(entry)
    if retained is None:
        die("base manifest does not record the defective fixture")

    repaired = json.loads(json.dumps(base))
    repaired["fixtures"] = new_fixtures
    repaired["execute_plan"]["compiler_argv"] = new_argv
    repaired["execute_plan"]["sources"] = new_sources
    public_now = []
    for name in AUTHORED + OPTIONAL_PUBLIC:
        path = os.path.join(PUBLIC, name)
        if os.path.isfile(path):
            data = read_bytes(path)
            public_now.append({"path": rel(path), "bytes": len(data), "sha256": sha256_bytes(data)})
    repaired["public_sources"] = public_now
    repaired["repair"] = {
        "stage": "BOUNDED_REPAIR_01",
        "supersedes": {"path": rel(MANIFEST), "bytes": len(base_bytes), "sha256": sha256_bytes(base_bytes)},
        "defect": "prepare wrote the transformed dispatch bytes to Calculation.RecoveredBodies.cs, omitted the verified 8183-byte prior accepted bodies, and compiled the same dispatch bytes twice under two paths",
        "effect_if_unrepaired": "duplicate partial-class member definitions plus missing CalculateHurt/GetElement*/GetAttrFromSnapshots/CalculateElementMatchUpRate definitions; compiler was never invoked",
        "repair": "compile the verified prior accepted bodies as Calculation.PriorAcceptedBodies.cs and remove the duplicate dispatch path from the compile set; no file was overwritten or deleted",
        "retained_not_compiled": {
            "path": retained["path"],
            "bytes": retained["bytes"],
            "sha256": retained["sha256"],
            "role": "RETAINED_DUPLICATE_DISPATCH_BYTES_NOT_COMPILED",
        },
        "superseded_public_sources": base["public_sources"],
        "current_public_sources": public_now,
        "authored_runner_edit": "run_dispatch.py gained repair01 mode, generic manifest verification and a basename-addressed --manifest; the two compiled public C# files and all fixtures are byte-unchanged",
        "compiler_invoked": False,
        "harness_executed": False,
    }
    repaired_bytes = canon_json(repaired)

    print("REPAIR01_MODE=" + ("WRITE" if write else "DRY_RUN"))
    print("REPAIR01_PRIOR_BODIES_SHA256=" + sha256_bytes(derived["calc_copy"]))
    print("REPAIR01_SOURCES=" + ",".join(os.path.basename(item) for item in new_sources))
    print("REPAIR01_REFCOUNT=" + str(REF_COUNT))
    print("REPAIR01_RETAINED=" + retained["path"])
    print("REPAIR01_MANIFEST_SHA256=" + sha256_bytes(repaired_bytes))
    if not write:
        print("RESULT=DRY_RUN_PASS_NO_WRITES")
        return

    write_new(PRIOR_BODIES_FIXTURE, derived["calc_copy"])
    write_new(FIX01_MANIFEST, repaired_bytes)

    checks = {
        "prior_bodies_bytes": len(derived["calc_copy"]),
        "prior_bodies_sha256": sha256_bytes(derived["calc_copy"]),
        "prior_bodies_match_spec_copy": sha256_bytes(derived["calc_copy"]) == spec["copies"][0]["sha256"],
        "reverse_transform_exact": True,
        "source_prefix_unchanged": derived["new_body"][: len(derived["prefix"])] == derived["prefix"],
        "source_suffix_unchanged": derived["new_body"][len(derived["prefix"]) + len(derived["after"]):] == derived["suffix"],
        "dispatch_fixture_sha256": sha256_bytes(read_bytes(DISPATCH_FIXTURE)),
        "dispatch_fixture_unchanged": sha256_bytes(read_bytes(DISPATCH_FIXTURE)) == retained["sha256"],
        "protected_baseline_unchanged": protected_baseline()["digest"] == baseline["digest"],
        "toolchain_unchanged": toolchain()["digest"] == tc["digest"],
        "compiled_source_count": len(new_sources),
        "reference_count": REF_COUNT,
        "duplicate_dispatch_paths_in_plan": new_sources.count(DISPATCH_FIXTURE),
    }
    if not (
        checks["prior_bodies_match_spec_copy"]
        and checks["source_prefix_unchanged"]
        and checks["source_suffix_unchanged"]
        and checks["dispatch_fixture_unchanged"]
        and checks["protected_baseline_unchanged"]
        and checks["toolchain_unchanged"]
    ):
        die("repair static checks did not all hold")

    receipt = {
        "schema": "damage-dispatch02.fix01-receipt/1",
        "task": "RE-DAMAGE-DISPATCH02",
        "stage": "BOUNDED_REPAIR_01",
        "status": "REPAIRED_PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED",
        "compiler_invoked": False,
        "harness_executed": False,
        "supersedes_manifest": {"path": rel(MANIFEST), "bytes": len(base_bytes), "sha256": sha256_bytes(base_bytes)},
        "repaired_manifest": {"path": rel(FIX01_MANIFEST), "bytes": len(repaired_bytes), "sha256": sha256_bytes(repaired_bytes)},
        "defect": repaired["repair"]["defect"],
        "effect_if_unrepaired": repaired["repair"]["effect_if_unrepaired"],
        "repair_action": repaired["repair"]["repair"],
        "created": [
            {"path": rel(PRIOR_BODIES_FIXTURE), "bytes": len(derived["calc_copy"]), "sha256": sha256_bytes(derived["calc_copy"])},
            {"path": rel(FIX01_MANIFEST), "bytes": len(repaired_bytes), "sha256": sha256_bytes(repaired_bytes)},
        ],
        "retained": [
            {"path": retained["path"], "bytes": retained["bytes"], "sha256": retained["sha256"], "reason": "retained duplicate dispatch bytes; not compiled"},
        ],
        "checks": checks,
        "not_run": ["dotnet", "csc", "harness", "MCP", "Git", "network", "game"],
    }
    receipt_bytes = canon_json(receipt)
    write_new(FIX01_RECEIPT, receipt_bytes)
    print("REPAIR01_MANIFEST_SHA256=" + sha256_bytes(repaired_bytes))
    print("REPAIR01_RECEIPT_SHA256=" + sha256_bytes(receipt_bytes))
    print("REPAIR01_EXPECTED_MANIFEST=manifest.fix01.json")
    print("RESULT=REPAIRED_PREPARED_NOT_COMPILED_NOT_EXECUTED_NOT_PASSED")


def execute(args):
    name = args.manifest
    if os.path.basename(name) != name or not name.endswith(".json"):
        die("--manifest must be a plain .json basename")
    manifest_path = os.path.join(PRIVATE, name)
    if not os.path.isfile(manifest_path):
        die("manifest is absent: " + name)
    manifest_bytes = read_bytes(manifest_path)
    actual_manifest_sha = sha256_bytes(manifest_bytes)
    if actual_manifest_sha != args.expected_manifest_sha:
        die("manifest SHA-256 does not match --expected-manifest-sha")
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    assert_no_reparse(manifest_path)
    assert_no_reparse(PUBLIC)
    assert_no_reparse(PRIVATE)
    assert_no_reparse(REFDIR)
    for path in EXECUTE_OUTPUTS:
        assert_absent(path)
    derived, fixtures = verify_manifest(manifest)
    plan = manifest["execute_plan"]
    if not os.path.isdir(RUN) or not os.path.isdir(BUILD):
        die("execute scratch directories are missing")
    print("EXECUTE_MANIFEST_SHA256=" + actual_manifest_sha)
    print("CSC_ARGV=" + json.dumps(plan["compiler_argv"]))
    print("CSC_CWD=" + plan["cwd"])
    print("CSC_ENV=" + json.dumps(plan["csc_env"], sort_keys=True))
    print("RUNTIME_ARGV=" + json.dumps(plan["runtime_argv"]))

    csc_result = subprocess.run(
        plan["compiler_argv"],
        cwd=plan["cwd"],
        env=plan["csc_env"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=plan["timeouts_seconds"]["csc"],
        check=False,
    )
    write_new(os.path.join(RECEIPTS, "csc.stdout.txt"), csc_result.stdout)
    write_new(os.path.join(RECEIPTS, "csc.stderr.txt"), csc_result.stderr)
    print("CSC_RC=" + str(csc_result.returncode))

    harness_rc = None
    harness_stdout = b""
    harness_stderr = b""
    if csc_result.returncode == 0:
        run_result = subprocess.run(
            plan["runtime_argv"],
            cwd=plan["cwd"],
            env=plan["run_env"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=plan["timeouts_seconds"]["harness"],
            check=False,
        )
        harness_rc = run_result.returncode
        harness_stdout = run_result.stdout
        harness_stderr = run_result.stderr
        write_new(os.path.join(RECEIPTS, "harness.stdout.txt"), harness_stdout)
        write_new(os.path.join(RECEIPTS, "harness.stderr.txt"), harness_stderr)
        print("HARNESS_RC=" + str(harness_rc))

    post = {
        "spec_sha256": sha256_bytes(read_bytes(SPEC)),
        "fixture_dispatch_sha256": sha256_bytes(read_bytes(DISPATCH_FIXTURE)),
        "protected_baseline_digest": protected_baseline()["digest"],
        "toolchain_digest": toolchain()["digest"],
        "dll_bytes": os.path.getsize(DLL) if os.path.isfile(DLL) else -1,
        "dll_sha256": sha256_bytes(read_bytes(DLL)) if os.path.isfile(DLL) else None,
    }
    receipt = {
        "schema": "damage-dispatch02.execute-receipt/1",
        "task": "RE-DAMAGE-DISPATCH02",
        "manifest_sha256": actual_manifest_sha,
        "csc_rc": csc_result.returncode,
        "csc_stdout_sha256": sha256_bytes(csc_result.stdout),
        "csc_stderr_sha256": sha256_bytes(csc_result.stderr),
        "harness_rc": harness_rc,
        "harness_stdout_sha256": sha256_bytes(harness_stdout) if harness_stdout else None,
        "harness_stderr_sha256": sha256_bytes(harness_stderr) if harness_stderr else None,
        "post": post,
        "classification": "EXPLICITLY_TRANSFORMED_RECOVERED_SOURCE_FIXTURE",
        "harness_evidence_level": "AUTHORED_MECHANISM_TEST_NOT_GAME_RUNTIME",
    }
    write_new(os.path.join(RECEIPTS, "execute-receipt.json"), canon_json(receipt))
    print("EXECUTE_RECEIPT_SHA256=" + sha256_bytes(canon_json(receipt)))
    if csc_result.returncode != 0:
        die("compiler failed; failure outputs retained")
    if harness_rc != 0:
        die("harness failed; failure outputs retained")
    print("RESULT=EXECUTED_HARNESS_PASS")


def main():
    parser = argparse.ArgumentParser(add_help=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--prepare-only", action="store_true")
    mode.add_argument("--repair01", action="store_true")
    mode.add_argument("--execute", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--manifest", default="")
    parser.add_argument("--expected-manifest-sha", default="")
    args = parser.parse_args()
    if args.prepare_only:
        prepare(write=not args.dry_run)
        return
    if args.repair01:
        repair01(write=not args.dry_run)
        return
    if not args.manifest or not args.expected_manifest_sha:
        die("--execute requires --manifest <name>.json and --expected-manifest-sha")
    execute(args)


if __name__ == "__main__":
    main()
