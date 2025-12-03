-- Migration: Create customers table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS customers (
    customer_id VARCHAR(32) PRIMARY KEY,
    customer_unique_id VARCHAR(32) NOT NULL,
    customer_zip_code_prefix VARCHAR(10) NOT NULL,
    customer_city VARCHAR(100) NOT NULL,
    customer_state VARCHAR(2) NOT NULL
);

CREATE INDEX idx_customers_zip_code_prefix ON customers(customer_zip_code_prefix);
CREATE INDEX idx_customers_city ON customers(customer_city);
CREATE INDEX idx_customers_state ON customers(customer_state);
