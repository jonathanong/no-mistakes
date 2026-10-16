"""Exact reviewed transport regression controls, adapted only module names/source guard isolation."""
import unittest,json,unittest.mock as mock
from pathlib import Path
import phase_common as common

class TransportTests(unittest.TestCase):
    def test_authenticated_modes_survive_safe_tar_filter(self):
        import tarfile,phase_state as state
        for mode,symlink in [(0o664,False),(0o775,False),(0o777,True)]:
            item=tarfile.TarInfo('proof/producer.json');item.mode=mode;item.mtime=123
            if symlink:item.type=tarfile.SYMTYPE;item.linkname='producer-target.json'
            if not symlink:self.assertNotEqual(tarfile.data_filter(item,'/tmp/nm2019-arm2019/baseline').mode,mode)
            row={'mode':mode,'mtime':123,'sha256':'raw','bytes':1,'kind':'symlink'if symlink else'file'}
            manifest={'run_id':'mock','archive_sha256':'archive','files':{item.name:row}}
            archive=mock.MagicMock();archive.getmembers.return_value=[item]
            with mock.patch('phase_source.assert_source'),mock.patch.object(state,'identity',return_value={'run_id':'mock'}),mock.patch.object(state,'sha',return_value='archive'),mock.patch.object(state,'git',return_value=''),mock.patch.object(Path,'read_text',return_value=json.dumps(manifest)),mock.patch.object(Path,'exists',return_value=False),mock.patch.object(Path,'is_symlink',return_value=False),mock.patch.object(Path,'mkdir'),mock.patch.object(state.tarfile,'open')as opened,mock.patch.object(state,'metadata',return_value=row),mock.patch.object(state.os,'chmod')as chmod,mock.patch.object(state.os,'utime')as utime:
                opened.return_value.__enter__.return_value=archive
                state.restore('baseline','core')
                target=common.WORK/'baseline/proof/producer.json'
                if symlink:
                    chmod.assert_not_called();utime.assert_called_once_with(target,(123,123),follow_symlinks=False)
                else:
                    chmod.assert_called_once_with(target,mode,follow_symlinks=False);utime.assert_not_called()
                archive.extract.assert_called_once_with(item,path=common.WORK/'baseline',filter='data')

    def test_only_prior_authenticated_regular_hardlinks_restore(self):
        import tarfile,phase_state as state
        cases=[('proof/base.bin',False,False,False,False,0),('../base.bin',False,False,False,True,0),('/outside',False,False,False,True,0),('proof/missing.bin',False,False,False,True,0),('proof/base.bin',True,False,False,True,0),('proof/base.bin',False,True,False,True,0),('proof/base.bin',False,False,True,True,0),('proof/link.bin',False,False,False,True,0),('proof/base.bin',False,False,False,False,.0000005),('proof/base.bin',False,False,False,True,.000002)]
        for linkname,symlink,mismatch,forward,reject,rounding in cases:
            base=tarfile.TarInfo('proof/base.bin');base.mode=0o644;base.mtime=123
            link=tarfile.TarInfo('proof/link.bin');link.type=tarfile.LNKTYPE;link.linkname=linkname;link.mode=0o644;link.mtime=123
            row={'mode':0o644,'mtime':123,'sha256':'raw','bytes':1,'kind':'file'}
            linkrow=dict(row,sha256='different')if mismatch else row
            manifest={'run_id':'mock','archive_sha256':'archive','files':{base.name:row,link.name:linkrow}}
            archive=mock.MagicMock();archive.getmembers.return_value=[link,base]if forward else[base,link];extracted=set()
            archive.extract.side_effect=lambda item,**kwargs:extracted.add(Path(item.name).name)
            base_reads=0
            def recorded_metadata(path):
                nonlocal base_reads
                if path.name=='base.bin':
                    base_reads+=1
                    if base_reads==2:return dict(row,mtime=row['mtime']+rounding)
                return row
            def is_symlink(path):return symlink and path.name=='base.bin'and path.name in extracted
            with mock.patch('phase_source.assert_source'),mock.patch.object(state,'identity',return_value={'run_id':'mock'}),mock.patch.object(state,'sha',return_value='archive'),mock.patch.object(state,'git',return_value=''),mock.patch.object(Path,'read_text',return_value=json.dumps(manifest)),mock.patch.object(Path,'exists',lambda path:path.name in extracted),mock.patch.object(Path,'is_file',lambda path:path.name in extracted),mock.patch.object(Path,'is_symlink',is_symlink),mock.patch.object(Path,'mkdir'),mock.patch.object(state.tarfile,'open')as opened,mock.patch.object(state,'metadata',side_effect=recorded_metadata),mock.patch.object(state.os,'chmod')as chmod,mock.patch.object(state.os,'utime')as utime:
                opened.return_value.__enter__.return_value=archive
                if reject:
                    with self.assertRaisesRegex(AssertionError,'hardlink'):state.restore('baseline','core')
                    self.assertEqual(archive.extract.call_count,0 if forward else 1)
                else:
                    state.restore('baseline','core');self.assertEqual(archive.extract.call_count,2)
                    self.assertEqual(chmod.call_count,2);utime.assert_not_called()

    def test_zip_permissions_restore_only_after_complete_authentication(self):
        import closed_modes as package
        root=common.WORK/'closed/baseline';files={'binaries/probe':'probe-bytes','binaries/addon.node':'addon-bytes'}
        modes={'binaries/probe':0o755,'binaries/addon.node':0o755}
        permissions={root/name:0o644 for name in files}
        with mock.patch.object(Path,'is_file',return_value=True),mock.patch.object(Path,'is_symlink',return_value=False),mock.patch.object(package,'sha',side_effect=lambda path:files[str(path.relative_to(root))]),mock.patch.object(package.os,'chmod',side_effect=lambda path,mode,**kwargs:permissions.__setitem__(path,mode))as chmod:
            package.restore_file_modes(root,files,modes)
            self.assertEqual(set(permissions.values()),{0o755})
            self.assertTrue(all(call.kwargs=={'follow_symlinks':False}for call in chmod.call_args_list))
        for invalid in [dict(modes,**{'binaries/probe':0o4777}),dict(modes,**{'binaries/probe':True}),{'binaries/probe':0o755}]:
            with mock.patch.object(Path,'is_file',return_value=True),mock.patch.object(Path,'is_symlink',return_value=False),mock.patch.object(package,'sha',side_effect=lambda path:files[str(path.relative_to(root))]),mock.patch.object(package.os,'chmod')as chmod:
                with self.assertRaises(AssertionError):package.restore_file_modes(root,files,invalid)
                chmod.assert_not_called()
        with mock.patch.object(Path,'is_file',return_value=True),mock.patch.object(Path,'is_symlink',return_value=False),mock.patch.object(package,'sha',side_effect=['probe-bytes','wrong']),mock.patch.object(package.os,'chmod')as chmod:
            with self.assertRaises(AssertionError):package.restore_file_modes(root,files,modes)
            chmod.assert_not_called()

