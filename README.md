Dashboard

A personal web dashboard to bring all your favorite websites together in one place.

Dashboard is a web application built with Rust, designed to provide a centralized entry point for your favorite websites and services.

Features
Centralized dashboard — Keep your favorite websites accessible from a single interface.
Rust backend — Built with Rust for performance, reliability, and memory safety.
PostgreSQL database — Persistent data storage managed through Diesel ORM.
Database migrations — Versioned schema changes to keep the database structure consistent.
Web API documentation — API documentation powered by OpenAPI and Swagger UI.
Docker support — Build and run the application and its supporting services using Docker Compose.
Asynchronous operations — Uses Tokio and asynchronous Rust libraries to handle concurrent operations.
Tech Stack
Technology	Purpose
Rust	Backend programming language
Actix Web	Web framework
PostgreSQL	Relational database
Diesel	ORM and database query interface
Tokio	Asynchronous runtime
Swagger UI	Interactive API documentation
Docker	Containerization and deployment
Why PostgreSQL?

PostgreSQL was chosen after considering SQLite, MySQL, and MariaDB, with a focus on the project's requirements and compatibility with the Rust ecosystem.

SQLite

SQLite is an excellent embedded database, particularly for lightweight applications that do not need a separate database server. However, its embedded architecture and feature set are not the best fit for every application.

For Dashboard, a dedicated relational database provides a more suitable foundation for persistent application data and future growth.

MySQL and MariaDB

MySQL and MariaDB are mature relational database systems. However, choosing a database is not just about its capabilities: compatibility with the application's database layer and ORM matters too.

Diesel provides a well-defined set of database backends. Its MySQL backend does not provide a separate, dedicated MariaDB backend, so MariaDB-specific compatibility is not the target of this project.

Supporting a different database backend would also require configuring and validating the corresponding database integration.

Why PostgreSQL?

PostgreSQL is the natural choice for this project because:

Diesel integration: The project uses Diesel's PostgreSQL backend directly, along with diesel-async for asynchronous database access.
Rich relational features: PostgreSQL offers constraints, transactions, indexing, and advanced SQL capabilities.
Reliable persistence: It provides a mature foundation for storing application data consistently.
Room to grow: Its feature set can accommodate more complex data relationships and queries as the application evolves.
Simpler architecture: Using PostgreSQL as the supported database avoids maintaining multiple database configurations and compatibility paths.

Ultimately, PostgreSQL was selected because it meets the project's needs and integrates directly with the Rust database stack.

Prerequisites

To run Dashboard locally, you will need:

Git
Docker and Docker Compose, for the containerized setup

For development outside Docker, you will also need:

Rust, with a toolchain compatible with the project's Rust edition
PostgreSQL and the development libraries required by Diesel and OpenSSL
Getting Started
1. Clone the repository
git clone https://github.com/Armel-P/Dashboard.git
cd Dashboard

2. Configure environment variables

Create a local environment file from the provided example:

cp .env.example .env


Review .env and update the values to match your environment.

Important variables include:

Variable	Description
SERVER_PORT	Port used by the web server
SERVER_HOST	Network interface used by the web server
POSTGRES_PASSWORD	PostgreSQL password
DATABASE_PORT	PostgreSQL port
DATABASE_URL	Database connection URL
SMTP_PORT	SMTP server port
SMTP_FROM	Sender address for outgoing email
SECRET	Application secret
TOKEN_ENCRYPTION_KEY	Key used for token encryption
RUST_LOG	Logging verbosity

The example configuration also contains SoundCloud client credentials for integrations that require them.

Security: Replace development placeholders with strong, private values. Never commit your real .env file, production credentials, or encryption keys to version control.

3. Build the application

Build the Docker services using the provided Compose configuration:

docker compose -f docker-compose.yml build

4. Start Dashboard

Start the application and its configured supporting services:

docker compose -f docker-compose.yml up -d


Check the container status and logs if the application does not start as expected:

docker compose -f docker-compose.yml ps
docker compose -f docker-compose.yml logs -f


The example environment configures the web server on port 8080. Once the application is running, try opening:

http://localhost:8080


The actual availability of the interface depends on the services starting successfully and the current application routes.

For more information about the containerized setup, see Docker.md.

Database Migrations

Dashboard uses Diesel migrations to manage changes to the PostgreSQL schema.

Migration files are located in the migrations/ directory. They allow database changes to be versioned alongside the application code.

When introducing a schema change:

Create a migration using the project's Diesel tooling.
Add the SQL required to apply the change.
Provide a corresponding rollback where appropriate.
Test the migration against PostgreSQL before deploying.

The database connection must be configured correctly before migrations can run.

Project Structure
Dashboard/
├── migrations/          # Database migrations
├── src/                 # Rust backend source code
├── web/                 # Web application assets or frontend
├── schema/              # Database schema definitions
├── .github/workflows/   # GitHub Actions workflows
├── .env.example         # Example environment configuration
├── Cargo.toml           # Rust dependencies and project metadata
├── diesel.toml          # Diesel configuration
├── Dockerfile           # Docker build configuration
├── docker-compose.yml   # Multi-container configuration
├── Docker.md            # Docker usage instructions
└── README.md            # Project documentation

Development

To build the Rust application locally:

cargo build


To run the application directly:

cargo run


Make sure PostgreSQL is running and the required environment variables are configured before starting the application.

For a production build:

cargo build --release

API Documentation

Dashboard uses Utoipa and Swagger UI to provide API documentation.

When the documentation route is enabled, it can be used to inspect available endpoints and test requests interactively. Consult the application's route configuration to determine the exact URL.

Contributing

Contributions, bug reports, and suggestions are welcome.

Fork the repository.
Create a branch for your changes.
Make and test your changes.
Open a pull request describing the changes.

Please keep database migrations, configuration changes, and application code consistent with one another.

License

No license information is specified in this README. Please consult the repository for the applicable license before redistributing or modifying the project.
