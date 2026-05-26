import sharp from "sharp";

export function processImage(path: string) {
  return sharp(path).toBuffer();
}
