import { loadFixture } from "./loader";
consume(loadFixture("reexport-named").label);
export { loadFixture } from "./loader";
