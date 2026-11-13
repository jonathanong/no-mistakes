import { CHILD_STAGES } from './stages.mts';
export async function purgeUntyped(db, id: string, n: number) {
  for (const stage of CHILD_STAGES) {
    await stage.deletion(db, id, n);
  }
}
export async function purgeTemplated(db, id: string) {
  for (const stage of CHILD_STAGES) {
    await stage.remaining(db, `${id}`);
  }
}
