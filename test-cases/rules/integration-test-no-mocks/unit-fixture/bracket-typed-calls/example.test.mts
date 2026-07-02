vi['mock']('x');
vi["fn"]();
jest['spyOn'](service, 'load');
vi.fn<() => User>();
jest.fn<Promise<User>, []>();
vi?.mock('y');
jest?.fn();
const text = "vi['mock']('not-real')";
