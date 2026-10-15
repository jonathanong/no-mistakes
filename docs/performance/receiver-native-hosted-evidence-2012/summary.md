# Historical imported-receiver native experiment

Refs #2012 and #1972. Historical 57c927d to 8d81aa1; not current main.
A hosted VM is not a claim of dedicated physical CPUs or PMU availability.
No startup subtraction, outlier removal, or selective reruns.
Full build/cache logs, libraries and executables are withheld.
PMU preflight checks capability only; these sample results are native timing, not hardware-counter attribution.

| Profile | Result |
| --- | --- |
| opt3 | lower cost observed |

opt3/values: lower cost observed; paired CPU -1.219%; descriptive 95% interval [-1.9155724363430664, -0.5289915692476959]; ABBA/BAAB {'ABBA': -0.8766457499500668, 'BAAB': -1.5672806253448468}; arm CPU CV {'baseline': 0.014479589023541739, 'candidate': 0.013985951275290742}.

opt3/imports: inconclusive; paired CPU -0.895%; descriptive 95% interval [-2.0417803271085067, 0.2872924825505316]; ABBA/BAAB {'ABBA': -0.6039578131805667, 'BAAB': -1.1692932175120574}; arm CPU CV {'baseline': 0.019459455329337152, 'candidate': 0.022562789458383935}.
| opt0 | inconclusive |

opt0/values: inconclusive; paired CPU -1.107%; descriptive 95% interval [-2.5981397935963013, 0.17264788313649101]; ABBA/BAAB {'ABBA': -1.8403582496755, 'BAAB': -0.3668125827432367}; arm CPU CV {'baseline': 0.025780452352278355, 'candidate': 0.02090978654193414}.

opt0/imports: inconclusive; paired CPU -0.021%; descriptive 95% interval [-0.566304799114256, 0.548236836834474]; ABBA/BAAB {'ABBA': 0.042911809983814386, 'BAAB': -0.0796299605381856}; arm CPU CV {'baseline': 0.011280745365224434, 'candidate': 0.011203309558668908}.

Intervals resample the eight balanced block log-ratios (10,000 draws, seed 2012).
They describe observed variation, not guaranteed stationarity or tail latency.
Positive cost needs stable samples, matching order directions and an interval above zero;
material regression requires the lower bound to reach 3%. Other results remain inconclusive.
A confirmed regression requires attribution before a runtime fix; #1972 is not closed here.
The original cloud-host opt0 slowdown remains an independent preserved observation.
