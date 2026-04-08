import { User } from '../../domain/entities/User.js';
import type { UserRepository } from '../../domain/repositories/UserRepository.js';
export declare class InMemoryUserRepository implements UserRepository {
    private users;
    save(user: User): Promise<void>;
    findById(id: string): Promise<User | null>;
    findByEmail(email: string): Promise<User | null>;
    findAll(): Promise<User[]>;
}
//# sourceMappingURL=InMemoryUserRepository.d.ts.map