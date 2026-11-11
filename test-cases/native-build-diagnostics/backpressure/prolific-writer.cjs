const { once } = require("node:events");
const settings = require("./payload.json");

function chunk(prefix, index) {
  const header = Buffer.from(`${prefix}:${String(index).padStart(4, "0")}:`);
  const value = Buffer.alloc(settings.chunkBytes, prefix === settings.stdoutPrefix ? 0x53 : 0x45);
  header.copy(value);
  return value;
}

async function emit(stream, prefix) {
  for (let index = 0; index < settings.chunks; index++) {
    if (!stream.write(chunk(prefix, index))) await once(stream, "drain");
  }
}

Promise.all([
  emit(process.stdout, settings.stdoutPrefix),
  emit(process.stderr, settings.stderrPrefix),
]).catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
