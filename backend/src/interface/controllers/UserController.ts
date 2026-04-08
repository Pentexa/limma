import type { Request, Response } from 'express';
import { RegisterUser } from '../../application/use-cases/RegisterUser.js';

export class UserController {
  constructor(private registerUser: RegisterUser) {}

  async handleRegister(req: Request, res: Response): Promise<void> {
    try {
      const { name, email } = req.body;
      const user = await this.registerUser.execute(name, email);
      res.status(201).json(user);
    } catch (error: any) {
      res.status(400).json({ error: error.message });
    }
  }
}
