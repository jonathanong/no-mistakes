-- A partial unique key bounds only when its predicate is proven for the scan.
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND pd.active = true;
