"""One immutable cohort attempt, all own-workload orders on exactly this VM."""
from phase_common import *
from pair import reviewed_stage
from cohorts import select, verify_rows
from deadline import sampling_deadline, clock, remaining
from measure import measure


def documents(root,cohort):
    result={}
    for work in select(SPEC,cohort):
        result[work['name']]={}
        for phase,suffix in [('startup','startup'),('calibration','calibration'),('pilot','pilot-rows'),('long','long-rows')]:
            path=root/'public'/(work['name']+'-'+suffix+'.json')
            require(path.is_file(),'missing_cohort_rows')
            result[work['name']][phase]=json.loads(path.read_text())
    return result


def run_cohort(root,cohort):
    reviewed_stage('sample');workloads=select(SPEC,cohort)
    require((root/'public/full182-semantic-manifest.json').is_file()
        and (root/'public/napi-parity.json').is_file()
        and (root/'public/conditional-executor-parity.json').is_file()
        and (root/'public/postgres-source-parity.json').is_file(),'missing_full_semantic_or_SQL_proof')
    frozen={str(path.relative_to(root)):sha(path)for prefix in [root/'inputs',root/'public',root/'baseline/transfer',root/'candidate/transfer']
        for path in sorted(prefix.rglob('*'))if path.is_file()}
    write(root/'public/frozen-before-first-row.json',{'run_id':os.environ['GITHUB_RUN_ID'],'cohort':cohort,'files':frozen})
    deadline=sampling_deadline(root);start=time.monotonic();receipt={'complete':False,'post_eligibility_passed':False,
        'cohort':cohort,'run_id':os.environ['GITHUB_RUN_ID'],'measurement_job_id':clock(root)['job_id'],
        'baseline_revision':SPEC['baseline']['revision'],'candidate_revision':SPEC['candidate']['revision'],
        'workloads':[work['name']for work in workloads],
        'workload_job_ids':{work['name']:clock(root)['job_id']for work in workloads},'sample_budget_s':1500,'sampling_started_monotonic':start}
    try:
        measure(root,cohort,deadline)
        receipt['rows']=verify_rows(SPEC,cohort,documents(root,cohort))
        for name,digest in frozen.items():require(sha(root/name)==digest,'frozen_pre_sampling_input_changed')
        remaining(root);reviewed_stage('sample');remaining(root)
        require(time.monotonic()<=deadline,'sampling_deadline_after_postverification')
        receipt['post_eligibility_passed']=True;receipt['complete']=True
    except BaseException as error:
        receipt['failure']=repr(error);raise
    finally:
        receipt['sampling_elapsed_s']=time.monotonic()-start
        receipt['whole_job_elapsed_s']=time.time()-clock(root)['started_unix']
        if receipt['sampling_elapsed_s']>1500 or receipt['whole_job_elapsed_s']>1800:receipt['complete']=False
        write(root/'public/cohort-receipt.json',receipt)
