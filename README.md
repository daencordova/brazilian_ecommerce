# Axum & SQLx Web Server Project

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/daencordova/brazilian_ecommerce.svg?style=social&label=Star)](https://github.com/daencordova/brazilian_ecommerce)

A high-performance, asynchronous REST API built with Rust, Axum, and PostgreSQL. This project demonstrates production-grade patterns including Dependency Injection, Graceful Shutdown, Structured Logging, and Type-safe Database interactions.

## Features

* **Web Framework**: Built on Axum for ergonomic and modular routing.
* **High Performance:** Built on `tokio` and `hyper`, leveraging Rust's performance capabilities.
* **Database**: Uses SQLx for compile-time checked SQL queries and async PostgreSQL interaction.
* **Compile-Time SQL Safety:** Uses `sqlx` to check all database queries during compilation, preventing runtime SQL errors.
* **Database Migrations:** Managed schema changes using the `sqlx-cli`.
* **Architecture**: Clean Architecture patterns using Repository and Service layers with Dependency Injection.
* **Validation**: Request payload validation using the validator crate.
* **Observability**: Structured logging and instrumentation via tracing and tracing-subscriber.
* **Resilience**: Implements Graceful Shutdown to handle signal interruptions (SIGTERM/Ctrl+C) safely.
* **Pagination**: standardized pagination logic for list endpoints.
* **Modular Routing:** Clean, easy-to-read routing definitions using the Axum framework.
* **Environment Configuration:** Secure configuration via `.env` files using `dotenvy`.
* **CORS**: Configuration with flexible options.

## Getting Started

### Prerequisites

You need the following installed locally:

* **Rust:** Use `rustup` to install the latest stable version.
* **PostgreSQL:** A running instance of PostgreSQL (or adjust dependencies for MySQL/SQLite).
* **SQLx CLI:** The command-line tool for migrations.
    ```bash
    cargo install sqlx-cli --no-default-features --features postgres
    ```

### Installation & Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/daencordova/brazilian_ecommerce.git
    cd brazilian_ecommerce
    ```
    
    #### Project Structure
    ```
    ├── src/
    │   ├── config.rs
    │   ├── error.rs
    │   ├── handlers.rs
    │   ├── main.rs
    │   ├── models.rs
    │   ├── repositories.rs
    │   ├── services.rs
    │   └── state.rs
    ├── migrations           # SQL migration files
    ├── .env                 # Environment variables
    ├── .env.example         # Template example file
    ├── Cargo.toml           # Dependencies
    └── README.md            # Documentation
    ```

2.  **Configure Environment:**
    Create a file named `.env` in the project root:
    
    ```bash
    touch .env
    ```
    
    ```env
    # .env
    DATABASE_URL=postgres://username:password@localhost:port/database_name
    PORT=3000
    RUST_LOG=info
    ```
    
    #### CORS Configuration
    
    ```env
    # Allow all origins (not recommended for production)
    ALLOWED_ORIGINS="*"
    ALLOW_CREDENTIALS=false
    MAX_AGE=3600  
    ```

3.  **Setup Database & Migrations:**
    Use the SQLx CLI to set up your database and run all migrations.

    ```bash
    # 1. Create the database (if it doesn't exist)
    sqlx database create
    # 2. Run all pending migrations
    sqlx migrate run
    ```

4.  **Prepare SQLx for Offline Building (Recommended):**
    Generate the `sqlx-data.json` file which caches query checks, allowing you to compile without a live database connection.

    ```bash
    cargo sqlx prepare
    ```

### Running the Application

To start the development server:

```bash
cargo run
```

The server will be available at http://127.0.0.1:3000/customers

### Usage Examples
#### Create a new Customer
Endpoint: POST

  - `/customers`

```bash
curl -X POST http://localhost:3000/customers \
  -H "Content-Type: application/json" \
  -H "Origin: http://localhost:3000" \
  -d '{    
    "customer_id": "06b8999e2fba1a1fbc88172c00ba8bc7",
    "customer_unique_id": "861eff4711a542e4b93843c6dd7febb0",
    "customer_zip_code_prefix": "14409",
    "customer_city": "franca",
    "customer_state": "SP"
  }'
```

#### Get all Customers
Endpoint: GET 

  - `/customers?city=Sao%20Paulo`
  - `/customers?state=SP`
  - `/customers?city=Rio%20de%20Janeiro&state=RJ&page=1&page_size=10`

```bash
curl -X GET http://localhost:3000/customers?page=1&page_size=10 \
   -H "Content-Type: application/json"
   -H "Origin: http://localhost:3000"
```

```json
{
  "data": [ ... ],
  "meta": {
    "total_records": 50,
    "current_page": 1,
    "page_size": 10,
    "total_pages": 5
  }
}
``` 
   
#### Get a Customer by ID
Endpoint: GET

  - `/customers/{id}`

```bash
curl -X GET http://localhost:3000/customers/06b899... \
    -H "Content-Type: application/json"
```

```json
{
  "customer_id":"06b8999e2fba1a1fbc88172c00ba8bc7",
  "customer_unique_id":"861eff4711a542e4b93843c6dd7febb0",
  "customer_zip_code_prefix":"14409",
  "customer_city":"franca",
  "customer_state":"SP"
}
```

### Testing

To run unit and integration tests (if implemented):

```bash
cargo test
```
