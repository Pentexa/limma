import { User } from '../../domain/entities/User.js';
export class InMemoryUserRepository {
    users = [];
    async save(user) {
        const userIndex = this.users.findIndex(u => u.id === user.id);
        if (userIndex !== -1) {
            this.users[userIndex] = user;
        }
        else {
            this.users.push(user);
        }
    }
    async findById(id) {
        return this.users.find(u => u.id === id) || null;
    }
    async findByEmail(email) {
        return this.users.find(u => u.email === email) || null;
    }
    async findAll() {
        return [...this.users];
    }
}
//# sourceMappingURL=InMemoryUserRepository.js.map