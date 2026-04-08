import { Router } from 'express';
import { UserController } from '../controllers/UserController.js';
import { RegisterUser } from '../../application/use-cases/RegisterUser.js';
import { InMemoryUserRepository } from '../../infrastructure/persistence/InMemoryUserRepository.js';
const router = Router();
const userRepository = new InMemoryUserRepository();
const registerUserUseCase = new RegisterUser(userRepository);
const userController = new UserController(registerUserUseCase);
router.post('/register', (req, res) => userController.handleRegister(req, res));
export { router as UserRoutes };
//# sourceMappingURL=UserRoutes.js.map