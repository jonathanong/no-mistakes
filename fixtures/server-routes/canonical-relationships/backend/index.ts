import { users } from "./api/users";

const app = express();
app.use("/api/v1", users);
