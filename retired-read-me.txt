# Journey
[![CI](https://github.com/CFdefense/Journey/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/CFdefense/Journey/actions/workflows/ci.yml)

[Public Api Docs](https://cfdefense.github.io/Journey/)

[Backend Code Cov](https://cfdefense.github.io/Journey/tarpaulin-report.html)

[Frontend Code Cov](https://cfdefense.github.io/Journey/frontend-codecov/index.html)

Travel agent that plans each day during your travel. The website creates an itinerary based on your location. Users may make changes or ask the agent to make changes to the itinerary.
## Team
* Ellie Knapp - Project Manager
* Christian Farrell - Software Engineer
* Nick Longo - Software Engineer
* Peter Arvanitis - Software Engineer
* Ethan Morton - Software Engineer
## Shared Google Drive
[click here](https://drive.google.com/drive/folders/1qaL4QgcQqS9PJ1wRcjRkr2MEaM80OV9i)
## Brightspace
[click here](https://brightspace.marist.edu/d2l/home/57958)
## Build Project
### Environment Variables
Make a `.env` in the root directory with these variables (example values are provided)
```
POSTGRES_USER="user"
POSTGRES_PASSWORD="password"
POSTGRES_DB="travelagent"
DATABASE_URL="postgres://user:password@localhost:5431/travelagent"
API_BASE_URL="http://localhost:3001"
FRONTEND_URL="http://localhost:5173"
BIND_ADDRESS="0.0.0.0:3001"
OPENAI_API_KEY="" # your key goes here
RUST_LOG="warn,Capping2025=debug"
```
Make a `.env` in the /frontend directory with these variables
```
VITE_API_BASE_URL="http://localhost:3001"
```
### Database
Install docker. You need an active connection to the database to compile. This brings the db container up.
```sh
# Windows
docker-compose up -d

# Linux (make sure docker.service and docker.socket are active and running)
sudo docker compose up -d
```
Install Postgres SQL CLI and login to database
```sh
psql -h localhost -p 5431 -U postgres -d travelagent
```
Run migrations (in psql command line - should have prompt `travelagent=# `)
```sql
\i migrations/01_migration_script.sql
```
Kill database (after you're done)
```sh
# Windows
# TODO: put windows command here

# Linux
sudo docker compose down db
```
### TypeScript
Install node and npm, then install the dependencies. All npm commands should be done from `./frontend`.
```sh
cd ./frontend
npm ci
```
Launch frontend with hot reloading (requires running server and active db connection)
```sh
npm run dev
```
Build frontend for server to serve static files
```sh
npm run build
```
Run tests
```sh
npm run test
```
Run code coverage
```sh
npm run codecov
```
### Rust
Install Rust using your package manager (likely called `rustup`) or from their [website](https://rust-lang.org/tools/install/).

Run the server (requires active db connection)
```sh
cargo run
```
Run tests
```sh
cargo test
```
Run code coverage
```sh
# Install tarpaulin
cargo install cargo-tarpaulin

# Run code coverage
cargo tarpaulin
```