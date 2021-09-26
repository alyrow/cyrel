use super::schema::users;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub groups: i32,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: i32,
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub groups: i32,
}

#[derive(Queryable)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub domain: String,
}
