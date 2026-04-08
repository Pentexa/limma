import { User } from '../../domain/entities/User.js';
import type { UserRepository } from '../../domain/repositories/UserRepository.js';

export class RegisterUser {
  constructor(private userRepository: UserRepository) {}

  async execute(name: string, email: string): Promise<User> {
    const existingUser = await this.userRepository.findByEmail(email);
    if (existingUser) {
      throw new Error('User already exists');
    }

    const newUser = new User(
      Math.random().toString(36).substring(7), // Sample ID generation logic
      name,
      email
    );

    await this.userRepository.save(newUser);

    return newUser;
  }
}
