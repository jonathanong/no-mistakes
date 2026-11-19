type Ev = { name: string; target?: { id: string } };

interface Service {
  values(): Array<{ event: Ev }>;
}

class Store extends Map<string, Ev> {}

export function mapValues(rows: Map<string, Ev>) {
  for (const event of rows.values()) {
    event.target = { id: "x" };
  }
}

export function mapEntries(rows: Map<string, Ev>) {
  for (const [, event] of rows.entries()) {
    event.target = { id: "x" };
  }
}

export function setValues(rows: Set<Ev>) {
  for (const event of rows.values()) {
    event.target = { id: "x" };
  }
}

export function tupleValues(row: [Ev, Ev]) {
  for (const event of row.values()) {
    event.target = { id: "x" };
  }
}

export function readonlyValues(rows: readonly Ev[]) {
  for (const event of rows.values()) {
    event.target = { id: "x" };
  }
}

export function storeValues(store: Store) {
  for (const event of store.values()) {
    event.target = { id: "x" };
  }
}

export function arrayOrService(source: Ev[] | Service) {
  for (const item of source.values()) {
    (item as Ev).target = { id: "x" };
  }
}

export function constrainedArray<T extends Ev[]>(rows: T) {
  for (const event of rows.values()) {
    event.target = { id: "x" };
  }
}

export function unconstrained<T>(rows: T) {
  for (const item of rows.values()) {
    item.target = { id: "x" };
  }
}

export function anyValues(service: any) {
  for (const item of service.values()) {
    item.target = { id: "x" };
  }
}

export function unknownValues(service: unknown) {
  for (const item of service.values()) {
    item.target = { id: "x" };
  }
}

export function plain(value: { name: string }) {
  for (const item of value.values()) {
    item.target = { id: "x" };
  }
}
