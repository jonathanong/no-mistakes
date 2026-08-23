import { globSync } from 'glob';

const pattern = 'resources/**/*.txt';
export const dynamicResources = globSync(pattern);
