import {
  duplicate as sameOrigin,
  differentModule as distinctModules,
  differentName as distinctNames,
  localCollision,
} from "./external-star-identities.mts";
import { unresolvedCollision as unresolvedStar } from "./external-star-unresolved.mts";

export function callExternalStars() {
  sameOrigin();
  distinctModules();
  distinctNames();
  localCollision();
  unresolvedStar();
}
