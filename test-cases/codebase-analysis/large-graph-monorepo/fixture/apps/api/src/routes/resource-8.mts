import app from '../app.mts';
import { service8 } from '../services/service-8.mts';
app.route('/api/v1/resource-8/:id').get(() => service8('read-8')).patch(() => service8('patch-8'));
app.post('/api/v1/resource-8', () => service8('create-8'));
