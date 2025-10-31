-- In psql run \i migrations/01_migration_script.sql\
-- IMPORTANT RUN FROM THE ROOT OF THE REPO DIRECTORY
-- This will wipe the database

-- Drop in dependency order to avoid FK issues
DROP TABLE IF EXISTS accounts CASCADE;
DROP TABLE IF EXISTS events CASCADE;
DROP TABLE IF EXISTS chat_sessions CASCADE;
DROP TABLE IF EXISTS itineraries CASCADE;
DROP TABLE IF EXISTS event_list CASCADE;
DROP TABLE IF EXISTS messages CASCADE;
DROP TYPE IF EXISTS risk_tolerence CASCADE;
DROP TYPE IF EXISTS budget_bucket CASCADE;
DROP TYPE IF EXISTS time_of_day CASCADE;

CREATE EXTENSION IF NOT EXISTS vector; -- Use PGVECTOR (kept for future use)

CREATE TYPE risk_tolerence AS ENUM (
    'ChillVibes',
    'LightFun',
    'Adventurer',
    'RiskTaker'
);

CREATE TYPE budget_bucket AS ENUM (
    'VeryLowBudget',
    'LowBudget',
    'MediumBudget',
    'HighBudget',
    'LuxuryBudget'
);

CREATE TYPE time_of_day AS ENUM (
    'Morning',
    'Noon',
    'Afternoon',
    'Evening'
);

-- Accounts table
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    budget_preference budget_bucket,
    risk_preference risk_tolerence,
    food_allergies TEXT NOT NULL DEFAULT '',
    disabilities TEXT NOT NULL DEFAULT ''
);

-- Events table
CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    street_address VARCHAR(255) NOT NULL,
    postal_code INTEGER NOT NULL,
    city VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_description TEXT NOT NULL,
    event_name VARCHAR(255) NOT NULL,
    user_created BOOLEAN NOT NULL DEFAULT FALSE,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    hard_start TIMESTAMP [ (0) ] WITHOUT TIME ZONE,
    hard_end TIMESTAMP [ (0) ] WITHOUT TIME ZONE
);

CREATE TABLE chat_sessions (
	id SERIAL PRIMARY KEY,
	account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE
);

-- Itineraries table
CREATE TABLE itineraries (
    id SERIAL PRIMARY KEY,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    chat_session_id INTEGER REFERENCES chat_sessions(id) ON DELETE SET NULL,
    saved BOOLEAN NOT NULL,
    title VARCHAR(255) NOT NULL
);

-- Event list table
CREATE TABLE event_list (
    id SERIAL PRIMARY KEY,
    itinerary_id INTEGER NOT NULL REFERENCES itineraries(id) ON DELETE CASCADE,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    time_of_day time_of_day NOT NULL,
    date DATE NOT NULL
);

CREATE TABLE messages (
	id SERIAL PRIMARY KEY,
	chat_session_id INTEGER NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
	itinerary_id INTEGER REFERENCES itineraries(id) ON DELETE SET NULL,
	is_user BOOLEAN NOT NULL,
	timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL,
	text TEXT NOT NULL
);

------- Dummy data to test ---------
--Accounts
-- CF: Password is "whatisrust"
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities)
VALUES (1, 'ellieknapp@gmail.com', 'ihateHR', 'ellie', 'knapp', 'VeryLowBudget', 'Adventurer', 'Vegan', 'Blind,Deaf'),
(2, 'nicklongo@gmail.com', 'iwannabeHR', 'nick', 'longo', 'LowBudget', 'RiskTaker', 'Gluten Free,Hates Cheese', ''),
(3, 'christianfarrell@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$boV4nNLYxj5VTn0yRZaQZg$dRSI/RHmPlgxGnKr/Q/bkBt1XRFjWx21FDVjbHKWJZs', 'christian', 'farrell', 'MediumBudget', 'ChillVibes', 'Tree Nuts,Peanuts', 'Mobility Impaired - Wheelchair'),
(4, 'ethanmorton@gmail.com', 'fakingmyankle', 'ethan', 'morton', 'HighBudget', 'LightFun', 'Dairy,Eggs,Quinoa', 'Bad Ankle'),
(5, 'peterarvanitis@gmail.com', 'ihateHR', 'peter', 'arvanitis', 'LuxuryBudget', 'ChillVibes','Dairy,Pine nuts,Kiwi','Lazy');

-- Ensure the accounts id sequence matches the max(id) after manual inserts
SELECT setval(
    pg_get_serial_sequence('accounts', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM accounts), 1)
);

-- Events
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name, hard_start)
VALUES (1, '1114 Shannon Ln', 17013, 'Carlisle', 'Hike', 'A beautiful stroll along a river in this cute small town.', 'Family Walking Path'),
(2, '35 Campus Court', 12601, 'Poughkeepsie', 'Restaurant', 'Local Italian restaurant known for its authentic pasta and upscale dining.', 'Cosimos'),
(3, '200 E 42nd St', 10017, 'New York', 'Museum', 'World famous art museum with a focus on modern works, including Starry Starry Night by VanGough.', 'Museum of Modern Art- MoMA'),
(4, '1 S Broad St', 19107, 'Philadelphia', 'Concert', 'Music center which hosts local and national bands.', 'Jazz night at Broad Street'),
(5, '1 Citizens Bank Way', 19148, 'Philadelphia', 'Sports', 'A Phillies baseball game is a must-do for locals and visitors alike.', 'Phillies Baseball Game', '2025-12-01 13:00'),
(6, '5250 S Park Dr', 60615, 'Chicago', 'Festival', 'Annual music festival with the biggest names in pop and indie scenes.', 'LollaPalooza'),
(7, '1 Rue de la Seine', 00000, 'Paris', 'Museum', 'Explore the beautiful landmark of Paris.', 'Eiffel Tower'),
(8, '3 Rue de la Museu', 00000, 'Paris', 'Museum', 'Wander the halls of the world famous art museum.', 'le Louvre');

-- Ensure the events id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('events', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM events), 1)
);

-- Create chat sessions
INSERT INTO chat_sessions (account_id)
SELECT id
FROM accounts;
INSERT INTO chat_sessions (account_id)
SELECT id
FROM accounts;

-- Ensure the chat session id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('chat_sessions', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM chat_sessions), 1)
);

-- Itineraries
INSERT INTO itineraries (id, account_id, is_public, start_date, end_date, chat_session_id, saved, title)
VALUES (1, 1, FALSE, '2025-11-05', '2025-11-10', 1, TRUE, 'NYC 11/5-10 2025'), -- Ellie
(2, 2, TRUE, '2025-10-12', '2025-10-14', 2, TRUE, 'Poughkeepsie 10/12-14 2025'), -- Nick
(3, 3, TRUE, '2025-12-01', '2025-12-08', 3, TRUE, 'Philly 12/1-8 2025'), --Christian
(4, 4, FALSE, '2025-07-15', '2025-07-17', 4, TRUE, 'Lollapalooza 7/15-17 2025'), -- Ethan
(5, 5, FALSE, '2025-08-15', '2025-08-28', 5, TRUE, 'Paris 8/15-28 2025'); -- Peter

-- Ensure the itineraries id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('itineraries', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM itineraries), 1)
);

-- Event List
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day, date)
-- Itinerary 1 (Nick to NYC + Poopsie)
VALUES (1, 2, 3, 'Morning', '2025-11-05'), -- MoMA
(2, 2, 2, 'Evening', '2025-10-12'), -- Cosimos
-- Itinerary 2 (Ellies Weekend in Carlisle)
(3, 1, 1, 'Afternoon', '2025-11-05'), -- Hike in Carlisle
-- Itinerary 3 (Christian exploring Philly)
(4, 3, 5, 'Afternoon', '2025-12-01'), -- Phillies Game
(5, 3, 4, 'Evening', '2025-12-01'), -- Jazz
-- Itinerary 4 (Ethan at Lollapalooza)
(6, 4, 6, 'Noon', '2025-07-15'), --Lolla
--Itinerary 5 (peter in paris)
(7, 5, 7, 'Morning', '2025-08-15'), --Eiffel Tower
(8, 5, 8, 'Afternoon', '2025-08-15'); --Lourve

-- Ensure the event_list id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('event_list', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM event_list), 1)
);

-- Create messages in each chat session
INSERT INTO messages (chat_session_id, is_user, timestamp, text)
SELECT id, TRUE, NOW(), 'make me an itinerary'
FROM chat_sessions;
INSERT INTO messages (chat_session_id, is_user, timestamp, text)
SELECT id, false, NOW(), 'no'
FROM chat_sessions;
INSERT INTO messages (chat_session_id, is_user, timestamp, text)
SELECT id, TRUE, NOW(), 'please?'
FROM chat_sessions;
INSERT INTO messages (chat_session_id, itinerary_id, is_user, timestamp, text)
VALUES (1, 1, FALSE, NOW(), 'ok'),
(2, 2, FALSE, NOW(), 'ok'),
(3, 3, FALSE, NOW(), 'ok'),
(4, 4, FALSE, NOW(), 'ok'),
(5, 5, FALSE, NOW(), 'ok'),
(6, 1, FALSE, NOW(), 'ok'),
(7, 2, FALSE, NOW(), 'ok'),
(8, 3, FALSE, NOW(), 'ok'),
(9, 4, FALSE, NOW(), 'ok'),
(10, 5, FALSE, NOW(), 'ok');

-- Ensure the message id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('messages', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM messages), 1)
);