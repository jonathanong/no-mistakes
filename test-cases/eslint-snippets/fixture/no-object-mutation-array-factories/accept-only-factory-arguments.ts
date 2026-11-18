export function existingArgumentOwnership() {
  const entryValue = { ready: false };
  Object.entries({ entryValue });
  entryValue.ready = true;
  const keyValue = { ready: false };
  Object.keys({ keyValue });
  keyValue.ready = true;
  const fromValue = { ready: false };
  Array.from([fromValue]);
  fromValue.ready = true;
}
