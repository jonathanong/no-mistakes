import { alpha, beta } from './source.mts';

class Other {
  run() {
    return beta();
  }
}

export class Client {
  run() {
    return alpha();
  }
}
