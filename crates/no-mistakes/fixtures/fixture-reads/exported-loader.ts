import { loadFixture } from "./loader";
consume(loadFixture("exported-loader").label);
export { loadFixture };
