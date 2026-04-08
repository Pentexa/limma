import express from 'express';
import { UserRoutes } from './interface/routes/UserRoutes.js';
const app = express();
app.use(express.json());
app.use('/api/users', UserRoutes);
export default app;
//# sourceMappingURL=app.js.map