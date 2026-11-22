import { blocks, boxes, classes, copies, heads, indexes, loops, ranks, slots } from "./queues";

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
  for (const cursor of [indexes.add("fromIndex", { id }, { priority: LEVEL }), ...levels]) {
    loops.add("boxedItem", { id }, { priority: cursor });
  }
  indexes.add("storeIndex", { id });
}

export function enqueueShadowedHead(levels: number[], id: string) {
  for (const LEVEL of [heads.add("shadowHead", { id }, { priority: LEVEL }), ...levels]) {
    loops.add("shadowBox", { id }, { priority: LEVEL });
  }
  heads.add("storeHead", { id });
  heads.add("rushHead", { id }, { priority: 4 });
}

export function enqueueBlocked(id: string) {
  class Local {
    static {
      var LEVEL = 2;
    }
  }
  blocks.add("rankedBlock", { id }, { priority: LEVEL });
  return Local;
}

export function enqueuePlainBlock(id: string) {
  blocks.add("plainBlock", { id });
}

export function enqueueBoxed(id: string) {
  const Box = class LEVEL {
    send() {
      classes.add("classItem", { id }, { priority: LEVEL });
    }
  };
  classes.add("storeClass", { id });
  classes.add("rushClass", { id }, { priority: 2 });
  boxes.add("outerBox", { id }, { priority: LEVEL });
  boxes.add("storeBox", { id });
  return Box;
}

export function enqueueCopied(id: string, job: object) {
  copies.addBulk([{ ...job }]);
  copies.add("rushCopy", { id }, { priority: 4 });
  copies.add("storeCopy", { id });
}

export function enqueueRanked(id: string) {
  for (const LEVEL = 9; ;) {
    ranks.add("countRank", { id }, { priority: LEVEL });
    break;
  }
  ranks.add("outerRank", { id }, { priority: LEVEL });
  ranks.add("storeRank", { id });
}
