import loadFixture from "./loader";
import get from "./loader";
const id = "renamed-default-import";
consume(loadFixture("renamed-default-import").label);
consume(get(id).size);
