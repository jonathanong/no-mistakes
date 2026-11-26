function load(id: string): Promise<Widget | null>;
function load(id: string, extra: boolean): Promise<Widget | null>;
function load(id: string): Promise<Widget | null> {
  return Promise.resolve(null);
}

export async function read(id: string): Promise<Widget | null> {
  return load(id);
}
