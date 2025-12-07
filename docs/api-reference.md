# API Documentation

## Account Routes

### Public Routes

#### 1. POST /api/account/signup

Creates a new user account

**Requires:** 
- `email`
- `first_name`
- `last_name`
- `password`

**Returns:** Sets auth-token cookie on success

**Errors:** 
- 400 (validation)
- 409 (email exists)
- 500 (server error)

---

#### 2. POST /api/account/login

Authenticates a user

**Requires:** 
- `email`
- `password`

**Returns:** Sets auth-token cookie on success

**Errors:** 
- 400 (invalid credentials)
- 500 (server error)

---

### Protected Routes (Require Authentication)

#### 3. GET /api/account/validate

Validates if the user has a valid auth-token cookie

**Returns:** 200 if valid, 401 if invalid/missing

---

#### 4. GET /api/account/current

Gets the current user's account information

**Returns:** 
- `email`
- `first_name`
- `last_name`
- `budget_preference`
- `risk_preference`
- `food_allergies`
- `disabilities`
- `profile_picture`

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

#### 5. POST /api/account/update

Updates user account information

**Accepts (all optional):** 
- `email`
- `first_name`
- `last_name`
- `password`
- `current_password`
- `budget_preference`
- `risk_preference`
- `food_allergies`
- `disabilities`
- `profile_picture`

**Note:** If updating password, `current_password` is required

**Returns:** Updated account information

**Errors:** 
- 400 (validation/wrong password)
- 401 (unauthorized)
- 500 (server error)

---

#### 6. GET /api/account/logout

Logs out the user by expiring their auth-token cookie

**Returns:** 200 on success

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

## Chat Routes

All chat routes require authentication.

### 1. GET /api/chat/chats

Fetches all chat session IDs and titles belonging to the user

**Returns:** Array of chat sessions with `id` and `title`

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

### 2. POST /api/chat/messagePage

Gets a page of messages from a chat session

**Requires:** 
- `chat_session_id`
- `message_id` (optional)

**Returns:** 
- Array of messages (up to MESSAGE_PAGE_LEN)
- `prev_message_id` for pagination

**Note:** If no `message_id` provided, returns latest messages; otherwise returns messages up to and including that message

**Errors:** 
- 400 (bad request)
- 401 (unauthorized)
- 500 (server error)

---

### 3. POST /api/chat/sendMessage

Sends a new message to the LLM and waits for response

**Requires:** 
- `chat_session_id`
- `text`
- `itinerary_id` (optional)

**Returns:** 
- `user_message_id`
- `bot_message` (includes generated itinerary)

**Note:** Inserts user message, sends to LLM, generates itinerary, returns bot response

**Errors:** 
- 400 (bad request/empty text)
- 401 (unauthorized)
- 404 (chat not found)
- 500 (server error)

---

### 4. POST /api/chat/updateMessage

Updates an existing user message and gets new LLM response

**Requires:** 
- `message_id`
- `new_text`
- `itinerary_id` (optional)

**Returns:** New bot message (includes generated itinerary)

**Note:** Deletes all messages after the updated message, updates the text, gets new LLM response

**Errors:** 
- 400 (bad request/empty text)
- 401 (unauthorized)
- 404 (message not found)
- 500 (server error)

---

### 5. GET /api/chat/newChat

Gets or creates an empty chat session for the user

**Returns:** `chat_session_id`

**Note:** Returns existing empty chat if one exists, otherwise creates new one with title "New Chat"

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

### 6. DELETE /api/chat/:id

Deletes a chat session and associated data

**Requires:** `id` (path parameter)

**Note:** Deletes unsaved, private itineraries and all messages in the chat session

**Errors:** 
- 401 (unauthorized)
- 404 (chat not found)
- 500 (server error)

---

### 7. POST /api/chat/rename

Renames a chat session

**Requires:** 
- `id`
- `new_title`

**Note:** Title cannot be empty string

**Errors:** 
- 400 (bad request/empty title)
- 401 (unauthorized)
- 404 (chat not found)
- 500 (server error)

---

## Itinerary Routes

All itinerary routes require authentication.

### 1. GET /api/itinerary/saved

Fetches all saved itineraries belonging to the user

**Returns:** Array of complete itineraries with `event_days` and `unassigned_events`

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

### 2. GET /api/itinerary/{id}

Fetches a specific itinerary by ID

**Returns:** Complete itinerary with all events

**Note:** Returns itinerary if it belongs to user OR if it's public

**Errors:** 
- 401 (unauthorized)
- 404 (not found)
- 500 (server error)

---

### 3. POST /api/itinerary/save

Saves a new itinerary or updates an existing one

**Requires:** Complete Itinerary object
- `id`
- `start_date`
- `end_date`
- `event_days`
- `chat_session_id`
- `title`
- `unassigned_events`

**Returns:** `id` of the saved itinerary

**Note:** If ID exists for user, updates it; otherwise creates new one. Sets `saved=TRUE` and rebuilds event_list

**Errors:** 
- 400 (bad request)
- 401 (unauthorized)
- 500 (server error)

---

### 4. POST /api/itinerary/unsave

Unsaves an existing itinerary

**Requires:** 
- `id` (itinerary ID)

**Note:** Sets `saved=FALSE` for the itinerary, verifies it belongs to user

**Errors:** 
- 401 (unauthorized)
- 404 (not found)
- 500 (server error)

---

### 5. POST /api/itinerary/userEvent

Creates or updates a user-created custom event

**Requires:** 
- `event_name` (required)

**Optional fields:**
- `id`
- `street_address`
- `postal_code`
- `city`
- `country`
- `event_type`
- `event_description`
- `hard_start`
- `hard_end`
- `timezone`

**Returns:** `id` of the created/updated event

**Note:** If `id` provided, updates existing event; otherwise creates new one. Event must belong to user for updates

**Errors:** 
- 400 (bad request/empty name)
- 401 (unauthorized)
- 404 (event not found for update)
- 500 (server error)

---

### 6. POST /api/itinerary/searchEvent

Searches for events matching provided filters

**Accepts filters (all optional):**
- `id`
- `event_name`
- `street_address`
- `postal_code`
- `city`
- `country`
- `event_type`
- `event_description`
- `hard_start_before`
- `hard_start_after`
- `hard_end_before`
- `hard_end_after`
- `timezone`

**Returns:** Array of matching events (limited to EVENT_SEARCH_RESULT_LEN)

**Note:** Returns non-user-created events OR user-created events belonging to this user. Uses case-insensitive partial matching (ILIKE) for string fields

**Errors:** 
- 401 (unauthorized)
- 500 (server error)

---

### 7. DELETE /api/itinerary/userEvent/{id}

Deletes a user-created event

**Requires:** `id` (path parameter)

**Note:** Only deletes events where `user_created=TRUE` and belongs to requesting user

**Errors:** 
- 401 (unauthorized)
- 404 (event not found or doesn't belong to user)
- 500 (server error)