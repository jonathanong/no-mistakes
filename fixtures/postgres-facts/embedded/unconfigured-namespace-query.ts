import * as http from './request';
// No configured module or broad executor selector proves this `query` export.
const { query: request } = http;
request({ resource: 'http' });
http.query({ resource: 'http' });
