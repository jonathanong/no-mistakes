// no-mistakes-disable-file forbidden-calls: retained exception
declare const scheduler: { register(value: number): void };
scheduler.register(4);
