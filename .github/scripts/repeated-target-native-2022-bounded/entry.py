"""One bounded event-driven phase; never polls, retries, dispatches or sets statuses."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
from common import require, write, failure_record
from pair import SPEC, reviewed_stage
from cohorts import validate
from deadline import start, clock, remaining


def root():return Path(os.environ['RUNNER_TEMP'])/'repeated-target-2022'


def execute(args):
    destination=root();destination.mkdir(parents=True,exist_ok=True)
    require(SPEC['status']=='REVIEWED AND FROZEN','draft_experiment_cannot_execute')
    validate(SPEC)
    if args.stage=='bootstrap':
        start(destination,os.environ['DIAG_JOB_NAME']);return
    clock(destination);remaining(destination)
    if args.stage=='eligibility':
        remote=reviewed_stage('pair-preflight')
        write(destination/'public/pair-preflight.json',{'remote':remote,'baseline_revision':SPEC['baseline']['revision'],
            'candidate_revision':SPEC['candidate']['revision'],'diagnostic_commit':os.environ['APPROVED_DIAGNOSTIC_COMMIT']})
    elif args.stage=='build':
        from phase_build import build
        build(args.arm,args.phase)
    elif args.stage=='layout':
        from phase_layout import layout
        layout(args.arm,args.phase)
    elif args.stage=='measure':
        from phase_common import run as supervised_run, HERE
        from cohort_run import run_cohort
        from layout_verify import verify as verify_layouts
        verify_layouts(destination)
        environment=dict(os.environ,DIAG_SUPERVISED_PREPARATION='1')
        # One owned process group encloses legacy semantic/NAPI subprocesses.
        supervised_run([sys.executable,str(HERE/'semantic_worker.py')],env=environment,
            log=destination/'public/semantic-preparation',timeout=remaining(destination)-1500)
        remaining(destination)
        run_cohort(destination,args.cohort)
    elif args.stage=='curate':
        from final_union import curate
        curate(destination)
    else:raise ValueError(args.stage)
    remaining(destination)


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('stage',choices=('bootstrap','eligibility','build','layout','measure','curate'))
    parser.add_argument('--arm',choices=('baseline','candidate'))
    parser.add_argument('--phase',choices=('core','cli','addon','timing','graph','probe'))
    parser.add_argument('--cohort',choices=('0','1'))
    args=parser.parse_args();code=0
    try:execute(args)
    except BaseException as error:
        write(root()/'public'/(args.stage+'-failure.json'),failure_record(error,args.stage,arm=args.arm,cohort=args.cohort))
        code=1
    finally:
        try:
            if args.stage=='measure':
                # Separate SQL raw inventory; never alter timing inputs or row counts.
                source=root()/'private/napi-parity';dest=root()/'public/sql-control-raw'
                for prefix in ('conditional-executors-','postgres-source-'):
                    for path in source.glob(prefix+'*') if source.exists() else ():
                        if path.is_file():dest.mkdir(parents=True,exist_ok=True);shutil.copy2(path,dest/path.name)
                from cohort_files import close
                close(root(),args.cohort)
        except BaseException as error:
            write(root()/'public'/'retention-failure.json',failure_record(error,'retention'))
            code=1
        finally:
            # Only this job's isolated targets, including retention-failure paths.
            for namespace in (root(),root()/'layout-default',root()/'layout-instrumented'):
                for arm in ('baseline','candidate'):
                    target=namespace/arm/'target'
                    require(not target.is_symlink(),'owned_target_is_symlink')
                    if target.exists():shutil.rmtree(target)
    return code


if __name__=='__main__':sys.exit(main())
