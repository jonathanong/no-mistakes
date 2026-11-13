import {read} from '@bench/db';
import {plain, builder} from './producer.mjs';
{ const {text, values} = plain(); read(text, values); }
{ const {text, values} = builder(); read(text, values); }
