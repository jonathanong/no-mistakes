"""Exercise the actual download binding and pre-GET seal guard without transport."""
import ast,json,unittest
from pathlib import Path
D=Path(__file__).parent
W=D.parents[1]/'workflows/receiver-fact-native.yml'
def actual_rows():
 text=W.read_text();start=text.index('          assets = [');end=text.index('\n          ]',start)+len('\n          ]')
 return ast.literal_eval(text[start:end].split('=',1)[1].strip())
def actual_guard(rows):
 line=next(line.strip()for line in W.read_text().splitlines()if line.strip().startswith('assert all(isinstance(asset_id,'))
 exec(compile(ast.parse(line),str(W),'exec'),{'assets':rows})
class TransportBindingTests(unittest.TestCase):
 def test_every_actual_workflow_role_matches_asset_metadata(self):
  assets=json.loads((D/'assets.json').read_text());expected={r['role']:(r['asset_id'],r['bytes'],r['sha256'])for r in assets}
  rows=actual_rows();self.assertEqual(len(rows),8);self.assertEqual(len({r[0]for r in rows}),8)
  self.assertEqual({r[0]:tuple(r[1:])for r in rows},expected)
 def test_actual_guard_accepts_saved_reviewed_assets_and_rejects_unsealed_draft(self):
  actual_guard(json.loads((D/'fixtures/prior-reviewed-eight-assets.json').read_text()))
  draft=json.loads((D/'fixtures/actual-seven-bound-one-pending.json').read_text())
  with self.assertRaisesRegex(AssertionError,'Unsealed draft asset bindings'):actual_guard(draft)
