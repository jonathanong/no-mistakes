"""A ready lower stack layer may merge without changing pinned native inputs."""
import copy
import unittest
from stack_guard import verify_stack


class StackGuard(unittest.TestCase):
    def setUp(self):
        self.spec = dict(main='old', baseline_head='parent', baseline_tree='tree',
                         baseline_branch='parent-branch', candidate_head='candidate')
        self.observed = dict(parent=dict(head=dict(sha='parent'), state='open', merged=False),
                             pr=dict(state='open', draft=True, head=dict(sha='candidate'),
                                     base=dict(ref='parent-branch', sha='parent')),
                             baseline_commit=dict(tree=dict(sha='tree')),
                             main=dict(object=dict(sha='old')),
                             main_commit=dict(tree=dict(sha='old-tree'), parents=[dict(sha='older')]))

    def test_open_ready_and_merged_parent(self):
        verify_stack(self.spec, self.observed)
        self.observed['parent']['draft'] = False
        verify_stack(self.spec, self.observed)
        self.observed['parent'].update(state='closed', merged=True, merge_commit_sha='merged')
        self.observed['main']['object']['sha'] = 'merged'
        self.observed['main_commit'] = dict(tree=dict(sha='tree'), parents=[dict(sha='old')])
        verify_stack(self.spec, self.observed)
        self.observed['pr']['base'] = dict(ref='main', sha='merged')
        verify_stack(self.spec, self.observed)

    def test_each_identity_drift_rejected(self):
        cases = [('parent head changed', lambda d: d['parent']['head'].update(sha='other')),
                 ('parent tree changed', lambda d: d['baseline_commit']['tree'].update(sha='other')),
                 ('parent closed unmerged', lambda d: d['parent'].update(state='closed')),
                 ('candidate must remain open draft', lambda d: d['pr'].update(draft=False)),
                 ('candidate head changed', lambda d: d['pr']['head'].update(sha='other')),
                 ('unrelated main advance', lambda d: d['main']['object'].update(sha='other')),
                 ('candidate base changed', lambda d: d['pr']['base'].update(sha='other'))]
        for reason, change in cases:
            with self.subTest(reason=reason):
                observed = copy.deepcopy(self.observed)
                change(observed)
                with self.assertRaisesRegex(AssertionError, '^' + reason + '$'):
                    verify_stack(self.spec, observed)


if __name__ == '__main__':
    unittest.main()
