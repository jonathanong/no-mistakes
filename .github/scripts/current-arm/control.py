"""Run the full original jobs=1 annotation control with a 900-second safety stop."""
import json
from pathlib import Path

import inputs
import common
import worker
from machine import stable_machine, verify_measurement

MAIN = '22a294398a082d08a4167cb79d9499d37b70f7bb'


def main():
    common.install_cancel_handler()
    output = Path('results')
    output.mkdir(exist_ok=True)
    proof = json.loads(Path('out/control/provenance.json').read_text())
    binary = Path('out/control/no-mistakes').resolve()
    assert proof['source'] == MAIN
    assert proof['binarySha256'] == common.digest(binary)
    assert proof['features'] == 'default'
    assert proof['profile']['lto'] is True and proof['profile']['codegen-units'] == 1
    assert proof['buildJobs'] == 2 and proof['incremental'] is False
    machine = verify_measurement(common.machine())
    assert stable_machine(proof['machine']) == stable_machine(machine)
    root = Path('fixture').resolve()
    consumer = json.loads(Path('prepared/fixture-provenance.json').read_text())
    assert common.workload(root) == consumer
    common.save(output / 'provenance.json', {
        'sourceProof': proof, 'measurementMachine': machine, 'consumer': consumer,
        'purpose': 'merge-base-main annotation capacity control',
        'collectionCapSeconds': 900, 'eligibleForGate': False,
        'note': '900 seconds total is a stricter bound than 900 seconds without progress.'})
    worker.CAP_SECONDS = 900
    row = worker.sample(binary, root, root / '.no-mistakes.yml', '1', 'on',
                        False, output, 'main-control', 0, heartbeat=True)
    common.save(output / 'sample.json', row)
    assert common.workload(root) == consumer
    common.save(output / 'outcome.json', {
        'completed': row['eligible'], 'exit': row['exit'],
        'deadlineHit': row['deadlineHit'], 'eligibleForGate': False})


if __name__ == '__main__':
    main()
