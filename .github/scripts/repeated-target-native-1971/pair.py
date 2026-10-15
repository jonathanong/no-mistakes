"""Opt-in #1971 exact current pair. Builds and layout helpers never overlap samples."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

HERE = Path(__file__).resolve().parent
# Keep this driver's modules ahead of the shared helper directory. Both
# directories contain report.py, and the current-pair test needs the local one.
sys.path.append(str(HERE.parent / 'receiver-fact-native'))
from common import command, digest, identity, require, resources, write, failure_record
from consumer_policy import validate_expectations

SPEC = json.loads((HERE / 'pair.json').read_text())
FEATURES = SPEC['features']


def current_pair(checkout):
    # Pull-request baseRefOid can retain an old merge-base even as main advances.
    # The actual branch ref is the ownership boundary for a current-main gate.
    main = json.loads(command(['gh', 'api',
        'repos/jonathanong/no-mistakes/git/ref/heads/main'], checkout))['object']['sha']
    require(main == SPEC['baseline']['revision'], 'actual_main_ref_changed',
            actual=main, expected=SPEC['baseline']['revision'])
    pr = json.loads(command(['gh', 'api', 'repos/jonathanong/no-mistakes/pulls/2022'], checkout))
    require(pr['head']['sha'] == SPEC['candidate']['pr_head'], 'pr_head_changed')
    require(pr['merge_commit_sha'] == SPEC['candidate']['revision'] and
            pr['mergeable'] is True, 'pr_synthetic_merge_changed')
    return pr, main


def analyzer_sources(source):
    files = command(['git', 'ls-files'], source).splitlines()
    selected = [p for p in files if p in ('Cargo.toml', 'Cargo.lock', 'crates/no-mistakes/Cargo.toml')
                or p.startswith('crates/no-mistakes/') and p.endswith('.rs')]
    rows = [{'path': p, 'sha256': digest(source / p)} for p in sorted(selected)]
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def prepare(root, checkout, expected_head):
    validate_expectations(SPEC.get('configured_consumer_expectations'))
    root.mkdir(parents=True, exist_ok=True)
    require(not (root / 'public/preflight.json').exists(), 'experiment_output_already_exists')
    machine = identity(root)
    require(machine['architecture'] == 'aarch64', 'runner_not_aarch64')
    cpuinfo = Path('/proc/cpuinfo').read_text().splitlines()
    machine['arm_cpu_identity'] = sorted(set(line.strip() for line in cpuinfo
        if line.startswith(('CPU implementer', 'CPU architecture', 'CPU part', 'CPU variant', 'CPU revision'))))
    rustc = command(['rustc', '-Vv'])
    require(rustc.startswith('rustc 1.99.0 (b940084d7 '), 'compiler_mismatch')
    require('host: ' + SPEC['target'] in rustc, 'compiler_host_mismatch')
    require(os.environ.get('GITHUB_REF') == 'refs/heads/' + SPEC['diagnostic_branch'], 'wrong_diagnostic_branch')
    require(command(['git', 'rev-parse', 'HEAD'], checkout) == os.environ.get('GITHUB_SHA'), 'checkout_differs_from_trigger_commit')
    require(command(['git', 'show', '-s', '--format=%P', 'HEAD'], checkout).split() == [SPEC['diagnostic_parent']], 'wrong_diagnostic_parent')
    changed = command(['git', 'diff', '--name-status', 'HEAD^', 'HEAD'], checkout).splitlines()
    allowed = ('.github/scripts/repeated-target-native-1971/',)
    require(changed and all(line.startswith('A\t') and
        (line[2:].startswith(allowed) or line[2:] == '.github/workflows/repeated-target-native-1971.yml')
        for line in changed), 'diagnostic_change_outside_new_measurement_allowlist')
    require(not command(['git', 'status', '--porcelain'], checkout), 'dirty_diagnostic_checkout')
    pr, current_main = current_pair(checkout)
    require(pr['head']['sha'] == expected_head, 'pr_head_changed')
    require(expected_head == SPEC['diagnostic_parent'], 'pr_head_differs_from_diagnostic_parent')
    ci = json.loads(command(['gh', 'api', 'repos/jonathanong/no-mistakes/actions/runs/' + str(SPEC['candidate']['verified_ci_run'])], checkout))
    require(ci['head_sha'] == expected_head and ci['conclusion'] == 'success', 'exact_ci_not_successful')
    checks = json.loads(command(['gh', 'api', 'repos/jonathanong/no-mistakes/commits/' + expected_head + '/check-runs?filter=latest&per_page=100'], checkout))['check_runs']
    required_checks = ['codecov/' + kind + '/' + language for kind in ('patch', 'project') for language in ('rust', 'javascript')]
    require(all(any(c['name'] == name and c['conclusion'] == 'success' for c in checks) for name in required_checks), 'four_codecov_checks_not_successful')
    current_merge = SPEC['candidate']['revision']
    require(pr['merge_commit_sha'] == current_merge and pr['mergeable'] is True, 'pr_synthetic_merge_changed')
    for arm in ('baseline', 'candidate'):
        revision = SPEC[arm]['revision']
        command(['git', 'fetch', '--no-tags', '--depth=2', 'origin', revision], checkout)
        source = root / arm
        command(['git', 'worktree', 'add', '--detach', str(source), revision], checkout)
        require(command(['git', 'rev-parse', 'HEAD^{tree}'], source) == SPEC[arm]['tree'], 'source_tree_mismatch')
        require(digest(source / 'Cargo.lock') == SPEC['lock_sha256'], 'lock_mismatch')
        profile = tomllib.loads((source / 'Cargo.toml').read_text())['profile']['release']
        require(profile['lto'] is True and profile['codegen-units'] == 1, 'workspace_release_lto_codegen_mismatch')
    require(command(['git', 'show', '-s', '--format=%P', 'HEAD'], root / 'candidate').split() == SPEC['candidate']['parents'], 'synthetic_merge_parent_mismatch')
    inputs = root / 'inputs'
    for item in SPEC['inputs'].values():
        for row in item['files']:
            source = root / 'candidate' / row['path']
            require(digest(source) == row['sha256'], 'fixture_hash_mismatch')
            target = inputs / row['path']
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
    write(root / 'public/preflight.json', dict(machine, rustc=rustc, pair=SPEC,
          current_pr_head=expected_head, current_merge=current_merge, actual_main_ref=current_main,
          diagnostic_commit=command(['git', 'rev-parse', 'HEAD'], checkout), diagnostic_added_paths=changed,
          current_merge_tree=SPEC['candidate']['tree'],
          analyzer_manifest_scope='All core Rust sources including cfg(test) files; full source identity, not production-equivalence inference',
          baseline_analyzer_source_sha256=analyzer_sources(root / 'baseline'), candidate_analyzer_source_sha256=analyzer_sources(root / 'candidate')))


def environment(target):
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS=str(SPEC['build_jobs']), CARGO_INCREMENTAL='0',
               RUSTUP_TOOLCHAIN='1.99.0', RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='')
    for key in list(env):
        if key.startswith('CARGO_PROFILE_') or key in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
            del env[key]
    return env


def build_library(root, source, mode, arm, instrumentation=False):
    resources(root, build=True)
    target = root / 'compile'
    require(not target.is_symlink(), 'owned_target_is_symlink')
    for path in (target / 'release/.fingerprint').glob('no-mistakes-*'):
        require(not path.is_symlink(), 'owned_fingerprint_is_symlink')
        shutil.rmtree(path)
    args = ['cargo', 'build' if mode == 'production' else 'rustc', '--locked', '-p', 'no-mistakes',
            '--lib', '--release', '--message-format=json']
    if mode == 'production':
        args += ['--bin', 'no-mistakes']
    if instrumentation:
        args += ['--features', 'test-instrumentation']
    if mode != 'production':
        args += ['--', '--cfg', 'receiver_layout_probe', '--check-cfg', 'cfg(receiver_layout_probe)']
    private = root / 'private'
    private.mkdir(exist_ok=True)
    records = command(args, source, environment(target), private / f'{mode}-{arm}-{instrumentation}-cargo.log')
    artifacts = [json.loads(line) for line in records.splitlines() if line.startswith('{')]
    artifacts = [r for r in artifacts if r.get('reason') == 'compiler-artifact']
    owned = [r for r in artifacts if r['target']['name'] == 'no_mistakes' and 'rlib' in r['target']['kind']]
    require(len(owned) == 1, 'expected_one_owned_library')
    artifact = owned[0]
    require(artifact['fresh'] is False, 'owned_crate_not_recompiled')
    require(Path(artifact['target']['src_path']).resolve() == source / 'crates/no-mistakes/src/lib.rs', 'wrong_source_artifact')
    expected = FEATURES + (['test-instrumentation'] if instrumentation else [])
    require(sorted(artifact['features']) == sorted(expected), 'feature_mismatch')
    require(artifact['profile'] == {'opt_level': '3', 'debuginfo': 0, 'debug_assertions': False,
            'overflow_checks': False, 'test': False}, 'release_profile_mismatch')
    library = Path(next(p for p in artifact['filenames'] if p.endswith('.rlib')))
    return target, library, artifacts, artifact


def link(root, target, library, source, output, extra=()):
    before = digest(library)
    require(not output.exists(), 'frozen_binary_already_exists')
    command(['rustc', str(source), '--edition=2021', '-C', 'opt-level=3', '-C', 'lto=true',
             '-C', 'codegen-units=1', '-L', 'dependency=' + str(target / 'release/deps'),
             '--extern', 'no_mistakes=' + str(library), *extra, '-o', str(output)], env=environment(target),
            log=root / 'private' / (output.name + '-link.log'))
    require(digest(library) == before, 'library_changed_during_link')
    return dict(binary_sha256=digest(output), library_sha256=before, source_sha256=digest(source))


def build(root):
    manifests = []
    for arm in ('baseline', 'candidate'):
        source = root / arm
        target, library, artifacts, artifact = build_library(root, source, 'production', arm)
        manifest = dict(revision=SPEC[arm]['revision'], tree=SPEC[arm]['tree'], analyzer_source_sha256=analyzer_sources(source),
                        lock_sha256=digest(source / 'Cargo.lock'), features=artifact['features'],
                        cargo_profile=artifact['profile'], fresh=False, probes={},
                        workspace_manifest_sha256=digest(source / 'Cargo.toml'),
                        crate_manifest_sha256=digest(source / 'crates/no-mistakes/Cargo.toml'),
                        profile='release opt3/full LTO/codegen-units1; default features',
                        verification_graph_probe='untimed; candidate-only alternatives_api bridge')
        for probe in ('timing-probe', 'graph-probe'):
            extra = ['--cfg', 'alternatives_api'] if probe == 'graph-probe' and arm == 'candidate' else []
            manifest['probes'][probe] = link(root, target, library, HERE / (probe + '.rs'), root / (arm + '-' + probe), extra)
        cli = [r for r in artifacts if r['target']['name'] == 'no-mistakes' and r.get('executable')]
        require(len(cli) == 1 and cli[0]['fresh'] is False and
                Path(cli[0]['target']['src_path']).resolve() == source / 'crates/no-mistakes/src/main.rs' and
                cli[0]['features'] == artifact['features'] and cli[0]['profile'] == artifact['profile'], 'wrong_cli_source_profile_or_features')
        binary = cli[0]['executable']
        require(not (root / (arm + '-cli')).exists(), 'frozen_cli_already_exists')
        shutil.copyfile(binary, root / (arm + '-cli'))
        os.chmod(root / (arm + '-cli'), 0o755)
        manifest['cli_sha256'] = digest(root / (arm + '-cli'))
        write(root / f'public/{arm}-production-manifest.json', manifest)
        manifests.append(manifest)
    require(manifests[0]['features'] == manifests[1]['features'], 'unmatched_features')
    require(manifests[0]['cargo_profile'] == manifests[1]['cargo_profile'], 'unmatched_profile')
    for key in ('workspace_manifest_sha256', 'crate_manifest_sha256', 'lock_sha256'):
        require(manifests[0][key] == manifests[1][key], 'unmatched_pair_manifest', invariant=key)
    for key in ('timing-probe', 'graph-probe'):
        require(manifests[0]['probes'][key]['binary_sha256'] != manifests[1]['probes'][key]['binary_sha256'], 'same_pair_binary')
    require(manifests[0]['cli_sha256'] != manifests[1]['cli_sha256'], 'same_pair_cli')


def layout(root):
    # Called only after samples finish. Frozen timing binaries are untouched.
    for arm in ('baseline', 'candidate'):
        source = root / arm
        target_source = source / 'crates/no-mistakes/src/codebase/dependencies/extract_collector.rs'
        before = target_source.read_bytes()
        target_source.write_bytes(before + b'\n' + (HERE / 'layout-helper.rs').read_bytes())
        try:
            for instrumented in (False, True):
                target, library, _, artifact = build_library(root, source, 'layout', arm, instrumented)
                binary = root / f'{arm}-layout-{instrumented}'
                provenance = link(root, target, library, HERE / 'layout-probe.rs', binary)
                rows = [line.split('\t') for line in command([str(binary)]).splitlines()]
                require(len(rows) == 6, 'missing_layout_types')
                write(root / f'public/{arm}-layout-{instrumented}.json', dict(
                      revision=SPEC[arm]['revision'], features=artifact['features'],
                      helper_sha256=digest(HERE / 'layout-helper.rs'), source_mode='measurement-only helper patch',
                      artifact=provenance, layouts=[{'type': name, 'size': int(size), 'align': int(align)} for name, size, align in rows]))
        finally:
            target_source.write_bytes(before)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=('prepare', 'build', 'layout'))
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--expected-pr-head')
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        if args.stage == 'prepare':
            prepare(root, HERE.parents[2], args.expected_pr_head)
        elif args.stage == 'build':
            build(root)
        else:
            layout(root)
    except Exception as error:
        failure = failure_record(error, args.stage)
        write(args.root / f'public/{args.stage}-failure.json', failure)
        print(json.dumps(failure, sort_keys=True))
        sys.exit(1)
