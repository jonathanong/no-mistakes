import {
  dynamicMatch,
  dynamicOther,
  entryReceiver,
  beforeDynamic,
  restoredReceiver,
  first,
  second,
} from "@vendor/dynamic-controls";
const matchingKey = "get";
const otherKey = "set";
for (const item of ["a", "b"]) {
  dynamicMatch.get = first;
  dynamicMatch[matchingKey] = second;
  dynamicMatch.get(item);
}
for (const item of ["a", "b"]) {
  dynamicOther.get = first;
  dynamicOther[otherKey] = second;
  dynamicOther.get(item);
}
for (const item of ["a", "b"]) {
  // The previous iteration's computed key also makes the entry tail uncertain.
  entryReceiver.get(item);
  entryReceiver.get = first;
  entryReceiver[otherKey] = second;
}
for (const item of ["a", "b"]) {
  beforeDynamic.get = first;
  beforeDynamic.get(item);
  beforeDynamic[matchingKey] = second;
}
for (const item of ["a", "b"]) {
  restoredReceiver.get = first;
  restoredReceiver[matchingKey] = second;
  restoredReceiver.get = first;
  restoredReceiver.get(item);
}
