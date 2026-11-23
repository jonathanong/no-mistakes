declare const scheduler: { register(value: number): void };
scheduler.register(0);
// no-mistakes-disable-next-line forbidden-calls: retained exception
scheduler.register(1);
scheduler.register(2); // no-mistakes-disable-line forbidden-calls: retained exception
const example = "// no-mistakes-disable-file forbidden-calls";
scheduler.register(3);
