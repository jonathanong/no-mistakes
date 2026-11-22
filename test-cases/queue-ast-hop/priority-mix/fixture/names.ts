import { defaults, marks, others } from "./queues";

const RANK = 5;

const sendMark = function RANK(id: string) {
  return marks.add("shadeMark", { id }, { priority: RANK });
};

export function enqueueStoredMark(id: string) {
  marks.add("storeMark", { id });
}

export default function RANK(id: string) {
  defaults.add("shadeDefault", { id }, { priority: RANK });
}

export function enqueueStoredDefault(id: string) {
  defaults.add("storeDefault", { id });
}

const sendOther = function LABEL(id: string) {
  return others.add("shadeOther", { id }, { priority: RANK });
};

export function enqueueStoredOther(id: string) {
  others.add("storeOther", { id });
}
