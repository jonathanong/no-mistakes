import app from '../app.mts';
import { service9 } from '../services/service-9.mts';
app.route('/api/v1/resource-9/:id').get(() => service9('read-9')).patch(() => service9('patch-9'));
app.post('/api/v1/resource-9', () => service9('create-9'));
