vi.mock('../src/target.mts', () => ({ value: 'mocked' }));
vi.mock('./runtime-helper.mts', () => ({ value: 'cut before registration' }));
