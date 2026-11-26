import { loadFixture } from "./loader";
consume(loadFixture("local-rebinding").label);
function inner() {
  const loadFixture = otherLoader;
  consume(loadFixture("other").size);
}
