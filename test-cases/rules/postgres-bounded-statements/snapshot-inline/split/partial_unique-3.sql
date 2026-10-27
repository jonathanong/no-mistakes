SELECT 1 FROM partial_docs AS pd WHERE pd.slug = $1 AND (pd.active = true OR pd.archived = true);
