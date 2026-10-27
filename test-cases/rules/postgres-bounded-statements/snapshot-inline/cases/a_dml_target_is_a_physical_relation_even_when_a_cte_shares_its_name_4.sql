WITH c AS (SELECT id FROM exports ORDER BY id LIMIT $1) UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id;
