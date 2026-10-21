type User = { id: string };

export function namedCall(): User | null {
  return rpc.call("/users/1");
}

export function spreadCall(): User | null {
  return rpc.call(...["/users/1"]);
}
