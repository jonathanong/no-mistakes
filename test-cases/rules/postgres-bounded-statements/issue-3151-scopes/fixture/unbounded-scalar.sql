SELECT COALESCE((SELECT k.label FROM kinds k), 'missing');
