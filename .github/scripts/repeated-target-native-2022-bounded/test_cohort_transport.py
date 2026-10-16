"""SOURCE ONLY: saved filenames reproduce upload rejection without native work."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import cohort_transport as transport

FIXTURE = Path(__file__).parent/'test-fixtures/cohort-transport-public.tar'
class TransportTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.public = self.root/'public'
        self.public.mkdir()
        with tarfile.open(FIXTURE) as saved:
            saved.extractall(self.public, filter='data')
        self.env = patch.dict(os.environ, GITHUB_RUN_ID='38894050188', APPROVED_DIAGNOSTIC_COMMIT='source-review-fixture')
        self.env.start()
    def tearDown(self):
        self.env.stop();self.tmp.cleanup()
    def sealed(self):
        directory=self.root/'transport';transport.pack(self.public,directory,'0');return directory
    def replace_archive(self, directory, members):
        archive=directory/transport.ARCHIVE
        with tarfile.open(archive,'w') as output:
            for name,data,kind in members:
                info=tarfile.TarInfo(name);info.type=kind;info.size=len(data) if kind==tarfile.REGTYPE else 0
                output.addfile(info,io.BytesIO(data) if kind==tarfile.REGTYPE else None)
        manifest=json.loads((directory/transport.MANIFEST).read_text())
        manifest.update(sha256=transport.digest(archive),bytes=archive.stat().st_size)
        (directory/transport.MANIFEST).write_text(json.dumps(manifest))
    def test_saved_colon_and_underscore_paths_bytes_roundtrip(self):
        # Direct upload's documented invalid colon path is retained INSIDE TAR.
        self.assertIn(':',str(next(self.public.rglob('*:*')).relative_to(self.public)))
        directory=self.sealed();self.assertEqual({p.name for p in directory.iterdir()},{transport.ARCHIVE,transport.MANIFEST})
        restored=self.root/'restored';transport.unpack(directory,restored,'0')
        expected={p.relative_to(self.public).as_posix():p.read_bytes() for p in self.public.rglob('*') if p.is_file()}
        actual={p.relative_to(restored).as_posix():p.read_bytes() for p in restored.rglob('*') if p.is_file()}
        self.assertEqual(expected,actual);self.assertEqual(len(actual),3)
    def test_extra_missing_tampered_and_foreign_rejected(self):
        for mutation in ('extra','missing','archive-byte','foreign-run','foreign-cohort','foreign-head'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temp:
                directory=Path(temp)/'transport';transport.pack(self.public,directory,'0')
                if mutation=='extra':(directory/'extra').write_bytes(b'')
                elif mutation=='missing':(directory/transport.ARCHIVE).unlink()
                elif mutation=='archive-byte':
                    with (directory/transport.ARCHIVE).open('ab') as output:output.write(b'changed')
                else:
                    manifest=json.loads((directory/transport.MANIFEST).read_text());manifest[{'foreign-run':'run_id','foreign-cohort':'cohort','foreign-head':'diagnostic_commit'}[mutation]]='other';(directory/transport.MANIFEST).write_text(json.dumps(manifest))
                destination=Path(temp)/'restored'
                with self.assertRaises((AssertionError,FileNotFoundError)):transport.unpack(directory,destination,'0')
                self.assertFalse(destination.exists())
    def test_unsafe_duplicate_link_missing_and_member_bytes_fail_before_write(self):
        names=[p.relative_to(self.public).as_posix() for p in self.public.rglob('*') if p.is_file()]
        good=[(name,(self.public/name).read_bytes(),tarfile.REGTYPE) for name in names]
        cases=[good+[good[0]],good[:-1],good+[("../escape",b'x',tarfile.REGTYPE)],[(name,data,tarfile.SYMTYPE if i==0 else kind) for i,(name,data,kind) in enumerate(good)],[(name,b'changed' if i==0 else data,kind) for i,(name,data,kind) in enumerate(good)]]
        for members in cases:
            with tempfile.TemporaryDirectory() as temp:
                directory=Path(temp)/'transport';transport.pack(self.public,directory,'0');self.replace_archive(directory,members)
                destination=Path(temp)/'restored'
                with self.assertRaises(AssertionError):transport.unpack(directory,destination,'0')
                self.assertFalse(destination.exists())
    def test_source_symlink_and_existing_destination_rejected(self):
        (self.public/'link').symlink_to(next(self.public.rglob('*.stderr')))
        with self.assertRaises(AssertionError):transport.pack(self.public,self.root/'bad','0')
        (self.public/'link').unlink();directory=self.sealed()
        destination=self.root/'existing';destination.mkdir()
        with self.assertRaises(AssertionError):transport.unpack(directory,destination,'0')

if __name__=='__main__':unittest.main()
