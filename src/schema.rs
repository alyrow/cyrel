table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password -> Text,
        groups -> Integer,
    }
}

table! {
    departments (id) {
        id -> Text,
        name -> Text,
        domain -> Text,
    }
}