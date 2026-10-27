SELECT 1 FROM partial_docs AS pd JOIN directories AS d ON d.id = $2 WHERE pd.slug = $1 AND d.active = true;
