import { globSync } from 'glob';

export const textResources = globSync('glob-resources/**/*.txt');
