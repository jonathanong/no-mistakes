import { FRAGMENT as shared } from "./fragments.mts";
// An executor in a sibling module observes this mutation of the shared object.
shared.append(" OFFSET 100");
