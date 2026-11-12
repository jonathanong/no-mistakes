import { STAGES } from './stages.mts';
for (const stage of STAGES) {
  const result = await stage.run();
}
