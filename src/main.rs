#[macro_use]
extern crate lazy_static;

mod console;

use console::log;

trait Derp {
    fn foo(&self) -> String;
}

impl Derp for bool {
    fn foo(&self) -> String {
        format!("HEH {}", self)
    }
}

fn print_derpa<T: Derp>(arg: T) {
    println!("{}", arg.foo())
}

fn main() {
    println!("Hello, world!");


    log::log("Foo!");
    log::log("Derp!");
    log::log("Klerp!");

    log::print_last_10();

    let boole = false;

    dbg!(boole);
    print_derpa(boole);
}
