export class User {
    id;
    name;
    email;
    createdAt;
    constructor(id, name, email, createdAt = new Date()) {
        this.id = id;
        this.name = name;
        this.email = email;
        this.createdAt = createdAt;
    }
}
//# sourceMappingURL=User.js.map