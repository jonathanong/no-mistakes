import sql from 'sql-template-strings';

// Keep the block body independent of concise-arrow support: only aliasing differs.
const block = () => {
  return sql`/* blockAlias */ SELECT 1`;
};
export { block as blockAlias };
export const direct = () => sql`/* directArrow */ SELECT 1`;
const concise = () => sql`/* conciseAlias */ SELECT 1`;
export { concise as conciseAlias };
