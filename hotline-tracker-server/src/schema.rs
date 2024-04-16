table! {
    banlist (id) {
        id -> Integer,
        address -> Text,
        notes -> Text,
        created_at -> Text,
    }
}

table! {
    passwords (id) {
        id -> Integer,
        password -> Text,
        notes -> Text,
        created_at -> Text,
    }
}

allow_tables_to_appear_in_same_query!(banlist, passwords,);
