"""Fresh genuine Cargo producers, separated at bounded command boundaries."""
from phase_common import *
from phase_source import checkout, compiler, select
from phase_state import directory, restore, seal, cleanup
from pair import reviewed_stage
import shutil

PHASES = ['core', 'cli', 'addon', 'timing', 'graph']


def refresh_library(data, root):
    for line in data.splitlines():
        if not line.startswith(b'{'): continue
        row = json.loads(line)
        if row.get('reason') != 'compiler-artifact' or row['target']['name'] != 'no_mistakes' or 'rlib' not in row['target']['kind']: continue
        path = Path(next(name for name in row['filenames'] if name.endswith('.rlib')))
        if row['fresh']:
            require(sha(path) == sha(root/'binaries/lib.rlib'), 'fresh_library_differs_from_producer')
        else:
            require(sorted(row['features']) == sorted(SPEC['features']), 'library_features_changed')
            shutil.copy2(path, root/'binaries/lib.rlib')
            write(root/'proof/library-producer.json', row)
        write(root/'proof/library-observation.json', row)


def cargo(phase, source, root, env):
    args = ['cargo', 'build', '--locked', '--release', '-p', 'no-mistakes']
    args += ['--lib'] if phase == 'core' else ['--bin', 'no-mistakes']
    if phase == 'addon': args = ['cargo', 'rustc', '--locked', '--release', '-p', 'no-mistakes', '--lib', '--crate-type', 'cdylib']
    args += ['--message-format=json']
    data = run(args, cwd=source, env=env, log=root/'proof'/phase)
    kind = {'core':'rlib', 'cli':'bin', 'addon':'cdylib'}[phase]
    row, _ = select(data, 'no-mistakes' if phase == 'cli' else 'no_mistakes', kind)
    require(Path(row['manifest_path']) == source/'crates/no-mistakes/Cargo.toml', 'foreign_Cargo_producer')
    write(root/'proof'/(phase+'-producer.json'), row)
    if phase in ('core', 'cli'): refresh_library(data, root)
    if phase == 'cli': shutil.copy2(row['executable'], root/'binaries/cli')
    if phase == 'addon': shutil.copy2(next(path for path in row['filenames'] if path.endswith('.so')), root/'binaries/addon.node')


def materialize_graph_probe(root):
    payload=HERE/'graph-probe.rs.txt'
    pinned=json.loads((HERE/'proposal-file-hashes.json').read_text())['files']['graph-probe.rs.txt']
    require(payload.is_file()and not payload.is_symlink()and sha(payload)==pinned,'graph_probe_payload_changed')
    target=root/'harness/graph-probe.rs'
    require(not target.exists()and not target.is_symlink(),'graph_probe_materialization_already_exists')
    target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(payload.read_bytes())
    require(sha(target)==pinned,'graph_probe_materialization_changed')
    write(root/'proof/graph-probe-materialization.json',{'payload':'graph-probe.rs.txt',
        'payload_sha256':pinned,'materialized_sha256':sha(target)})
    return target


def link(phase, arm, root, env):
    name = phase+'-probe'
    source=materialize_graph_probe(root)if phase=='graph'else HERE/(name+'.rs')
    args = ['rustc', '+1.99.1', str(source), '--edition=2021', '-C', 'opt-level=3',
        '-C', 'lto=true', '-C', 'codegen-units=1', '-C', 'debuginfo=0', '-L',
        'dependency='+str(root/'target/release/deps'), '--extern', 'no_mistakes='+str(root/'binaries/lib.rlib')]
    if phase == 'graph' and arm == 'candidate': args += ['--cfg', 'alternatives_api']
    run(args+['-o', str(root/'binaries'/name)], env=env, log=root/'proof'/name)


def build(arm, phase):
    reviewed_stage('production-build')
    require(phase in PHASES, 'unknown_production_phase')
    root = directory(arm)
    source = checkout(arm)
    previous = PHASES[PHASES.index(phase)-1] if phase != 'core' else None
    try:
        if previous: restore(arm, previous)
        (root/'proof').mkdir(exist_ok=True); (root/'binaries').mkdir(exist_ok=True)
        run(['rustup', 'update', 'stable'], log=root/'proof/stable-update', timeout=120)
        rustc = compiler(); env = environment(root/'target')
        if phase in ('core', 'cli', 'addon'): cargo(phase, source, root, env)
        else: link(phase, arm, root, env)
        require(not git('status', '--porcelain', cwd=source), 'production_source_changed')
        write(root/'proof'/(phase+'-closed.json'), {'arm':arm, 'phase':phase, 'revision':SPEC[arm]['revision'],
            'tree':SPEC[arm]['tree'], 'rustc':rustc, 'source_manifest_sha256':SPEC[arm]['source_manifest_sha256']})
        if phase != 'graph': seal(arm, phase)
        else:
            from phase_package import package
            package(arm, rustc)
    finally: cleanup(arm)
