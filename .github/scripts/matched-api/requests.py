"""Saved request shapes, referencing only the fixed checked-out physical fixtures."""
from pathlib import Path

def make_request(pair,case,root):
    if pair=='2127':
        assert case in ['representative','stress']
        return {'root':str(root),'reports':[{'type':'importUsages','id':f'usages-{i}'}for i in range(1 if case=='representative'else 128)]}
    assert pair=='2129' and case in ['representative','control']
    return {'root':str(Path(root)/'fixtures/lang-frontends/python-celery-django'),'reports':([{'type':'queues'},{'type':'serverRoutes'},{'type':'dependencies','files':['app/enqueue.py']}]if case=='representative'else[{'type':'queues'}])}

def nonvacuous(pair,case,report):
    assert report['reports']
    if pair=='2127':
        expected=1 if case=='representative' else 128
        assert len(report['reports'])==expected
        for i,item in enumerate(report['reports']):
            assert item['type']=='importUsages' and item['id']==f'usages-{i}'
            files=item['result']['files'];assert files
            assert any(f['imports']for f in files),'real import occurrences required, roots alone do not count'
            assert all(isinstance(f['path'],str) and f['path']for f in files)
            for f in files:
                for use in f['imports']:
                    assert isinstance(use['specifier'],str) and use['specifier']
                    assert use['kind'] in ['static','type','dynamic','require','require-resolve']
                    assert isinstance(use['line'],int) and not isinstance(use['line'],bool) and use['line']>0
    else:
        result=[x['result']for x in report['reports']]
        assert result[0]['producers']
        if case=='representative':assert result[1]['routes'] and result[2]['files']
