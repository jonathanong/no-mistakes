import { loadFixture } from "./loader";
consume(loadFixture("reexport-all").label);
export * from "./loader";
