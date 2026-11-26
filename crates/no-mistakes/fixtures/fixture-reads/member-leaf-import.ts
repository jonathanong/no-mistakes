import * as loader from "./loader";
import { fetch } from "./loader";
const id = "member-leaf-import";
consume(loader.fetch("member-leaf-import").label);
consume(fetch(id).size);
