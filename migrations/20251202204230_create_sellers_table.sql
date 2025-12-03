-- Migration: Create sellers table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS sellers (
    seller_id VARCHAR(32) PRIMARY KEY,
    seller_zip_code_prefix VARCHAR(10) NOT NULL,
    seller_city VARCHAR(100) NOT NULL,
    seller_state VARCHAR(2) NOT NULL
);

CREATE INDEX idx_sellers_zip_code_prefix ON sellers(seller_zip_code_prefix);
CREATE INDEX idx_sellers_city ON sellers(seller_city);
CREATE INDEX idx_sellers_state ON sellers(seller_state);
