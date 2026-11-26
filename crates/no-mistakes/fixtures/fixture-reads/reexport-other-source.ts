import { loadFixture } from "./loader";
consume(loadFixture("reexport-other-source").label);
export { loadFixture } from "./other-loader";
