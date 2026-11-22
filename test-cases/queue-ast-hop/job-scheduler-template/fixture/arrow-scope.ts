import { Queue } from "bullmq";

const orders = new Queue("orders");
const options = { delay: 1 };

export function scheduleWithArrow() {
  const wrap = () => {
    var options = { attempts: 1 };
    orders.upsertJobScheduler("inner", { every: 1000 }, { opts: options });
    return options;
  };
  orders.upsertJobScheduler("outer", { every: 1000 }, { opts: options });
  return wrap;
}

export class Scheduler {
  static {
    const options = { delay: 2 };
    const wrap = () => {
      var options = { attempts: 2 };
      orders.upsertJobScheduler("staticInner", { every: 1000 }, { opts: options });
      return options;
    };
    orders.upsertJobScheduler("staticOuter", { every: 1000 }, { opts: options });
    void wrap;
  }
}
