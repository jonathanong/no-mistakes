import { loadFixture, loadFixture as get } from "./loader";
const id = "aliased-import";
consume(loadFixture("aliased-import").label);
consume(get(id).size);
