import { read } from '@data-stores/psql'
import { query, selected } from './state.mts'
read(query)
read(selected)
