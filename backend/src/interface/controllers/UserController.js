import { RegisterUser } from '../../application/use-cases/RegisterUser.js';
export class UserController {
    registerUser;
    constructor(registerUser) {
        this.registerUser = registerUser;
    }
    async handleRegister(req, res) {
        try {
            const { name, email } = req.body;
            const user = await this.registerUser.execute(name, email);
            res.status(201).json(user);
        }
        catch (error) {
            res.status(400).json({ error: error.message });
        }
    }
}
//# sourceMappingURL=UserController.js.map