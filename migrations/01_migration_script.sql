-- Migration Script can be run using the cli argument 'migrate'
-- In psql run \i migrations/migration_script.sql\
-- IMPORTANT RUN FROM THE ROOT OF THE REPO DIRECTORY
-- This will wipe the database

DROP TABLE IF EXISTS users CASCADE;

CREATE EXTENSION IF NOT EXISTS vector; -- Use PGVECTOR

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    ai_credits INT NOT NULL DEFAULT 10
);