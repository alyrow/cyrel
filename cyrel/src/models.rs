use serde::Serialize;

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}

pub struct NewUser {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub groups: Vec<i32>,
}

pub struct Department {
    pub id: String,
    pub name: String,
    pub domain: String,
}

#[derive(Serialize)]
pub struct Identity {
    pub firstname: String,
    pub lastname: String,
}
