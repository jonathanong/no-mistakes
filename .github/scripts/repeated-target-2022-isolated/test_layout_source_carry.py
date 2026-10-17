import hashlib,json,unittest
from pathlib import Path
import contracts
D=Path(__file__).parent
class SourceCarry(unittest.TestCase):
 def plan(self):
  p=json.loads((D/'fixtures/historical-five-plan.json').read_text());g=p['layout_gate']
  for key in ['proof','root_review']:
   path=D/'fixtures'/('current-layout-'+('proof' if key=='proof' else 'review')+'.json');g[key]={'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
  source=D/'fixtures/current-layout-source-carry.json';review=D/'fixtures/current-layout-source-review.json';data=json.loads(source.read_text());p['candidate_actual_compiler']=data['new_native_candidate'];p['candidate_current_source']=data['public_cfg_only_carry'];g['source_carry']={key:{'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}for key,path in [('proof',source),('root_review',review)]}
  # Retain the authentic prior24 bytes; only their saved fixture paths change.
  g['proof']['sha256']=data['prior_actual24']['sha256']
  return p
 def test_exact_saved_source_carry_preserves_all24(self):
  p=self.plan();self.assertEqual(contracts.layouts(p),24)
 def test_wrong_candidate_and_source_pin_rejected(self):
  for key in ['candidate_actual_compiler','source_pin']:
   p=self.plan()
   if key=='source_pin':p['layout_gate']['source_carry']['proof']['sha256']='0'*64
   else:p[key]='wrong'
   with self.assertRaises(AssertionError):contracts.layouts(p)
