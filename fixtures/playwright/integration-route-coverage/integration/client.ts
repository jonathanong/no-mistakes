export class WebClient {
  async request(path: string) { return fetch(path) }
}
export async function loadPage(origin: string, path: string) { return fetch(origin + path) }
