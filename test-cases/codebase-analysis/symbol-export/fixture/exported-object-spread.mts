import { alpha } from './source.mts';

const extra = {};

export const api = {
  ...extra,
  label: alpha,
  run() {
    return alpha();
  },
};
