vi['mock']('x');
vi["fn"]();
jest['spyOn'](service, 'load');
vi.fn<() => User>();
jest.fn<Promise<User>, []>();
const text = "vi['mock']('not-real')";
