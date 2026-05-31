import { loaded } from "./loaded.mts";

export function run(loaded = () => {}) {
  loaded();
}
