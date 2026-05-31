import { loaded } from "./loaded.mts";

export function run() {
  try {
    loaded();
  } catch (error) {
    loaded();
  }
  loaded();
}
