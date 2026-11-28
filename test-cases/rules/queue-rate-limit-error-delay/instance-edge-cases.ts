import { Worker } from "glide-mq";

export const EXPORTED_ERR = new Worker.RateLimitError();

declare function getErr(): new () => Error;
declare function getHelper(): { assign(target: Error, src: { delayMs: number }): Error };
declare const extra: Record<string, number>;

function LocalCtor(): void {}

export function instanceEdgeCases(flag: boolean): void {
  const { message } = new Worker.RateLimitError();
  void message;

  let mutable = new Worker.RateLimitError();
  mutable = new Error("other");
  void mutable;

  const readOnly = new Worker.RateLimitError();
  void readOnly.message;
  let sink = "";
  sink = readOnly.message;
  void sink;

  const condAssign = new Worker.RateLimitError();
  if (flag) {
    Object.assign(condAssign, { delayMs: 5000 });
  }
  if (flag) throw condAssign;

  const tpl = new Worker.RateLimitError();
  (tpl as unknown as Record<string, number>)[`delayMs`] = 5000;
  if (flag) throw tpl;

  const compound = new Worker.RateLimitError();
  (compound as unknown as Record<string, number>).delayMs += 5000;
  if (flag) throw compound;

  if (flag) {
    throw Object.assign(new Worker.RateLimitError(), {
      get delayMs() {
        return 5000;
      },
    });
  }
  if (flag) {
    throw Object.assign(new Worker.RateLimitError(), { ...extra });
  }
  if (flag) {
    throw Object.assign(new Worker.RateLimitError(), { __proto__: 1 });
  }
  if (flag) {
    throw Object.freeze(new Worker.RateLimitError());
  }
  if (flag) {
    throw getHelper().assign(new Worker.RateLimitError(), { delayMs: 5000 });
  }
  if (flag) {
    throw new (getErr())();
  }
  if (flag) {
    throw new (LocalCtor as unknown as new () => Error)();
  }
}
