import { coreFn8 } from '@fixture/core/core-8.mts';
import type { DataRecord8 } from '@fixture/data/records/data-8.mts';
export function Card8({ record }: { record: DataRecord8 }) {
  return <section data-testid="card-8">{coreFn8()}{record.id}</section>;
}
