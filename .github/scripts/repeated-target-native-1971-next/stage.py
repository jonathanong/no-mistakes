"""Fail-closed executable stages; no event polling and no implicit dispatch."""
import argparse
import json
import os
from pathlib import Path
import sys

from common import ExperimentFailure, failure_record, identity, write
from pair import SPEC, reviewed_stage


def root():
    return Path(os.environ['RUNNER_TEMP'])/'repeated-target-2022'


def execute(stage,arm=None):
    destination=root()
    destination.mkdir(parents=True,exist_ok=True)
    if stage=='pair-preflight':
        snapshot=reviewed_stage(stage)
        write(destination/'public/pair-preflight.json',{'status':'reviewed exact source pair',
              'remote':snapshot,'machine':identity(destination),'driver_revision':os.environ['APPROVED_DIAGNOSTIC_COMMIT'],
              'baseline':SPEC['baseline']['revision'],'candidate':SPEC['candidate']['revision']})
    elif stage=='production-build':
        assert arm in ('baseline','candidate')
        from build_arm import build
        build(arm)
    elif stage=='layout-build':
        assert arm in ('baseline','candidate')
        from layout_arm import layout
        layout(arm)
    elif stage=='semantic-preflight':
        from semantics import preflight
        preflight(destination)
    elif stage=='napi-parity':
        reviewed_stage(stage)
        from napi_parity import run
        run(destination)
    elif stage=='sample':
        reviewed_stage(stage)
        assert (destination/'public/semantic-preflight.json').is_file()
        assert (destination/'public/napi-parity.json').is_file()
        from measure import measure
        measure(destination)
    elif stage=='summary':
        # Preserve every existing row even when the remote source pair drifts or
        # earlier stages fail. report() labels such a run ineligible.
        from report import report
        return report(destination)
    else:raise ValueError(stage)
    return 0


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('stage',choices=('pair-preflight','production-build','layout-build',
        'semantic-preflight','napi-parity','sample','summary'))
    parser.add_argument('--arm',choices=('baseline','candidate'))
    args=parser.parse_args()
    try:
        return execute(args.stage,args.arm)
    except Exception as error:
        destination=root()/'public'
        destination.mkdir(parents=True,exist_ok=True)
        write(destination/(args.stage+'-failure.json'),failure_record(error,args.stage,arm=args.arm))
        print(json.dumps(failure_record(error,args.stage,arm=args.arm),sort_keys=True))
        return 1


if __name__=='__main__':
    sys.exit(main())
