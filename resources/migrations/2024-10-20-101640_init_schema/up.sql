CREATE TABLE melonbooks_product (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    image_url TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    availability TEXT NOT NULL,
    price TEXT,
    circle TEXT,
    CONSTRAINT fk__melonbooks_product__category FOREIGN KEY (category_id) REFERENCES melonbooks_category,
    CONSTRAINT uk__melonbooks_product__url UNIQUE (url)
);

CREATE TABLE melonbooks_artist (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    name TEXT NOT NULL,
    following BOOLEAN NOT NULL,
    date_followed TIMESTAMP,
    CONSTRAINT uk__melonbooks_artist__name UNIQUE (name)
);

CREATE TABLE melonbooks_product_artist (
    product_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    PRIMARY KEY (product_id, artist_id),
    CONSTRAINT fk__melonbooks_product_artist__product FOREIGN KEY (product_id) REFERENCES melonbooks_product (id),
    CONSTRAINT fk__melonbooks_product_artist__artist FOREIGN KEY (artist_id) REFERENCES melonbooks_artist (id)
);

CREATE TABLE melonbooks_category (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    category TEXT NOT NULL,
    CONSTRAINT uk__melonbooks_category UNIQUE (category)
);

CREATE TRIGGER tr__melonbooks_remove_unused_categories AFTER DELETE ON melonbooks_product
BEGIN
    DELETE FROM melonbooks_category
    WHERE id = OLD.category_id
      AND NOT EXISTS (SELECT 1 FROM melonbooks_product WHERE category_id = OLD.category_id);
end;

CREATE TABLE melonbooks_flag (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    flag TEXT NOT NULL,
    CONSTRAINT uk__melonbooks_flag__flag UNIQUE (flag)
);

CREATE TABLE melonbooks_product_flag (
    product_id INTEGER NOT NULL,
    flag_id INTEGER NOT NULL,
    PRIMARY KEY (product_id, flag_id),
    CONSTRAINT fk__melonbooks_product_flag__product FOREIGN KEY (product_id) REFERENCES melonbooks_product (id) ON DELETE CASCADE,
    CONSTRAINT fk__melonbooks_product_flag__flag FOREIGN KEY (flag_id) REFERENCES melonbooks_flag (id) ON DELETE CASCADE
);

CREATE TRIGGER tr__melonbooks_remove_unused_flags AFTER DELETE ON melonbooks_product_flag
BEGIN
    DELETE FROM melonbooks_flag
    WHERE id = OLD.flag_id
      AND NOT EXISTS (SELECT 1 FROM melonbooks_product_flag WHERE flag_id = OLD.flag_id);
end;

CREATE TABLE melonbooks_tag (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    tag TEXT NOT NULL,
    CONSTRAINT uk__melonbooks_tag__tag UNIQUE (tag)
);

CREATE TABLE melonbooks_product_tag (
    product_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (product_id, tag_id),
    CONSTRAINT fk__melonbooks_product_tag__product FOREIGN KEY (product_id) REFERENCES melonbooks_product (id) ON DELETE CASCADE,
    CONSTRAINT fk__melonbooks_product_tag__tag FOREIGN KEY (tag_id) REFERENCES melonbooks_tag (id) ON DELETE CASCADE
);

CREATE TRIGGER tr__melonbooks_remove_unused_tags AFTER DELETE ON melonbooks_product_tag
BEGIN
    DELETE FROM melonbooks_tag
    WHERE id = OLD.tag_id
      AND NOT EXISTS (SELECT 1 FROM melonbooks_product_tag WHERE tag_id = OLD.tag_id);
end;

CREATE TABLE melonbooks_skip_product (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    url TEXT NOT NULL,
    CONSTRAINT uk__melonbooks_skip_product__url UNIQUE (url)
);

CREATE TABLE melonbooks_skip_product_artist (
    skip_product_id INTEGER NOT NULL,
    artist_name TEXT NOT NULL,
    PRIMARY KEY (skip_product_id, artist_name),
    CONSTRAINT fk__melonbooks_skip_product_artist__skip_product FOREIGN KEY (skip_product_id) REFERENCES melonbooks_skip_product (id) ON DELETE CASCADE
);

CREATE TABLE melonbooks_title_skip_sequence (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    sequence TEXT NOT NULL,
    CONSTRAINT uk__melonbooks_title_skip_sequence__sequence UNIQUE (sequence)
);