type Ev = { name: string; target?: { id: string } };

interface Service {
  values(): Array<{ event: Ev }>;
  entries(): Array<[number, { event: Ev }]>;
}

interface Other {
  values(): Array<{ event: Ev }>;
}

export function serviceValues(service: Service) {
  for (const item of service.values()) {
    item.event.target = { id: "x" };
  }
}

export function serviceEntries(service: Service) {
  for (const [, item] of service.entries()) {
    item.event.target = { id: "x" };
  }
}

export function eitherService(service: Service | Other) {
  for (const item of service.values()) {
    item.event.target = { id: "x" };
  }
}

export function extended(service: Service & { extra: number }) {
  for (const item of service.values()) {
    item.event.target = { id: "x" };
  }
}

export function constrained<T extends Service>(service: T) {
  for (const item of service.values()) {
    item.event.target = { id: "x" };
  }
}

export class Registry {
  private byId = new Map<string, { event: Ev }>();
  values() {
    return this.byId.values();
  }
}

export function sharedRegistry(registry: Registry) {
  for (const item of registry.values()) {
    item.event.target = { id: "x" };
  }
}
