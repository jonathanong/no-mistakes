import direct from './exported-default-direct';
import AnonymousService from './exported-default-anonymous-class';
import Service from './exported-default-named-class';
import nested from './exported-default-nested';
import wrapped from './exported-default-wrapped';

test('loads default expression resources', () => {
  expect(direct).toBeTruthy();
  expect(wrapped.config).toBeTruthy();
  expect(new Service().load()).toBeTruthy();
  expect(new AnonymousService().load()).toBeTruthy();
  expect(nested.load()).toBe(1);
});
