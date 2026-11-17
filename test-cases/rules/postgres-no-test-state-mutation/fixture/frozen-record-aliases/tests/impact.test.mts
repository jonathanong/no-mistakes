import { writeConst, writeLet, writeReassigned } from '../test-helpers/write.mts';

export async function exercise(name: string, column: string) {
  await writeConst(name);
  await writeLet(name);
  await writeReassigned(name, column);
}
