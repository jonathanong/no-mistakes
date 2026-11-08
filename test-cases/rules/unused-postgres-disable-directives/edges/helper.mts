// no-mistakes-disable-file postgres-required-predicates: stale header
export function probe() {
  return `
-- no-mistakes-disable-next-line postgres-required-predicates: shared indexed probe
SELECT id FROM topic_posts WHERE topic_id = $1 LIMIT 1
`;
}
