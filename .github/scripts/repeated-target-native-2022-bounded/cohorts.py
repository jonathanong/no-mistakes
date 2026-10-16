"""Frozen workload partition; statistics never combine different measurement VMs."""
from common import require


def validate(spec):
    workloads = spec['workloads']
    names = [workload['name'] for workload in workloads]
    require(len(names) == 16 and len(set(names)) == 16, 'original_workload_inventory_changed')
    expected = {str(cohort): names[cohort::2] for cohort in (0, 1)}
    require(spec.get('cohorts') == expected, 'fixed_cohort_partition_changed')
    require(spec['bounded_job'] == {'total_seconds': 1800, 'sample_seconds': 1500,
        'non_sampling_reserve_seconds': 300, 'upload_reserve_seconds': 180,
        'other_stage_command_seconds': 1500}, 'bounded_job_contract_changed')
    return expected


def select(spec, cohort):
    partition = validate(spec)
    require(cohort in partition, 'unknown_cohort')
    return [workload for workload in spec['workloads'] if workload['name'] in partition[cohort]]


def verify_union(spec, receipts):
    partition = validate(spec)
    require(len(receipts) == 2 and {row['cohort'] for row in receipts} == set(partition), 'missing_or_duplicate_cohort')
    for row in receipts:
        require(row['complete'] is True and row['post_eligibility_passed'] is True,
                'partial_or_ineligible_cohort')
        require(row['workloads'] == partition[row['cohort']], 'cohort_workload_identity_changed')
        require(set(row['workload_job_ids'])==set(row['workloads']) and all(job==row['measurement_job_id'] for job in row['workload_job_ids'].values()),'workload_split_across_measurement_jobs')
        require(row['sampling_elapsed_s'] <= 1500 and row['whole_job_elapsed_s'] <= 1800,
                'late_cohort')
        require(row['baseline_revision'] == spec['baseline']['revision']
                and row['candidate_revision'] == spec['candidate']['revision'], 'cohort_source_changed')
    require(receipts[0]['run_id'] == receipts[1]['run_id'], 'cross_run_cohort_pooling')
    require(receipts[0]['measurement_job_id'] != receipts[1]['measurement_job_id'], 'cohort_job_identity_collision')
    return True


def verify_rows(spec, cohort, documents):
    from paired_summary import validate as balanced, stable
    import math
    workloads=select(spec,cohort)
    require(set(documents)=={work['name'] for work in workloads},'cross_cohort_or_missing_workload')
    total=0
    for work in workloads:
        phases=documents[work['name']]
        require(set(phases)=={'startup','calibration','pilot','long'},'missing_or_extra_sample_phase')
        for phase in ('startup','calibration'):
            rows=phases[phase]
            require([row['arm'] for row in rows]==['baseline','candidate'],'startup_or_calibration_order_changed')
            count=1 if phase=='startup' else (128 if work['kind']=='probe' else 16)
            require(all(row['count']==count and row['valid'] and row['exit']==0 and not row['timed_out']
                for row in rows),'startup_or_calibration_invalid')
        pilot=phases['pilot'];long=phases['long'];balanced(pilot,1);balanced(long,4)
        require(stable(pilot),'unstable_pilot_cannot_be_complete')
        calibration_count=128 if work['kind']=='probe' else 16
        pilot_count=min(spec['iterations_cap'],max(1,math.ceil(2*calibration_count/max(min(row['cpu_s']for row in phases['calibration']),1e-6))))
        long_count=min(spec['iterations_cap'],max(pilot_count,math.ceil(5/min(row['cpu_s']for row in pilot)*pilot_count)))
        require(all(row['count']==pilot_count for row in pilot)and all(row['count']==long_count for row in long),'iteration_calibration_changed')
        for phase,blocks in [('pilot',1),('long',4)]:
            expected=[(order,block,position,'baseline'if letter=='A'else'candidate')
                for order in ('ABBA','BAAB')for block in range(blocks)for position,letter in enumerate(order)]
            require([(row['order'],row['block'],row['position'],row['arm'])for row in phases[phase]]==expected,'row_order_or_multiplicity_changed')
        for rows in phases.values():
            for row in rows:
                require(row['valid'] and row['exit']==0 and not row['timed_out'] and row['rayon_workers']==4,'invalid_resource_row')
                require(all(math.isfinite(row[key])for key in ('cpu_s','user_s','system_s','wall_s','rss_kib'))
                    and row['user_s']>=0 and row['system_s']>=0
                    and all(row[key]>0 for key in ('cpu_s','wall_s','rss_kib'))
                    and abs(row['cpu_s']-row['user_s']-row['system_s'])<1e-8,'invalid_resource_accounting')
        affinity={tuple(row['affinity'])for rows in phases.values()for row in rows}
        require(len(affinity)==1 and len(next(iter(affinity)))==4,'workload_split_across_machine_affinity')
        total+=sum(len(rows)for rows in phases.values())
    require(total==352,'cohort_full_schedule_incomplete')
    return total
