import { indexes, loops, ranks, slots } from "./queues";

const LEVEL = 5;

export function enqueueLooped(levels: number[], id: string) {
  for (const LEVEL of levels) {
    loops.add("loopItem", { id }, { priority: LEVEL });
  }
  loops.add("storeLoop", { id });
  loops.add("rushLoop", { id }, { priority: 2 });
}

export function enqueueSlotted(levels: number[], id: string) {
  for (const LEVEL in levels) {
    slots.add("slotItem", { id }, { priority: LEVEL });
  }
  slots.add("storeSlot", { id });
  slots.add("rushSlot", { id }, { priority: 3 });
}

export function enqueueIndexed(levels: number[], id: string) {
  for (const LEVEL of [indexes.add("fromIndex", { id }, { priority: LEVEL }), ...levels]) {
    loops.add("boxedItem", { id }, { priority: LEVEL });
  }
  indexes.add("storeIndex", { id });
}

export function enqueueRanked(id: string) {
  for (const LEVEL = 9; ;) {
    ranks.add("countRank", { id }, { priority: LEVEL });
    break;
  }
  ranks.add("outerRank", { id }, { priority: LEVEL });
  ranks.add("storeRank", { id });
}
