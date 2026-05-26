import app from '../app.mts';
import { service7 } from '../services/service-7.mts';
app.route('/api/v1/resource-7/:id').get(() => service7('read-7')).patch(() => service7('patch-7'));
app.post('/api/v1/resource-7', () => service7('create-7'));
