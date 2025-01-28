// @generated automatically by Diesel CLI.

diesel::table! {
    logs (transaction_hash, log_index) {
        address -> Text,
        block_number -> Int8,
        block_hash -> Text,
        chain -> Int8,
        data -> Text,
        log_index -> Int8,
        removed -> Bool,
        timestamp -> Int8,
        topic0 -> Text,
        topic1 -> Nullable<Text>,
        topic2 -> Nullable<Text>,
        topic3 -> Nullable<Text>,
        transaction_hash -> Text,
        transaction_log_index -> Nullable<Int8>,
    }
}

diesel::table! {
    sync_state (id) {
        id -> Text,
        last_block_number -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    logs,
    sync_state,
);
