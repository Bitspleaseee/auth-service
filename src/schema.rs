#![allow(proc_macro_derive_resolution_fallback)]
table! {
    roles (id) {
        id -> Integer,
        name -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Integer,
        email -> Varchar,
        username -> Varchar,
        password -> Tinytext,
        banned -> Nullable<Bool>,
        verified -> Nullable<Bool>,
        email_token -> Nullable<Varchar>,
    }
}

joinable!(roles -> users (id));

allow_tables_to_appear_in_same_query!(
    roles,
    users,
);
