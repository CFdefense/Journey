# Capping2025
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
## Project Idea
Travel agent that plans each day during your travel. The website creates an itinerary based on your location. Users may make changes or ask the agent to make changes to the itinerary.
## Build Project
### Environment Variables
Make a `.env` in the root directory with these variables
```
POSTGRES_USER=""
POSTGRES_PASSWORD=""
POSTGRES_DB=""
DATABASE_URL=""
API_BASE_URL=""
FRONTEND_URL=""
BIND_ADDRESS=""
OPENAI_API_KEY=""
RUST_LOG=""
```
Make a `.env` in the /frontend directory with these variables
```
VITE_API_BASE_URL="http://localhost:3001"
```
### Database
You need an active connection to the database to compile. This brings the db container up.
```sh
# Windows
docker-compose up -d

# Unix (make sure docker.service and docker.socket are active and running)
sudo docker compose up -d
```
Login to database
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
# TODO

# Unix
sudo docker compose down db
```
### Rust
Install Rust using your package manager (likely called `rustup`) or from their [website](https://rust-lang.org/tools/install/).

Run the server
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