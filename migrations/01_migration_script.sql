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
DROP TYPE IF EXISTS event_period CASCADE;

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
    'Afternoon',
    'Evening'
);

CREATE TYPE event_period AS (
	open_date DATE,
	open_truncated BOOLEAN,
	open_day INTEGER,
	open_hour INTEGER,
	open_minute INTEGER,
	close_date DATE,
	close_truncated BOOLEAN,
	close_day INTEGER,
	close_hour INTEGER,
	close_minute INTEGER
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
    --places.displayName
    event_name VARCHAR(255) NOT NULL,
    --places.editorialSummary
    event_description TEXT,
    street_address VARCHAR(255),
    city VARCHAR(255),
    country VARCHAR(255),
    postal_code INTEGER,
    coords VARCHAR(255),
    --places.primaryType
    event_type VARCHAR(255),
    user_created BOOLEAN NOT NULL DEFAULT FALSE,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    --Timestamp of event location
    hard_start TIMESTAMP WITHOUT TIME ZONE,
    --Timestamp of event location
    hard_end TIMESTAMP WITHOUT TIME ZONE,
    timezone VARCHAR(255),
    --remaining places fields
    place_id VARCHAR(255),
    wheelchair_accessible_parking BOOLEAN,
    wheelchair_accessible_entrance BOOLEAN,
    wheelchair_accessible_restroom BOOLEAN,
    wheelchair_accessible_seating BOOLEAN,
    serves_vegetarian_food BOOLEAN,
    price_level INTEGER,
    utc_offset_minutes INTEGER,
    website_uri VARCHAR(255),
    types VARCHAR(255),
    photo_name TEXT,
    photo_width INTEGER,
    photo_height INTEGER,
    photo_author VARCHAR(255),
    photo_author_uri VARCHAR(255),
    photo_author_photo_uri VARCHAR(255),
    weekday_descriptions TEXT,
    secondary_hours_type INTEGER,
    next_open_time TIMESTAMP WITHOUT TIME ZONE,
    next_close_time TIMESTAMP WITHOUT TIME ZONE,
    open_now BOOLEAN,
    periods event_period[] NOT NULL DEFAULT ARRAY[]::event_period[],
    special_days DATE[] NOT NULL DEFAULT ARRAY[]::DATE[]
);

CREATE TABLE chat_sessions (
	id SERIAL PRIMARY KEY,
	account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
	title VARCHAR(255) NOT NULL
);

-- Itineraries table
CREATE TABLE itineraries (
    id SERIAL PRIMARY KEY,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    -- date of destination's local timezone
    start_date DATE NOT NULL,
    -- date of destination's local timezone
    end_date DATE NOT NULL,
    chat_session_id INTEGER REFERENCES chat_sessions(id) ON DELETE SET NULL,
    saved BOOLEAN NOT NULL,
    title VARCHAR(255) NOT NULL,
    -- Array of event IDs that are unassigned to any specific time slot
    unassigned_event_ids INTEGER[] NOT NULL DEFAULT ARRAY[]::INTEGER[]
);

-- Event list table
CREATE TABLE event_list (
    id SERIAL PRIMARY KEY,
    itinerary_id INTEGER NOT NULL REFERENCES itineraries(id) ON DELETE CASCADE,
    -- event_id can be NULL for placeholder entries that preserve empty days
    event_id INTEGER REFERENCES events(id) ON DELETE CASCADE,
    time_of_day time_of_day NOT NULL,
    -- date of destination's local timezone
    date DATE NOT NULL,
    block_index INTEGER
);

CREATE TABLE messages (
	id SERIAL PRIMARY KEY,
	chat_session_id INTEGER NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
	itinerary_id INTEGER REFERENCES itineraries(id) ON DELETE SET NULL,
	is_user BOOLEAN NOT NULL,
	-- UTC
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
INSERT INTO events (id, street_address, postal_code, city, country, event_type, event_description, event_name, user_created, account_id, hard_start, hard_end)
VALUES (1, '1114 Shannon Ln', 17013, 'Carlisle', 'USA', 'Hike', 'A beautiful stroll along a river in this cute small town.', 'Family Walking Path', FALSE, NULL, NULL, NULL),
(2, '35 Campus Court', 12601, 'Poughkeepsie', 'USA', 'Restaurant', 'Local Italian restaurant known for its authentic pasta and upscale dining.', 'Cosimos', FALSE, NULL, NULL, NULL),
(3, '200 E 42nd St', 10017, 'New York', 'USA', 'Museum', 'World famous art museum with a focus on modern works, including Starry Starry Night by VanGough.', 'Museum of Modern Art- MoMA', FALSE, NULL, NULL, NULL),
(4, '1 S Broad St', 19107, 'Philadelphia', 'USA', 'Concert', 'Music center which hosts local and national bands.', 'Jazz night at Broad Street', FALSE, NULL, NULL, NULL),
(5, '2 Citizens Bank Way', 19148, 'Philadelphia', 'USA', 'Sports', 'A Phillies baseball game is a must-do for locals and visitors alike.', 'Phillies Baseball Game', FALSE, NULL, '2025-11-05 13:00', '2025-11-05 16:00'),
(6, '5250 S Park Dr', 60615, 'Chicago', 'USA', 'Festival', 'Annual music festival with the biggest names in pop and indie scenes.', 'LollaPalooza', FALSE, NULL, NULL, NULL),
(7, '1 Rue de la Seine', 00000, 'Paris', 'France', 'Museum', 'Explore the beautiful landmark of Paris.', 'Eiffel Tower', FALSE, NULL, NULL, NULL),
(8, '3 Rue de la Museu', 00000, 'Paris', 'France', 'Museum', 'Wander the halls of the world famous art museum.', 'le Louvre', FALSE, NULL, '2025-11-05 08:00', NULL);

UPDATE events
SET 
    photo_name = 'places/ChIJWTGPjmaAhYARxz6l1hOj92w/photos/AWn5SU5Er9gWi5jilVeHqFVvSEXMwCeX-mlXTiRpSoacmgGi6eJk021kXT7TSPe1PbFEj8Oe-5JYpMk5jc2e6A1zHf76i45Mg2JcTqhmihTAWyW4b2vixtaHLq7GCuxQicTvwvM65I1qTt9sAEiMW2k1jPvHiU5qL7R8BN2Ltqfo_I0zTcUrJyup6Pdsd0FLpDAm7ia5B-qaaw80x6guoa0r7uoiFGkyJeB3zqXIDKsP5q25HgPb6HMIVtiTM7Gf0WEU6hnhpg9Admq3h-WARF0hTmQQqCg_J9C-dRrzuu279uwDaw',
    photo_width = 3850,
    photo_height = 2991,
    photo_author = 'Ferry Building',
    photo_author_uri = 'https://maps.google.com/maps/contrib/116985961358506551836',
    photo_author_photo_uri = 'https://lh3.googleusercontent.com/a/ACg8ocKXdGznTApl3bUhp8fBHvPlpZCrTK_fq8h1PliUWm9Q7aJDBg=s100-p-k-no-mo'
WHERE id = 1;

UPDATE events
SET 
    photo_name = 'places/ChIJ1dIqa2OAhYAREimtEtfBLyc/photos/AWn5SU64-FqzTMnkxlg9F-trzmWqkl2f0y_bCRBqLjkP8qLzGD8JTaThyGhmIzilkbOptFyHfbKj11kyTbga2m9scTSyhPBYd6TvsoDUoL6UTzXn5pKpUcbist3SB1ccAE5H7Wt-d_C2ycfmqAgHudsYBNOJLMtVV5Ij559zA4KoA_hOuDoNBaNI9pKwRharo8uKaAKDwDDefIpQki7VLigFZ7R0dvzhIl57MIdLoRgEoCQvELCv9KjE-9cORp7X1qsKG37QbudAJzgmmN8xwlf9XpCc57YTOABuM8dtS4WmNYOXEEyoKpNZptwOBh6RkiKhOY7Uhu8gwBXuVv9PLAHr6aa3ZOFhnaPCewvAdwfA0yd3insxeH12f80VdQyJ5nx9umKNVZHlEG-d7Tek5cc3k3UDIgdc2gbirc80wXn_AUyMkHDU',
    photo_width = 3024,
    photo_height = 4032,
    photo_author = 'Barry Yen',
    photo_author_uri = 'https://maps.google.com/maps/contrib/102121354402180572298',
    photo_author_photo_uri = 'https://lh3.googleusercontent.com/a-/ALV-UjUoOkFbUTTB1UQ96TqA-K8eeYRUXWzmR4APPQ4DMBPzzj8dPygh=s100-p-k-no-mo'
WHERE id = 4;

-- Ensure the events id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('events', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM events), 1)
);

-- Create chat sessions
INSERT INTO chat_sessions (account_id, title)
SELECT id, 'Dummy Chat 1' FROM accounts;
INSERT INTO chat_sessions (account_id, title)
SELECT id, 'Dummy Chat 2' FROM accounts;

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
(6, 4, 6, 'Afternoon', '2025-07-15'), --Lolla
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