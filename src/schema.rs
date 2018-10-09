table! {
    roles (id) {
        id -> Unsigned<Integer>,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Unsigned<Integer>,
        email -> Varchar,
        username -> Varchar,
        password -> Tinytext,
        banned -> Bool,
        verified -> Bool,
        email_token -> Nullable<Varchar>,
    }
}

joinable!(roles -> users (id));

allow_tables_to_appear_in_same_query!(roles, users,);
