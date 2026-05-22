import data from './data.json';
import './style.css';
import { privateValue } from '#private';
import { local as selfLocal } from '@fixture/app/local';

export const entry = [data, privateValue, selfLocal];
