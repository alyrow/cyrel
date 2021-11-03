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
