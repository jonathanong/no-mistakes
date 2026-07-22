import defaultApi, {
  api,
  eagerApi,
  NamedService,
  Service,
} from './exported-member-consumer';

test('loads exported member resources', () => {
  api.load();
  new Service().load();
  new NamedService().load();
  expect(eagerApi.schema).toBeTruthy();
  expect(defaultApi.schema).toBeTruthy();
});
