import sharp from "sharp";

export async function processImage(path: string) {
  return sharp(path).resize(100).toBuffer();
}
