table! {
    entries (id) {
        id -> Int4,
        journey_id -> Int4,
        user_id -> Int4,
        created -> Timestamp,
        archived -> Bool,
        description -> Nullable<Varchar>,
        coordinates -> Nullable<Varchar>,
        location -> Nullable<Varchar>,
    }
}

table! {
    journeys (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        archived -> Bool,
        start_date -> Timestamp,
        end_date -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        date -> Timestamp,
    }
}

joinable!(entries -> journeys (journey_id));
joinable!(entries -> users (user_id));
joinable!(journeys -> users (user_id));

allow_tables_to_appear_in_same_query!(entries, journeys, users,);
