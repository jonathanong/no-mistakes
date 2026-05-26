import { getUsers } from '../../lib/api';

export default function DashboardPage() {
  getUsers();
  return <div>Dashboard</div>;
}
