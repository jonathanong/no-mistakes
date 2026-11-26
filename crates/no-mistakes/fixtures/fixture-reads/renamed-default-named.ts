import loadFixture from "./loader";
import { default as get } from "./loader";
const id = "renamed-default-named";
consume(loadFixture("renamed-default-named").label);
consume(get(id).size);
