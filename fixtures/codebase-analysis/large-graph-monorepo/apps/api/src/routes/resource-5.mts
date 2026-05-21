import app from '../app.mts';
import { service5 } from '../services/service-5.mts';
app.route('/api/v1/resource-5/:id').get(() => service5('read-5')).patch(() => service5('patch-5'));
app.post('/api/v1/resource-5', () => service5('create-5'));
