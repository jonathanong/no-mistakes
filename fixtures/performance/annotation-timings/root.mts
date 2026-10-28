import { read } from '@data-stores/psql'
import sql from 'sql-template-strings'

// Keep postgres_annotation_timings.snap in sync when lines change.
// This unsupported class call exercises the unmodeled-call timing.
class TimingProbe {
  inspect() { Math.max(1, 2) }
}

// Independent entrypoints replay initialized state; sequence arms require a full join.
export function run(condition: boolean) {
  const annotated = sql`/* timing fixture */ SELECT 1`
  const query = condition ? (0, annotated) : (0, annotated)
  read(query)
  read(sql`SELECT 2`) // unsafe-query
}
