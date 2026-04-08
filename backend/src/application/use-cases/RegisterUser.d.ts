import { User } from '../../domain/entities/User.js';
import type { UserRepository } from '../../domain/repositories/UserRepository.js';
export declare class RegisterUser {
    private userRepository;
    constructor(userRepository: UserRepository);
    execute(name: string, email: string): Promise<User>;
}
//# sourceMappingURL=RegisterUser.d.ts.map