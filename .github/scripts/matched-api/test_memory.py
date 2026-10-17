import io,json,subprocess,unittest,zipfile
from pathlib import Path
from frozen_verify import inventory_archive
P=Path(__file__).parent
class MemoryTests(unittest.TestCase):
    def test_actual_worker_parser(self):
        source=(P/'worker.cjs').read_text();start=source.index('  const vmHwm =');end=source.index('\n',source.index('assert(Number.isSafeInteger',start));snippet=source[start:end]
        controls=json.loads((P/'saved-memory-status.json').read_text())
        for name,status in controls.items():
            js="const assert=require('node:assert');const apiCompletionProcStatus="+json.dumps(status)+';'+snippet
            result=subprocess.run(['node','-e',js],capture_output=True,timeout=10)
            self.assertEqual(result.returncode==0,name=='positive',name)
    def test_archive_rejects_traversal_and_duplicates(self):
        for entries in [['../escape'],['/absolute'],['x','x']]:
            b=io.BytesIO()
            with zipfile.ZipFile(b,'w')as z:
                for name in entries:z.writestr(name,b'public')
            b.seek(0)
            with zipfile.ZipFile(b)as z,self.assertRaises(AssertionError):inventory_archive(z)
    def test_memory_only_scope(self):
        self.assertNotIn("'wall_seconds'",(P/'analyze.py').read_text())
        w=(P/'workflow-source.yml').read_text();self.assertNotIn('  build:',w);self.assertEqual(w.count('            case:'),24);self.assertEqual(w.count('run-id: 39039054266'),2)
if __name__=='__main__':unittest.main()
