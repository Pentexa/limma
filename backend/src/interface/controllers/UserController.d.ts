import type { Request, Response } from 'express';
import { RegisterUser } from '../../application/use-cases/RegisterUser.js';
export declare class UserController {
    private registerUser;
    constructor(registerUser: RegisterUser);
    handleRegister(req: Request, res: Response): Promise<void>;
}
//# sourceMappingURL=UserController.d.ts.map