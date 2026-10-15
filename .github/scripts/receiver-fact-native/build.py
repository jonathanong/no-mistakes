"""Build sequential exact historical artifacts; publish curated provenance only."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys

from common import (FEATURES, FIXTURE, FIXTURE_SHA, LOCK, PROBE, PROBE_SHA,
                    REVISIONS, TREES, cleanup_target, command, digest, identity,
                    resources, write, require, failure_record)
from pmu import capability

CURRENT_ARM = None


def prepare(root, checkout):
    global CURRENT_ARM
    root.mkdir(parents=True, exist_ok=True)
    require(not (root / 'public/preflight.json').exists(), 'experiment_output_already_exists')
    cpu = identity(root)
    require(cpu['architecture'] == 'x86_64', 'runner_architecture_not_x86_64')
    rustc = command(['rustc', '-Vv'])
    require(rustc.startswith('rustc 1.99.0 (b940084d7 '), 'historical_compiler_mismatch')
    require(digest(checkout / PROBE) == PROBE_SHA, 'probe_hash_mismatch')
    write(root / 'public/preflight.json', dict(cpu, rustc=rustc, pmu=capability(),
          historical_scope='57c927d to 8d81aa1; not current main', issue=2012))
    for label, sha in REVISIONS.items():
        CURRENT_ARM = label
        command(['git', 'fetch', '--no-tags', '--depth=1', 'origin', sha], checkout)
        source = root / label
        command(['git', 'worktree', 'add', '--detach', str(source), sha], checkout)
        require(command(['git', 'rev-parse', 'HEAD^{tree}'], source) == TREES[label], 'source_tree_mismatch')
        require(digest(source / 'Cargo.lock') == LOCK, 'cargo_lock_hash_mismatch')
        require(digest(source / FIXTURE) == FIXTURE_SHA, 'fixture_hash_mismatch')


def build(root, checkout, profile):
    global CURRENT_ARM
    resources(root, build=True)
    target = root / ('compile-' + profile)
    require(not target.is_symlink(), 'compilation_target_is_symlink')
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='1',
               RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='', RUSTUP_TOOLCHAIN='1.99.0')
    # Do not inherit caller profile overrides or compiler wrappers.
    for key in list(env):
        if key.startswith('CARGO_PROFILE_') or key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
            del env[key]
    mode = 'release' if profile == 'opt3' else 'debug'
    if profile == 'opt0':
        env['CARGO_PROFILE_DEV_DEBUG'] = '0'
    private = root / 'private'
    private.mkdir(exist_ok=True)
    manifests = []
    for label, sha in REVISIONS.items():
        CURRENT_ARM = label
        resources(root, build=True)
        source = root / label
        fingerprints = target / mode / '.fingerprint'
        for path in fingerprints.glob('no-mistakes-*'):
            require(not path.is_symlink(), 'owned_fingerprint_is_symlink')
            shutil.rmtree(path)
        args = ['cargo', 'build', '--locked', '-p', 'no-mistakes', '--lib', '--message-format=json']
        if profile == 'opt3':
            args.append('--release')
        records = command(args, source, env, private / f'{profile}-{label}-cargo.log')
        artifacts = [json.loads(line) for line in records.splitlines() if line.startswith('{')]
        artifacts = [r for r in artifacts if r.get('reason') == 'compiler-artifact'
                     and r['target']['name'] == 'no_mistakes' and 'rlib' in r['target']['kind']]
        require(len(artifacts) == 1, 'expected_one_owned_library_artifact', observed_count=len(artifacts))
        artifact = artifacts[0]
        require(artifact['fresh'] is False, 'owned_crate_was_not_recompiled')
        require(Path(artifact['target']['src_path']).resolve() ==
                (source / 'crates/no-mistakes/src/lib.rs').resolve(), 'compiler_artifact_source_path_mismatch')
        expected = {'opt_level': '3' if profile == 'opt3' else '0', 'debuginfo': 0,
                    'debug_assertions': profile == 'opt0', 'overflow_checks': profile == 'opt0', 'test': False}
        require(artifact['profile'] == expected, 'compiler_profile_mismatch')
        require(artifact['features'] == FEATURES, 'default_features_mismatch')
        library = Path(next(p for p in artifact['filenames'] if p.endswith('.rlib')))
        library_sha = digest(library)
        binary = root / f'{profile}-{label}-facts'
        require(not binary.exists(), 'frozen_binary_already_exists')
        flags = ['--edition=2021', '-C', 'opt-level=' + ('3' if profile == 'opt3' else '0')]
        if profile == 'opt3':
            flags += ['-C', 'lto=true', '-C', 'codegen-units=1']
        command(['rustc', str(checkout / PROBE), *flags, '-L', 'dependency=' + str(target / mode / 'deps'),
                 '--extern', 'no_mistakes=' + str(library), '-o', str(binary)], source, env,
                private / f'{profile}-{label}-link.log')
        require(digest(library) == library_sha, 'library_changed_during_probe_link')
        manifest = dict(label=label, revision=sha, tree=TREES[label], profile=profile,
                        cargo_profile=artifact['profile'], features=FEATURES, fresh=False,
                        crate_version='0.90.0', lock_sha256=LOCK, probe_sha256=PROBE_SHA,
                        fixture_sha256=FIXTURE_SHA, library_sha256=library_sha,
                        library_bytes=library.stat().st_size, binary_sha256=digest(binary),
                        binary_bytes=binary.stat().st_size, probe_flags=flags,
                        lto=profile == 'opt3', codegen_units=1 if profile == 'opt3' else 'dev default',
                        workspace_manifest_sha256=digest(source / 'Cargo.toml'),
                        crate_manifest_sha256=digest(source / 'crates/no-mistakes/Cargo.toml'))
        write(root / f'public/{profile}-{label}-manifest.json', manifest)
        manifests.append(manifest)
    for key in ('cargo_profile', 'features', 'lock_sha256', 'probe_flags', 'workspace_manifest_sha256',
                'crate_manifest_sha256', 'fixture_sha256', 'probe_sha256'):
        require(manifests[0][key] == manifests[1][key], 'pair_provenance_mismatch', invariant=key)
    require(manifests[0]['binary_sha256'] != manifests[1]['binary_sha256'], 'pair_binaries_are_identical')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['prepare', 'opt3', 'opt0', 'cleanup-opt3', 'cleanup-opt0'])
    parser.add_argument('--root', required=True, type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    checkout = Path(__file__).resolve().parents[3]
    try:
        if args.stage == 'prepare':
            prepare(root, checkout)
        elif args.stage.startswith('cleanup-'):
            cleanup_target(root, args.stage.removeprefix('cleanup-'))
        else:
            build(root, checkout, args.stage)
    except Exception as error:
        write(root / f'public/{args.stage}-failure.json', failure_record(
            error, args.stage, arm=CURRENT_ARM, profile=args.stage if args.stage in ('opt3', 'opt0') else None))
        print('Experiment incomplete: ' + type(error).__name__, file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
