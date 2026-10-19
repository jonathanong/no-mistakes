"""Fan out only when every required job count has an uncensored canary pair."""
import json
import os
from pathlib import Path

from common import save

expected = {(case, jobs) for case in ('verdicts-ledger', 'phases-full')
            for jobs in ('1', '4', 'default')}
observed = {}
for path in Path('canaries').glob('*/provenance.json'):
    proof = json.loads(path.read_text())
    identity = (proof['case'], proof['jobs'])
    assert identity in expected and identity not in observed
    assert proof['harnessCommit'] == os.environ['GITHUB_SHA']
    assert proof['order'] == 'bc' and proof['diagnostics'] == 'off'
    block_path = path.with_name('block.json')
    block = json.loads(block_path.read_text()) if block_path.exists() else {}
    rows_path = path.with_name('samples.json')
    rows = json.loads(rows_path.read_text()) if rows_path.exists() else []
    complete = (block.get('eligible') is True and len(rows) == 2 and
                [row['arm'] for row in rows] == ['b', 'c'] and
                all(row.get('eligible') is True for row in rows))
    # This is a scheduling feasibility bound, never a regression budget or a
    # replacement for the unchanged 600-second collection cap.
    fits = complete and all(row['wallSeconds'] <= 240 for row in rows)
    observed[identity] = {'case': identity[0], 'jobs': identity[1],
                          'complete': complete, 'fitsThirtyMinuteBlock': fits,
                          'wallSeconds': [row.get('wallSeconds') for row in rows]}
feasible = set(observed) == expected and all(row['fitsThirtyMinuteBlock']
                                            for row in observed.values())
result = {'feasible': feasible, 'expectedPairs': len(expected),
          'observedPairs': len(observed), 'pairs': list(observed.values()),
          'interpretation': 'Canaries are feasibility evidence, not a balanced performance gate.'}
save('feasibility.json', result)
with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
    output.write('feasible=' + str(feasible).lower() + '\n')
with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as output:
    output.write('Balanced matrix scheduling feasible: ' + str(feasible) + '\n')
