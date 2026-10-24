# Hard inputs under low ignore

Every config selects a nested schema pair under `effort: low` and
`unanalyzable: ignore`. `compatible-low-ignore.yml` is the control: valid
nested inputs yield no finding. Duplicate JSON keys, raw-number precision loss,
a missing consumer document, an excluded consumer, and an invalid consumer
pointer each remain one failing `unproven` finding with no advisory. The
duplicate key and precision-loss tokens are in wrapper metadata outside the
selected `/payload` schema, so the check must validate the whole document before
the low-effort shape policy can ignore the nested comparison.
