import { read, sql } from '@example/db';

export function mappedBuilderAlias() {
  const query = sql`SELECT 1`;
  const mapped = [false, true].map(append => {
    if (append) query.append(sql`; DELETE FROM records`);
    return query;
  });
  read(mapped[0]);
}

export function mappedNestedBuilderAlias() {
  const query = sql`SELECT 1`;
  const mapped = [false, true].map(append => {
    if (append) query.append(sql`; DELETE FROM records`);
    return { query };
  });
  read(mapped[0].query);
}

export function mappedArrayBuilderAlias() {
  const query = sql`SELECT 1`;
  const mapped = [false, true].map(append => {
    if (append) query.append(sql`; DELETE FROM records`);
    return [query];
  });
  read(mapped[0][0]);
}
