import { shadows } from "./queues";

const RANK = 5;

export function enqueueShade(RANK: number, id: string) {
  shadows.add("shadeItem", { id }, { priority: RANK });
  shadows.add("storeShadow", { id });
  shadows.add("rushShadow", { id }, { priority: 2 });
}
