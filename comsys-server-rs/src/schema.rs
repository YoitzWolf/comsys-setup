// @generated automatically by Diesel CLI.

diesel::table! {
    organisations (id) {
        id -> Int4,
        name -> Text,
        owner -> Int4,
    }
}

diesel::table! {
    tokens (id) {
        id -> Int4,
        hash -> Text,
        ttype -> Int4,
        owner -> Int4,
        sub -> Text,
    }
}

diesel::table! {
    user_orgs (id) {
        id -> Int4,
        uid -> Int4,
        oid -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        login -> Text,
        hash -> Text,
    }
}

diesel::joinable!(organisations -> users (owner));
diesel::joinable!(tokens -> users (owner));
diesel::joinable!(user_orgs -> organisations (oid));
diesel::joinable!(user_orgs -> users (uid));

diesel::allow_tables_to_appear_in_same_query!(
    organisations,
    tokens,
    user_orgs,
    users,
);
