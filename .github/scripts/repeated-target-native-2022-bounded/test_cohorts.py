"""Saved real row-shape controls for the new partition; never performance evidence."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from cohorts import validate, select, verify_rows, verify_union
from common import ExperimentFailure

HERE=Path(__file__).resolve().parent
SPEC=json.loads((HERE/'pair.json').read_text())
SAVED=json.loads((HERE/'test-fixtures/historical704-validator-data.json').read_text())['documents']


def rows(cohort):
    return copy.deepcopy({work['name']:SAVED[work['name']]for work in select(SPEC,cohort)})


def receipt(cohort):
    names=SPEC['cohorts'][cohort];job=100+int(cohort)
    return {'cohort':cohort,'complete':True,'post_eligibility_passed':True,'workloads':names,
        'workload_job_ids':{name:job for name in names},'measurement_job_id':job,'run_id':'UNIT ONLY',
        'sampling_elapsed_s':1200,'whole_job_elapsed_s':1450,'baseline_revision':SPEC['baseline']['revision'],
        'candidate_revision':SPEC['candidate']['revision']}


class CohortTests(unittest.TestCase):
    def test_all_original_rows_and_fixed_partition(self):
        self.assertEqual([verify_rows(SPEC,c,rows(c))for c in ('0','1')],[352,352])
        self.assertTrue(verify_union(SPEC,[receipt('0'),receipt('1')]))
    def test_partition_missing_duplicate_and_reassigned_reject(self):
        for kind in ('missing','duplicate','reassigned'):
            spec=copy.deepcopy(SPEC)
            if kind=='missing':spec['cohorts'].pop('1')
            if kind=='duplicate':spec['cohorts']['0'][1]=spec['cohorts']['0'][0]
            if kind=='reassigned':spec['cohorts']['0'][0],spec['cohorts']['1'][0]=spec['cohorts']['1'][0],spec['cohorts']['0'][0]
            with self.assertRaises(ExperimentFailure):validate(spec)
    def test_missing_duplicate_split_foreign_or_failed_cohort_reject(self):
        for kind in ('missing','duplicate','split','foreign_run','foreign_source','partial','postverify','late_sample','late_job'):
            values=[receipt('0'),receipt('1')]
            if kind=='missing':values.pop()
            elif kind=='duplicate':values[1]=copy.deepcopy(values[0])
            elif kind=='split':values[0]['workload_job_ids'][values[0]['workloads'][0]]=999
            elif kind=='foreign_run':values[1]['run_id']='different'
            elif kind=='foreign_source':values[1]['candidate_revision']='0'*40
            elif kind=='partial':values[1]['complete']=False
            elif kind=='postverify':values[1]['post_eligibility_passed']=False
            elif kind=='late_sample':values[1]['sampling_elapsed_s']=1500.1
            else:values[1]['whole_job_elapsed_s']=1800.1
            with self.assertRaises(ExperimentFailure):verify_union(SPEC,values)
    def test_missing_partial_reordered_changed_resource_or_count_rows_reject(self):
        for kind in ('missing','partial','reorder','resource','count','affinity'):
            data=rows('0');key=next(iter(data))
            if kind=='missing':data.pop(key)
            elif kind=='partial':data[key]['long'].pop()
            elif kind=='reorder':data[key]['long'][0],data[key]['long'][1]=data[key]['long'][1],data[key]['long'][0]
            elif kind=='resource':data[key]['long'][0]['cpu_s']+=1
            elif kind=='count':data[key]['long'][0]['count']+=1
            else:data[key]['startup'][0]['affinity']=[9,10,11,12]
            with self.assertRaises((ExperimentFailure,AssertionError)):verify_rows(SPEC,'0',data)
    def test_reserve_and_job_deadline_never_extend(self):
        from deadline import remaining,sampling_deadline
        with patch('deadline.clock',return_value={'deadline_monotonic':2000}),patch('deadline.time.monotonic',return_value=500):
            self.assertEqual(remaining(Path('/unused')),1320)
            with self.assertRaises(ExperimentFailure):sampling_deadline(Path('/unused'))
            with self.assertRaises(ExperimentFailure):remaining(Path('/unused'),0)
        with patch('deadline.clock',return_value={'deadline_monotonic':600}),patch('deadline.time.monotonic',return_value=500):
            with self.assertRaises(ExperimentFailure):remaining(Path('/unused'))
