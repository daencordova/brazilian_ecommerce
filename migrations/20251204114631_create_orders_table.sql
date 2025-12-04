-- Migration: Create orders table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS orders (
    order_id VARCHAR(32) PRIMARY KEY,
    customer_id VARCHAR(32) NOT NULL,
    order_status VARCHAR(12) NOT NULL,
    order_purchase_timestamp TIMESTAMP NOT NULL,
    order_approved_at TIMESTAMP NOT NULL,
    order_delivered_carrier_date TIMESTAMP,
    order_delivered_customer_date TIMESTAMP,
    order_estimated_delivery_date TIMESTAMP NOT NULL,
    CONSTRAINT fk_seller_orders
        FOREIGN KEY (customer_id)
        REFERENCES customers(customer_id)
        ON DELETE CASCADE
        ON UPDATE NO ACTION
);

CREATE INDEX idx_orders_status ON orders(order_status);
CREATE INDEX idx_orders_purchase_timestamp ON orders(order_purchase_timestamp);
CREATE INDEX idx_orders_approved_at ON orders(order_approved_at);
CREATE INDEX idx_orders_delivered_carrier_date ON orders(order_delivered_carrier_date);
CREATE INDEX idx_orders_delivered_customer_date ON orders(order_delivered_customer_date);
CREATE INDEX idx_orders_estimated_delivery_date ON orders(order_estimated_delivery_date);
