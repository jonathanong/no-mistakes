import * as l from "./loader";
const id = "namespace-member";
consume(loadFixture("namespace-member").label);
consume(l.loadFixture(id).size);
