// This provider has no tag imports or scalar declarations to force binding demand.
export function pureRecord(value: unknown) {
  return { text: `/* pureImportedRecord */ SELECT ${value}`, values: [value] }
}
