import { query } from 'db';
query(`
  SELECT src.id
  FROM unnest($1::uuid[]) AS src(id)
  LEFT JOIN joined_a ON joined_a.id = src.id
  LEFT JOIN joined_b ON joined_b.id = src.id
  LEFT JOIN joined_c ON joined_c.id = src.id
  LEFT JOIN joined_d ON joined_d.id = src.id
  LEFT JOIN joined_e ON joined_e.id = src.id
  LEFT JOIN joined_f ON joined_f.id = src.id
  LEFT JOIN joined_g ON joined_g.id = src.id
  LEFT JOIN joined_h ON joined_h.id = src.id
  WHERE num_nonnulls(
    COALESCE(joined_a.id, joined_b.id, joined_c.id, joined_d.id, joined_e.id),
    joined_f.id,
    joined_g.id,
    joined_h.id
  ) = 1;
`);
