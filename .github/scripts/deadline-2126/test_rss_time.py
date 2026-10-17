from pathlib import Path
import unittest
from rss_time import child_rss
class RssTests(unittest.TestCase):
    def test_saved_strict_receipts(self):
        d=Path(__file__).parent/'fixtures'
        self.assertEqual(child_rss((d/'rss-valid.txt').read_bytes()),44520)
        for name in ['rss-zero.txt','rss-extra.txt']:
            with self.assertRaises(AssertionError):child_rss((d/name).read_bytes())
if __name__=='__main__':unittest.main()
