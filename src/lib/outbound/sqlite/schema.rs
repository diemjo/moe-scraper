// @generated automatically by Diesel CLI.

diesel::table! {
    amiami_category (id) {
        id -> Integer,
        date_added -> Timestamp,
        category -> Text,
        following -> Bool,
    }
}

diesel::table! {
    amiami_product (id) {
        id -> Integer,
        date_added -> Timestamp,
        url -> Text,
        title -> Text,
        image_url -> Text,
        category_id -> Integer,
        maker -> Text,
        full_price -> Integer,
        min_price -> Integer,
        availability -> Text,
    }
}

diesel::table! {
    melonbooks_artist (id) {
        id -> Integer,
        date_added -> Timestamp,
        name -> Text,
        following -> Bool,
        date_followed -> Nullable<Timestamp>,
    }
}

diesel::table! {
    melonbooks_category (id) {
        id -> Integer,
        date_added -> Timestamp,
        category -> Text,
    }
}

diesel::table! {
    melonbooks_flag (id) {
        id -> Integer,
        date_added -> Timestamp,
        flag -> Text,
    }
}

diesel::table! {
    melonbooks_product (id) {
        id -> Integer,
        date_added -> Timestamp,
        url -> Text,
        title -> Text,
        image_url -> Text,
        category_id -> Integer,
        availability -> Text,
        price -> Nullable<Text>,
        circle -> Nullable<Text>,
    }
}

diesel::table! {
    melonbooks_product_artist (product_id, artist_id) {
        product_id -> Integer,
        artist_id -> Integer,
    }
}

diesel::table! {
    melonbooks_product_flag (product_id, flag_id) {
        product_id -> Integer,
        flag_id -> Integer,
    }
}

diesel::table! {
    melonbooks_product_tag (product_id, tag_id) {
        product_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    melonbooks_skip_product (id) {
        id -> Integer,
        date_added -> Timestamp,
        url -> Text,
    }
}

diesel::table! {
    melonbooks_skip_product_artist (skip_product_id, artist_name) {
        skip_product_id -> Integer,
        artist_name -> Text,
    }
}

diesel::table! {
    melonbooks_tag (id) {
        id -> Integer,
        date_added -> Timestamp,
        tag -> Text,
    }
}

diesel::table! {
    melonbooks_title_skip_sequence (id) {
        id -> Integer,
        date_added -> Timestamp,
        sequence -> Text,
    }
}

diesel::joinable!(amiami_product -> amiami_category (category_id));
diesel::joinable!(melonbooks_product -> melonbooks_category (category_id));
diesel::joinable!(melonbooks_product_artist -> melonbooks_artist (artist_id));
diesel::joinable!(melonbooks_product_artist -> melonbooks_product (product_id));
diesel::joinable!(melonbooks_product_flag -> melonbooks_flag (flag_id));
diesel::joinable!(melonbooks_product_flag -> melonbooks_product (product_id));
diesel::joinable!(melonbooks_product_tag -> melonbooks_product (product_id));
diesel::joinable!(melonbooks_product_tag -> melonbooks_tag (tag_id));
diesel::joinable!(melonbooks_skip_product_artist -> melonbooks_skip_product (skip_product_id));

diesel::allow_tables_to_appear_in_same_query!(
    amiami_category,
    amiami_product,
    melonbooks_artist,
    melonbooks_category,
    melonbooks_flag,
    melonbooks_product,
    melonbooks_product_artist,
    melonbooks_product_flag,
    melonbooks_product_tag,
    melonbooks_skip_product,
    melonbooks_skip_product_artist,
    melonbooks_tag,
    melonbooks_title_skip_sequence,
);
