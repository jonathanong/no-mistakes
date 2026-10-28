import { outsideEffect } from '../outside/sql';

export function insideSql() {
  return 'DROP TABLE app.inside_helper';
}

export function finiteSql(mode: string) {
  switch (mode) {
    case 'clear': return 'SELECT 1';
    default: throw outsideEffect();
  }
}
