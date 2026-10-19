"""Preregistered effect thresholds and a conservative replication requirement."""
import math
import statistics

ADOPTION_RULE = {
    "status": "pre-registered", "results": None, "applied_to_summary": False,
    "median_wall_ratio_max": 1.00, "pair_wall_ratio_max": 1.05,
    "median_peak_rss_ratio_at_jobs_1_max": 1.00,
    "median_peak_rss_ratio_at_max_jobs_on_many_core_host_max": 0.95,
    "report_parity": "100%", "minimum_pairs_per_cell": 12,
    "one_sided_median_confidence": 0.95,
}


def upper_median_bound(values):
    """Exact binomial order statistic; four observations cannot bound at 95%."""
    values = sorted(values)
    n = len(values)
    for k in range(1, n + 1):
        if sum(math.comb(n, j) for j in range(k, n + 1)) / 2 ** n <= 0.05:
            return values[k - 1]
    return None


def evaluate_cell(groups, many_core):
    pairs = [pair for group in groups for pair in group['pairs']]
    identities = [(p.get('measurement_id'), p.get('host_id'), p['pair']) for p in pairs]
    if (any(not run or not host for run, host, _ in identities)
            or len(identities) != len(set(identities))):
        raise ValueError('missing or duplicated measurement/pair identity')
    if any(group.get('valid') is not True for group in groups) or any(p.get('valid') is not True for p in pairs):
        raise ValueError('invalid or censored paired evidence')
    jobs = groups[0]['jobs']
    thresholds = {'wall_seconds': 1.00}
    if jobs == '1':
        thresholds['peak_rss_bytes'] = 1.00
    elif many_core and jobs == 'max':
        thresholds['peak_rss_bytes'] = 0.95
    observed, bounds, failures = {}, {}, []
    for metric, limit in thresholds.items():
        values = [p['ratio'].get(metric) for p in pairs]
        if any(type(v) not in (int, float) or not math.isfinite(v) or v <= 0 for v in values):
            raise ValueError('missing or invalid gate metric')
        observed[metric] = statistics.median(values)
        bounds[metric] = upper_median_bound(values)
        if observed[metric] > limit:
            failures.append(metric)
    if any(p['ratio']['wall_seconds'] > 1.05 for p in pairs):
        failures.append('pair_wall_seconds')
    sufficient = len(pairs) >= ADOPTION_RULE['minimum_pairs_per_cell']
    passed = sufficient and not failures and all(bounds[k] is not None and bounds[k] <= v for k, v in thresholds.items())
    return {'pairs': len(pairs), 'pointMedianRatios': observed, 'upper95MedianRatios': bounds,
            'observedThresholdFailures': failures, 'sufficientReplication': sufficient,
            'eligible': passed}
