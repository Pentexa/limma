import { User } from '../../domain/entities/User.js';
export class RegisterUser {
    userRepository;
    constructor(userRepository) {
        this.userRepository = userRepository;
    }
    async execute(name, email) {
        const existingUser = await this.userRepository.findByEmail(email);
        if (existingUser) {
            throw new Error('User already exists');
        }
        const newUser = new User(Math.random().toString(36).substring(7), // Sample ID generation logic
        name, email);
        await this.userRepository.save(newUser);
        return newUser;
    }
}
//# sourceMappingURL=RegisterUser.js.map