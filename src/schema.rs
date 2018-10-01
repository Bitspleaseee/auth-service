#[macro_use]

table! {
    users (id) {
        id -> Integer,
        email -> Text,
        username -> Varchar,
        password -> Text,
        banned -> Bool,
        verified -> Bool,
        email_token -> Integer,
    }
}


table! {
    roles(id) {
        id -> Integer,
        name -> Varchar,
    }
}


joinable!(roles -> users (id));


allow_tables_to_appear_in_same_query!(users, roles);