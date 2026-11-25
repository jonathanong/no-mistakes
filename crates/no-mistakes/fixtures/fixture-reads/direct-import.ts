import { loadFixture } from "./loader";
import * as memberLoader from "./member-loader";
declare const ambientLoader: (id: string) => { size: number };

consume(loadFixture("direct-import").size);
consume(memberLoader.fetch("direct-member").size);
consume(ambientLoader("direct-ambient").size);
