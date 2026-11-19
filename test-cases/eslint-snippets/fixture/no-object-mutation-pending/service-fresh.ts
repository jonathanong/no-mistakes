type Ev = { name: string; target?: { id: string } };

export class Snapshot {
  private items: Array<{ event: Ev }> = [];
  values() {
    return [...this.items];
  }
  entries() {
    return Array.from(this.items.entries());
  }
}

export function freshSnapshot(snapshot: Snapshot) {
  for (const item of snapshot.values()) {
    item.event.target = { id: "x" };
  }
  for (const [, item] of snapshot.entries()) {
    item.event.target = { id: "x" };
  }
}
