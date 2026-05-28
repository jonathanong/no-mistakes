// Minimal stub so the HTTP client imports in the fixture resolve.
export interface Client {
  get(path: string): Promise<unknown>;
}

export const client: Client = {
  async get(_path: string) {
    return {};
  },
};
