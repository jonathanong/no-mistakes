declare const api: { getUser(id: string): Promise<User | null> };
type Loader = (id: string) => Promise<User | null>;
type Api = { getUser(id: string): Promise<User | null> };

export const ok = async (id: string): Promise<User | null> => api.getUser(id);
export const wrapped = (async (id: string): Promise<User | null> => api.getUser(id)) satisfies Loader;
export async function wrappedRecv(id: string): Promise<User | null> {
  return (api satisfies Api).getUser(id);
}
