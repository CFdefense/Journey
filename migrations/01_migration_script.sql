-- In psql run \i migrations/migration_script.sql\
-- IMPORTANT RUN FROM THE ROOT OF THE REPO DIRECTORY
-- This will wipe the database

-- Drop in dependency order to avoid FK issues
DROP TABLE IF EXISTS event_list CASCADE;
DROP TABLE IF EXISTS itineraries CASCADE;
DROP TABLE IF EXISTS events CASCADE;
DROP TABLE IF EXISTS accounts CASCADE;
DROP TYPE IF EXISTS event_type CASCADE;

CREATE EXTENSION IF NOT EXISTS vector; -- Use PGVECTOR (kept for future use)

-- Enum for event types
CREATE TYPE event_type AS ENUM (
    'Concert',
    'Museum',
    'Restaurant',
    'Hike',
    'Festival',
    'Sports',
    'Other'
);

-- Accounts table
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL
);
CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    street_address VARCHAR(255) NOT NULL,
    postal_code INTEGER NOT NULL,
    city VARCHAR(255) NOT NULL,
    event_type event_type NOT NULL,
    event_description TEXT NOT NULL,
    event_name VARCHAR(255) NOT NULL
);

-- Itineraries table
CREATE TABLE itineraries (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    date TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- Event list table
CREATE TABLE event_list (
    id SERIAL PRIMARY KEY,
    itinerary_id INTEGER NOT NULL REFERENCES itineraries(id) ON DELETE CASCADE,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    time_of_day VARCHAR(64) NOT NULL
);