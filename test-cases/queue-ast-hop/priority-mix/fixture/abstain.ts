import { credits, drafts, extras, shipments, tags, weights } from "./queues";

export function enqueueKnownShipment(id: string) {
  return shipments.add("packShipment", { id }, { priority: 4 });
}

export function enqueueChosenShipment(id: string, level: number) {
  return shipments.add("packShipment", { id }, { priority: level });
}

export function enqueueWholeWeight(id: string) {
  return weights.add("weighItem", { id }, { priority: 0 });
}

export function enqueuePartialWeight(id: string) {
  return weights.add("weighItem", { id }, { priority: 1.5 });
}

export function enqueueZeroCredit(id: string) {
  return credits.add("applyCredit", { id }, { priority: 0 });
}

export function enqueueNegativeCredit(id: string) {
  return credits.add("applyCredit", { id }, { priority: -1 });
}

export function enqueuePlainExtra(id: string) {
  return extras.add("packExtra", { id });
}

export function enqueueSpreadExtra(id: string, extra: object) {
  return extras.add("packExtra", { id }, { priority: 3, ...extra });
}

export function enqueuePlainDraft(id: string) {
  return drafts.add("saveDraft", { id });
}

export function enqueueLeveledDraft(id: string) {
  let level = 2;
  return drafts.add("saveDraft", { id }, { priority: level });
}

export function enqueuePlainTag(id: string) {
  return tags.add("printTag", { id });
}

export function enqueueKeyedTag(id: string, key: string) {
  return tags.add("printTag", { id }, { priority: 1, [key]: 0 });
}
