const fs = require('fs');

class Logger {
    constructor(prefix) {
        this.prefix = prefix;
    }

    log(message) {
        console.log(`${this.prefix}: ${message}`);
    }
}

function greet(name) {
    return `Hello, ${name}!`;
}

function main() {
    const logger = new Logger("App");
    const msg = greet("World");
    logger.log(msg);
}

main();
