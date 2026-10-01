namespace Legacy {
  // Not flagged: `export =` makes the namespace the module itself, so it is
  // never an ordinary export of the file.
  export class LegacyDead extends Error {}
}

export = Legacy;
