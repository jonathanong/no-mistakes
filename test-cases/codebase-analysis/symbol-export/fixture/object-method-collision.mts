import { alpha, beta } from "./source.mts";

const a = {
  run() {
    return alpha();
  },
};

const b = {
  run() {
    return beta();
  },
};

export const api = a;
