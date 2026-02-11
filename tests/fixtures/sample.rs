use std::collections::HashMap;

struct Config {
    name: String,
    value: i32,
}

impl Config {
    fn new(name: &str, value: i32) -> Self {
        Config {
            name: name.to_string(),
            value,
        }
    }

    fn display(&self) {
        println!("{}: {}", self.name, self.value);
    }
}

fn helper(x: i32) -> i32 {
    x * 2
}

fn main() {
    let cfg = Config::new("test", 42);
    cfg.display();
    let result = helper(cfg.value);
    println!("Result: {}", result);
}
