import { type HelperType, loadFixture } from "./loader";
import type { FixtureShape } from "./types";
import { unusedHelper } from "./\uD800";
import * as memberLoader from "./member-loader";
declare const ambientLoader: (id: string) => { size: number };
declare global {
  function globalAmbientLoader(id: string): { size: number };
}

export { unusedHelper } from "./other";
export { type loadFixture } from "./other";

consume(loadFixture("direct-import").size);
consume(memberLoader.fetch("direct-member").size);
consume(ambientLoader("direct-ambient").size);
consume(globalAmbientLoader("direct-global").size);
