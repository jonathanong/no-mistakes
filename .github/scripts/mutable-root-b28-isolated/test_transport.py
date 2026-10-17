"""Saved portable native inventory; no binary execution or privileged tokens."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from transport import verify

D = Path(__file__).resolve().parent


class TransportTests(unittest.TestCase):
    def test_complete_inventory_before_any_mode(self):
        assets=json.loads((D/'fixtures/transport-assets.json').read_text())
        stage=D/'fixtures/transport'
        proof=json.loads((stage/'transport.json').read_text())
        context=dict(GITHUB_REPOSITORY=proof['repository'],GITHUB_RUN_ID=proof['run_id'],
                     GITHUB_RUN_ATTEMPT=proof['run_attempt'],GITHUB_SHA=proof['workflow_sha'])
        with patch.object(Path,'chmod') as modes:
            verify(stage,assets,context)
            self.assertEqual(modes.call_count,8)
        for mutate in [lambda p:p.update(complete=False),lambda p:p.update(run_id='other'),
                       lambda p:p['assets'].pop(),lambda p:p['assets'][-1].update(sha256='wrong'),
                       lambda p:p['assets'][-1].update(bytes=0),lambda p:p['assets'][-1].update(exit=1),
                       lambda p:p['assets'][-1].update(asset_id=999),
                       lambda p:p['assets'][-1].update(name=p['assets'][0]['name'])]:
            changed=copy.deepcopy(proof);mutate(changed)
            with patch.object(Path,'read_text',return_value=json.dumps(changed)),patch.object(Path,'chmod') as modes:
                with self.assertRaises(AssertionError):verify(stage,assets,context)
                modes.assert_not_called()
