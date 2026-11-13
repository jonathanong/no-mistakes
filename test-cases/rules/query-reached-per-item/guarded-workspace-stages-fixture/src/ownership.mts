import { perform } from './assertion-helper.mts';
export async function guardedOwner(flag: boolean) {
  const stages = [{ run: perform }];
  for (const stage of stages) {
    await stage.run();
    if (flag) return;
  }
}
