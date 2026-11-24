SELECT 1 FROM jobs WHERE id = ':x' FOR KEY SHARE;
DELETE FROM jobs WHERE id = ':x';
