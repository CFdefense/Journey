-- In psql run \i migrations/01_migration_script.sql\
-- IMPORTANT RUN FROM THE ROOT OF THE REPO DIRECTORY
-- This will wipe the database

-- Drop in dependency order to avoid FK issues
DROP TABLE IF EXISTS event_list CASCADE;
DROP TABLE IF EXISTS itineraries CASCADE;
DROP TABLE IF EXISTS events CASCADE;
DROP TABLE IF EXISTS accounts CASCADE;
DROP TYPE IF EXISTS event_type CASCADE;
DROP TYPE IF EXISTS risk_tolerence CASCADE;
DROP TYPE IF EXISTS budget_bucket CASCADE;

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

CREATE TYPE risk_tolerence AS ENUM (
    'Chill vibes',
    'Light Fun',
    'Adventurer',
    'Risk Taker'
);

CREATE TYPE budget_bucket AS ENUM (
    'Very low budget',
    'Low budget',
    'Medium budget',
    'High budget',
    'Luxury budget'
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
    food_allergies TEXT,
    disabilities TEXT
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

------- Dummy data to test ---------
--Accounts
-- CF: Password is "whatisrust"
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities) VALUES (1, 'ellieknapp@gmail.com', 'ihateHR', 'ellie', 'knapp', 'Very low budget', 'Adventurer', 'Vegan', 'Blind,Deaf');
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities) VALUES (2, 'nicklongo@gmail.com', 'iwannabeHR', 'nick', 'longo', 'Low budget', 'Risk Taker', 'Gluten Free,Hates Cheese', '');
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities) VALUES (3, 'christianfarrell@gmail.com', '$argon2id$v=19$m=19456,t=2,p=1$boV4nNLYxj5VTn0yRZaQZg$dRSI/RHmPlgxGnKr/Q/bkBt1XRFjWx21FDVjbHKWJZs', 'christian', 'farrell', 'Medium budget', 'Chill vibes', 'Tree Nuts,Peanuts', 'Mobility Impaired - Wheelchair'); 
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities) VALUES (4, 'ethanmorton@gmail.com', 'fakingmyankle', 'ethan', 'morton', 'High budget', 'Light Fun', 'Dairy,Eggs,Quinoa', 'Bad Ankle');
INSERT INTO accounts (id, email, password, first_name, last_name, budget_preference, risk_preference, food_allergies, disabilities) VALUES (5, 'peterarvanitis@gmail.com', 'ihateHR', 'peter', 'arvanitis', 'Luxury budget', 'Chill vibes','Dairy,Pine nuts,Kiwi','Lazy');

-- Ensure the accounts id sequence matches the max(id) after manual inserts
SELECT setval(
    pg_get_serial_sequence('accounts', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM accounts), 1)
);

-- Events
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (1, '1114 Shannon Ln', 17013, 'Carlisle', 'Hike', 'A beautiful stroll along a river in this cute small town.', 'Family Walking Path');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (2, '35 Campus Court', 12601, 'Poughkeepsie', 'Restaurant', 'Local Italian restaurant known for its authentic pasta and upscale dining.', 'Cosimos');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (3, '200 E 42nd St', 10017, 'New York', 'Museum', 'World famous art museum with a focus on modern works, including Starry Starry Night by VanGough.', 'Museum of Modern Art- MoMA');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (4, '1 S Broad St', 19107, 'Philadelphia', 'Concert', 'Music center which hosts local and national bands.', 'Jazz night at Broad Street');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (5, '1 Citizens Bank Way', 19148, 'Philadelphia', 'Sports', 'A Phillies baseball game is a must-do for locals and visitors alike.', 'Phillies Baseball Game');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (6, '5250 S Park Dr', 60615, 'Chicago', 'Festival', 'Annual music festival with the biggest names in pop and indie scenes.', 'LollaPalooza');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (7, '1 Rue de la Seine', 00000, 'Paris', 'Museum', 'Explore the beautiful landmark of Paris.', 'Eiffel Tower');
INSERT INTO events (id, street_address, postal_code, city, event_type, event_description, event_name) VALUES (8, '3 Rue de la Museu', 00000, 'Paris', 'Museum', 'Wander the halls of the world famous art museum.', 'le Louvre');

-- Ensure the events id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('events', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM events), 1)
);

-- Itineraries
INSERT INTO itineraries (id, account_id, is_public, date) VALUES (1, 1, FALSE, '2025-11-05 00:00:00'); -- Ellie
INSERT INTO itineraries (id, account_id, is_public, date) VALUES (2, 2, TRUE, '2025-10-12 00:00:00'); -- Nick
INSERT INTO itineraries (id, account_id, is_public, date) VALUES (3, 3, TRUE, '2025-12-01 00:00:00'); --Christian
INSERT INTO itineraries (id, account_id, is_public, date) VALUES (4, 4, FALSE, '2025-07-15 00:00:00'); -- Ethan
INSERT INTO itineraries (id, account_id, is_public, date) VALUES (5, 5, FALSE, '2025-08-15 00:00:00'); -- Peter

-- Ensure the itineraries id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('itineraries', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM itineraries), 1)
);

-- Event List 
-- Itinerary 1 (Nick to NYC + Poopsie)
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (1, 2, 3, 'Morning'); -- MoMA
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (2, 2, 2, 'Evening'); -- Cosimos

-- Itinerary 2 (Ellies Weekend in Carlisle)
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (3, 1, 1, 'Afternoon'); -- Hike in Carlisle

-- Itinerary 3 (Christian exploring Philly)
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (4, 3, 5, 'Afternoon'); -- Phillies Game
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (5, 3, 4, 'Evening'); -- Jazz

-- Itinerary 4 (Ethan at Lollapalooza)
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (6, 4, 6, 'All Day'); --Lolla

--Itinerary 5 (peter in paris)
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (7, 5, 7, 'Morning'); --Eiffel Tower
INSERT INTO event_list (id, itinerary_id, event_id, time_of_day) VALUES (8, 5, 8, 'Afternoon'); --Lourve 

-- Ensure the event_list id sequence matches the max(id)
SELECT setval(
    pg_get_serial_sequence('event_list', 'id'),
    GREATEST((SELECT COALESCE(MAX(id), 0) FROM event_list), 1)
);