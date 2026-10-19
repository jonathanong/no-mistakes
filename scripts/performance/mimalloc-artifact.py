"""Verify native allocator artifact identity before resource collection."""
import hashlib
import json
from pathlib import Path
import runpy


def verify_artifact(binary, variant, source, platform_name):
    binary_path = Path(binary).resolve()
    provenance_path = binary_path.parent / "provenance.json"
    provenance = json.loads(provenance_path.read_text())
    target, lto, units = {
        "linux-arm64": ("aarch64-unknown-linux-gnu", "fat", 1),
        "linux-x64": ("x86_64-unknown-linux-gnu", "fat", 1),
        "macos-arm64": ("aarch64-apple-darwin", "thin", 16),
    }[platform_name]
    expected = {"variant": variant, "source": source, "target": target, "lto": lto,
                "codegenUnits": units}
    for key, value in expected.items():
        if provenance.get(key) != value:
            raise ValueError(f"{variant} artifact {key}={provenance.get(key)!r}, expected {value!r}")
    recorded = provenance.get("no-mistakes", {})
    digest = hashlib.sha256(binary_path.read_bytes()).hexdigest()
    if recorded.get("sha256") != digest:
        raise ValueError(f"{variant} binary hash does not match its provenance")
    probe_path = binary_path.parent / "rayon-probe"
    probe_record = provenance.get("rayon-probe", {})
    probe_digest = hashlib.sha256(probe_path.read_bytes()).hexdigest()
    if probe_record.get("sha256") != probe_digest:
        raise ValueError(f"{variant} Rayon probe hash does not match its provenance")
    addon = binary_path.parent / "no-mistakes.node"
    if provenance.get("no-mistakes.node", {}).get("sha256") != hashlib.sha256(addon.read_bytes()).hexdigest():
        raise ValueError(f"{variant} addon hash does not match its provenance")
    if platform_name.startswith("linux"):
        tls = json.loads((binary_path.parent / "addon-tls.json").read_text())
        dump = (binary_path.parent / "addon-dynamic.txt").read_bytes()
        if (tls.get("addonSha256") != provenance["no-mistakes.node"]["sha256"]
                or tls.get("dynamicDumpSha256") != hashlib.sha256(dump).hexdigest()
                or tls.get("staticTls") is not False):
            raise ValueError("missing or mismatched ELF TLS evidence")
        validator = runpy.run_path(str(Path(__file__).with_name("mimalloc-tls.py")))
        validator["validate_dynamic"](dump.decode())
    return provenance



def verify_pair(artifacts):
    system, mimalloc = (artifacts[name] for name in ("system", "mimalloc"))
    for key in ("source", "tree", "lock", "rustc", "cargo", "target", "lto", "codegenUnits", "build_flags"):
        if key not in system or system[key] in (None, "") or system[key] != mimalloc.get(key):
            raise ValueError(f"system and mimalloc have missing or different {key}")
    expected_features = {"system": "--no-default-features --features mermaid-validation", "mimalloc": "default"}
    for name, artifact in artifacts.items():
        if artifact.get("features") != expected_features[name]:
            raise ValueError(f"unexpected {name} allocator feature set")
    flags = system["build_flags"]
    expected = {"RUSTFLAGS": None, "CARGO_ENCODED_RUSTFLAGS": None, "RUSTC_WRAPPER": "",
                "CARGO_PROFILE_RELEASE_DEBUG": "0", "CARGO_INCREMENTAL": "0", "NO_MISTAKES_BUILD_NAPI": "1"}
    if set(flags) != set(expected) or any(flags[k] not in (None, "") if v is None else flags[k] != v
                                          for k, v in expected.items()):
        raise ValueError("unexpected native build flags")
    for key in ("no-mistakes", "no-mistakes.node"):
        hashes = [a.get(key, {}).get("sha256") for a in (system, mimalloc)]
        if any(not value for value in hashes) or hashes[0] == hashes[1]:
            raise ValueError(f"system and mimalloc require distinct {key} artifacts")


def verify_probe(path, artifact):
    probe = Path(path).resolve()
    if hashlib.sha256(probe.read_bytes()).hexdigest() != artifact.get("rayon-probe", {}).get("sha256"):
        raise ValueError("supplied CPU probe does not match artifact provenance")
    return probe
