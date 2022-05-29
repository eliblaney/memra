table! {
    addons (id) {
        id -> Int4,
        user_id -> Int4,
        visibility -> Nullable<Bool>,
        name -> Varchar,
        description -> Text,
        data -> Nullable<Bytea>,
    }
}

table! {
    cards (id) {
        id -> Int4,
        user_id -> Int4,
        deck_id -> Int4,
        front -> Nullable<Bytea>,
        back -> Nullable<Bytea>,
    }
}

table! {
    course_subscriptions (id) {
        id -> Int4,
        user_id -> Int4,
        course_id -> Int4,
    }
}

table! {
    coursedecks (id) {
        id -> Int4,
        course_id -> Int4,
        deck_id -> Int4,
    }
}

table! {
    courses (id) {
        id -> Int4,
        user_id -> Int4,
        visibility -> Nullable<Bool>,
        name -> Varchar,
        image -> Nullable<Bytea>,
    }
}

table! {
    credentials (id) {
        id -> Int4,
        user_id -> Int4,
        email -> Varchar,
        password -> Varchar,
    }
}

table! {
    deck_subscriptions (id) {
        id -> Int4,
        user_id -> Int4,
        deck_id -> Int4,
    }
}

table! {
    decks (id) {
        id -> Int4,
        user_id -> Int4,
        visibility -> Nullable<Bool>,
        name -> Varchar,
        image -> Nullable<Bytea>,
    }
}

table! {
    followers (id) {
        id -> Int4,
        follower_id -> Int4,
        following_id -> Int4,
    }
}

table! {
    history (id) {
        id -> Int4,
        user_id -> Int4,
        card_id -> Int4,
        ts -> Nullable<Timestamp>,
        num_confident -> Int4,
        num_correct -> Int4,
        num_wrong -> Int4,
    }
}

table! {
    notifications (id) {
        id -> Int4,
        user_id -> Int4,
        message -> Text,
        icon -> Nullable<Bytea>,
    }
}

table! {
    settings (id) {
        id -> Int4,
        user_id -> Int4,
        public_profile -> Nullable<Bool>,
        avatar -> Nullable<Bytea>,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        real_name -> Nullable<Varchar>,
        verified -> Bool,
    }
}

joinable!(addons -> users (user_id));
joinable!(cards -> decks (deck_id));
joinable!(cards -> users (user_id));
joinable!(course_subscriptions -> courses (course_id));
joinable!(course_subscriptions -> users (user_id));
joinable!(coursedecks -> courses (course_id));
joinable!(coursedecks -> decks (deck_id));
joinable!(courses -> users (user_id));
joinable!(credentials -> users (user_id));
joinable!(deck_subscriptions -> decks (deck_id));
joinable!(deck_subscriptions -> users (user_id));
joinable!(decks -> users (user_id));
joinable!(history -> cards (card_id));
joinable!(history -> users (user_id));
joinable!(notifications -> users (user_id));
joinable!(settings -> users (user_id));

allow_tables_to_appear_in_same_query!(
    addons,
    cards,
    course_subscriptions,
    coursedecks,
    courses,
    credentials,
    deck_subscriptions,
    decks,
    followers,
    history,
    notifications,
    settings,
    users,
);
