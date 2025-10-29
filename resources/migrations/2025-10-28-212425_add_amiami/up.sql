CREATE TABLE amiami_product (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    image_url TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    maker TEXT NOT NULL,
    full_price INTEGER NOT NULL,
    min_price INTEGER NOT NULL,
    release_date DATE NOT NULL,
    availability TEXT NOT NULL,
    CONSTRAINT fk__amiami_product__category FOREIGN KEY (category_id) REFERENCES amiami_category,
    CONSTRAINT uk__amiami_product__url UNIQUE (url)
);

CREATE TABLE amiami_category (
    id INTEGER PRIMARY KEY NOT NULL,
    date_added TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    category TEXT NOT NULL,
    following BOOLEAN NOT NULL,
    CONSTRAINT uk__amiami_category UNIQUE (category)
);