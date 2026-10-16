"""Union accounting only. Every estimate is reproduced inside its original workload VM."""
from phase_common import *
from cohort_files import verify
from cohort_transport import unpack
from cohorts import verify_rows, verify_union
from cohort_run import documents
from paired_summary import assess
from pair import reviewed_stage
import collections


def curate(root):
    public=root/'public';public.mkdir(parents=True,exist_ok=True)
    record={'eligible':False,'performance_gate':'PENDING independent numeric review','run_id':os.environ['GITHUB_RUN_ID'],
        'baseline_revision':SPEC['baseline']['revision'],'candidate_revision':SPEC['candidate']['revision'],
        'candidate_PR_head':SPEC['pr']['head_sha'],'pooling':'No cross-VM relative pooling; union accounting only'}
    try:
        receipts=[];outcomes=[]
        for cohort in ('0','1'):
            incoming=root/'cohorts'/cohort
            unpack(incoming/'transport',incoming/'public',cohort)
            verify(incoming/'public',cohort)
            receipt=json.loads((incoming/'public/cohort-receipt.json').read_text());receipts.append(receipt)
            verify_rows(SPEC,cohort,documents(incoming,cohort))
            actual=json.loads((incoming/'public/assessments.json').read_text())
            require([row['workload']for row in actual]==SPEC['cohorts'][cohort],'cohort_assessment_order_changed')
            for row in actual:
                rows=json.loads((incoming/'public'/(row['workload']+'-long-rows.json')).read_text())
                expected=assess(rows)
                require(all(row[key]==value for key,value in expected.items()),'frozen_assessment_changed')
                outcomes.append(row)
        verify_union(SPEC,receipts)
        require({row['workload']for row in outcomes}=={work['name']for work in SPEC['workloads']},'whole_experiment_workload_missing')
        require(all(os.environ.get(name)=='success'for name in ('BUILD_RESULT','LAYOUT_RESULT','PREFLIGHT_RESULT','COHORT_RESULT')),'failed_prerequisite_job')
        # Independently bind every completed prerequisite to its actual engine clock.
        api_path='repos/'+os.environ['GITHUB_REPOSITORY']+'/actions/runs/'+os.environ['GITHUB_RUN_ID']+'/attempts/'+os.environ['GITHUB_RUN_ATTEMPT']+'/jobs?per_page=100'
        jobs=json.loads(run(['gh','api',api_path],timeout=30,log=public/'terminal-job-readback'))
        write(public/'terminal-job-api.json',jobs)
        require(jobs['total_count']<=100 and len(jobs['jobs'])==jobs['total_count'],'incomplete_terminal_job_page')
        expected=['eligibility']+[f'production-{phase} ({arm})' for phase in ('core','cli','addon','timing','graph')for arm in ('baseline','candidate')]+[f'layout-{phase} ({arm}, {mode})'for phase in ('core','probe')for arm in ('baseline','candidate')for mode in ('default','instrumented')]+[f'measurement ({cohort})'for cohort in ('0','1')]
        import datetime
        def seconds(value):return datetime.datetime.fromisoformat(value.replace('Z','+00:00')).timestamp()
        for name in expected:
            matched=[job for job in jobs['jobs']if job['name']==name]
            require(len(matched)==1 and matched[0]['conclusion']=='success','actual_prerequisite_job_not_success')
            job=matched[0]
            require(0<=seconds(job['completed_at'])-seconds(job['started_at'])<=1800,'actual_prerequisite_job_exceeds_1800')
        reviewed_stage('summary')
        record.update(eligible=True,complete_rows=704,cohort_receipts=receipts,
            metric_status_counts=dict(collections.Counter(metric['status']for row in outcomes for metric in row['metrics'].values())))
        write(public/'all16-assessments.json',outcomes)
    except BaseException as error:
        record['failure']=repr(error);record['performance_gate']='INELIGIBLE or incomplete; all retained rows remain evidence'
        raise
    finally:
        write(public/'final-pair.json',record)
