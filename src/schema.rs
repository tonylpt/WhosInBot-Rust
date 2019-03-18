table! {
    w_roll_call_responses (id) {
        id -> Int8,
        roll_call_id -> Int8,
        unique_token -> Varchar,
        user_id -> Nullable<Int8>,
        user_name -> Nullable<Text>,
        status -> Varchar,
        reason -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    w_roll_calls (id) {
        id -> Int8,
        chat_id -> Int8,
        status -> Varchar,
        title -> Text,
        quiet -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(w_roll_call_responses -> w_roll_calls (roll_call_id));

allow_tables_to_appear_in_same_query!(w_roll_call_responses, w_roll_calls,);
