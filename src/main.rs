#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher},
    Pbkdf2
};

fn hash_password(password: String, salt: String) -> String {
    Pbkdf2.hash_password_simple(password.as_bytes(), salt.as_ref()).unwrap().hash.unwrap().to_string()
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/pass/<password>/<salt>")]
fn fun(password: String, salt: String) -> String {
    hash_password(password, salt)
}

fn main() {
    rocket::ignite().mount("/", routes![index, fun]).launch();
}
