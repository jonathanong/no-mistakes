import { processImage } from "../lib/image.mts";

export async function handleRequest(url: string) {
  return processImage(url);
}

