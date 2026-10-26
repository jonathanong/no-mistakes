-- A partial unique key bounds only when its predicate is proven for the scan.
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND pd.active = true;
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1;
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND (pd.active = true OR pd.archived = true);
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND pd.archived = true;
SELECT 1 FROM partial_docs AS pd JOIN directories AS d ON d.id = $2 WHERE pd.slug = $1 AND d.active = true;
SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND pd.active::text = 'true';
