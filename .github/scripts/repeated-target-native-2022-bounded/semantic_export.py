"""Retain all182 original raw/full-object controls; SQL controls are a separate inventory."""
from phase_common import *
import shutil


def export(root):
    dest=root/'public/semantic-raw';dest.mkdir(exist_ok=False)
    selected={}
    def take(source,name):
        require(source.is_file() and not source.is_symlink(),'missing_semantic_raw_control')
        require(name not in selected,'duplicate_semantic_raw_control')
        target=dest/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
        selected[name]={'sha256':sha(target),'bytes':target.stat().st_size}
    for key in SPEC['semantic_policy']['untimed_graph_inputs']:
        for arm in ARM:
            for suffix in ('stdout','stderr'):
                take(root/'private/semantics'/f'{key}-{arm}.graph.{suffix}',f'graphs/{key}-{arm}.{suffix}')
            take(root/'private/reports/graph'/f'{key}-{arm}.json',f'reports/graph/{key}-{arm}.json')
    for work in SPEC['workloads']:
        key=work['name']
        for arm in ARM:
            if work['kind']=='cli':
                for suffix in ('stdout','stderr','exit'):
                    take(root/'private/semantics'/f'{key}-{arm}'/f'0.{suffix}',f'cli-raw/{key}-{arm}/0.{suffix}')
                take(root/'private/reports/cli'/f'{key}-{arm}.json',f'reports/cli/{key}-{arm}.json')
            else:
                for suffix in ('stdout','stderr'):
                    take(root/'private/semantics'/f'{key}-{arm}.probe.{suffix}',f'probes/{key}-{arm}.{suffix}')
    for case in SPEC['semantic_policy']['napi_cases']:
        key=case['prefix']
        for arm in ARM:
            for label in [f'{key}-{arm}',*[f'{key}-{arm}-{i}'for i in range(len(case['entries']))],f'{key}-{arm}-check']:
                for suffix in ('stdout','stderr'):
                    take(root/'private/napi-parity'/f'{label}.{suffix}',f'napi/{label}.{suffix}')
            take(root/'private/reports/napi'/f'{key}-{arm}.json',f'reports/napi/{key}-{arm}.json')
    require(len(selected)==182,'full182_semantic_inventory_incomplete')
    write(root/'public/full182-semantic-manifest.json',{'run_id':os.environ['GITHUB_RUN_ID'],
        'baseline_revision':SPEC['baseline']['revision'],'candidate_revision':SPEC['candidate']['revision'],
        'report_hashes':SPEC['semantic_policy']['report_hashes'],'raw_files':selected,'count':182})
