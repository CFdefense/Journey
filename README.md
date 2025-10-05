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
## Build Project (WIP)
In dev we can use react as a backend to make changes to UI/UX, and that should hopefully make development easier.

In prod, we would compile the react files into html/css/js files (or however it works) and just serve them with our backend (axum).

## Database
Bring container up
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
Kill database
```sh
# Windows
# TODO

# Unix
sudo docker compose down db
```
## Development
Install Rust using your package manager or from their [website](https://rust-lang.org/tools/install/).

Make a `.env` with these variables
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
Run the project
```sh
cargo run
```
Run the linter
```sh
cargo clippy --all-targets
```
Run the tests
```sh
cargo test --all-targets
```
Run code coverage
```sh
# Install tarpaulin
cargo install cargo-tarpaulin

# Run code coverage
cargo tarpaulin --all-targets --fail-under 80 --out Stdout Html --output-dir ./tmp
```