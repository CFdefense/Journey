# Consolidated Agentic Workflow (Full Raw Markdown)

## Overview  

A complete 5-agent pipeline where the **Orchestrator** routes tasks between all agents and handles the full lifecycle of itinerary generation, updates, and feedback.  

Each agent has **minimal, focused responsibilities** and only the tools required for that role.

---

# AGENT 1 — Orchestrator Agent (Brain + Router)

**Role:**  

Central router + supervisor. Does **not** do low-level work (research, constraints, optimization) and does **not** directly parse profile/intent itself; instead it:

- routes tasks  
- manages pipeline flow  
- validates stage outputs  
- calls the right sub-agent in the right order  
- formats final results  

**Responsibilities (conceptual):**  

- Receive raw **user input** (new message)  
- Delegate context/intent work to **Task Agent** via `route_task(task_type = "task")`  
- After Task Agent finishes and context is ready:
  - Route to **Research / Constraint / Optimizer** in proper order using `route_task`  
  - Validate each stage’s output at a high level  
  - Decide whether to continue the pipeline or stop  
- Call `respond_to_user` when the itinerary or clarification is ready  
- Save resulting itinerary / context to DB (via downstream tools/controllers)  

**Actual Tools (Rust `Tool::name()`):**  

- `route_task`  
  - `task_type`: `"task" | "research" | "constraint" | "optimize"`  
  - `payload`: JSON **string** with the data to send to the sub-agent  
  - Used to:
    - First: call `route_task` with `"task"` to let Task Agent build context.  
    - Then: call `route_task` with `"research"`, `"constraint"`, `"optimize"` to run the pipeline.  
- `respond_to_user`  
  - Final step to insert a message to the user (itinerary or fallback “need more info” message).  

> Note: Profile loading, chat history loading, and intent parsing are **not** Orchestrator tools. They belong to the Task Agent.

---

# AGENT 2 — Task Agent (Clarification + Profile / Context Loading)

**Role:**  

Gather missing information and convert user input into a complete **Trip Requirements Object** / parsed intent, stored in shared context.  
Does **not** route tasks to Research / Constraint / Optimizer and does **not** build itineraries.

**Responsibilities:**  

- Load **user profile** (budgets, preferences, allergies, mobility, etc.)  
- Load **chat-session context** (conversation history + existing context blob)  
- Extract required fields:
  - destination  
  - dates  
  - party size  
  - mobility needs / allergies  
  - budget  
  - preferences  
- Detect missing or ambiguous details  
- Ask clarifying questions until everything essential is captured  
- Normalize results into a structured intent / requirements object and persist it in context  

**Actual Tools (Rust `Tool::name()`):**  

- `retrieve_user_profile`  
  - Loads the logged-in user’s profile and writes it into `context.user_profile`.  
- `retrieve_chat_context`  
  - Loads recent messages and the current `ContextData` snapshot (events, pipeline_stage, etc.).  
- `parse_user_intent`  
  - Uses an LLM to turn chat history / structured input into a `UserIntent` (destination, dates, budget, preferences, constraints, `missing_info`).  
- `ask_for_clarification`  
  - Generates and **inserts** a clarification question message, returning a `FINAL_ANSWER:` marker string.  
- `respond_to_user` *(primarily used later in the pipeline; also available to Task if needed)*  
  - Inserts a message back to the user, based on `ContextData.active_itinerary` and/or a custom message.  

**Output:**  

- Parsed intent + requirements written into `ContextData` (e.g., `parsed_intent`, `constraints`, `user_profile`).  
- If information is missing, a clarification message is sent to the user.  

---

# AGENT 3 — Research & Data Agent

**Role:**  

Gather all activity data needed to build itinerary candidates.

**Responsibilities:**  

- Query internal DB (`events`, `event_list`, itineraries) *(optional / future)*  
- Fetch POIs from external APIs *(future)*  
- Expand search across categories (sightseeing, food, nightlife, nature, etc.)  
- Validate core data points where possible:
  - price  
  - hours  
  - availability  
  - closures  
- Normalize all data into POI/Event objects that downstream agents can consume.  

**Tools (conceptual interface – to be implemented in `tools/research.rs`):**  

- `query_db(...)`  
- `find_related_pois(destination, dates, constraints)`  
- `validate_poi_data(poi)`  

**Output:**  

`List<POI>` — comprehensive candidate activity list, saved into context (`researched_events`).  

---

# AGENT 4 — Constraint & Feasibility Agent

**Role:**  

Determine which activities are realistically doable for this user under their constraints.

**Responsibilities:**  

- Check open/close times against planned days/time blocks  
- Validate reservation requirements (if known)  
- Compute travel time between POIs  
- Filter by:
  - budget  
  - allergies  
  - accessibility / disabilities  
  - schedule conflicts  
- Produce feasibility notes + filtered POIs  

**Tools (conceptual – wired via `tools/constraint.rs` and the constraint agent’s LLM prompt):**  

- `calculate_travel_time(A, B)`  
- `check_time_overlap(events)`  
- `check_budget_feasibility(event, user_profile)`  
- `verify_accessibility(event, user_profile)`  
- `score_feasibility(event_set)`  

**Output:**  

`Filtered<List<POI>>` + constraint metadata – a filtered list of events that satisfy constraints, plus explanations/metadata, saved into context (`constrained_events`, `constraints`).  

---

# AGENT 5 — Schedule Optimizer Agent

**Role:**  

Build the final itinerary using feasible activities.

**Responsibilities:**  

- Rank POIs using:
  - user preferences  
  - proximity  
  - diversity (avoid repetitive activity types)  
- Build a **multi-day schedule**  
- Assign POIs to time blocks (Morning/Afternoon/Evening)  
- Optimize for minimal travel + maximal enjoyment  
- Include meals and rest blocks  
- Output final structured itinerary ready for storage and presentation  

**Actual Tools (from `tools/optimizer.rs`):**  

- `rank_pois_by_preference` (`RankPOIsByPreferenceTool`)  
  - Rank POIs based on user profile (budget, risk tolerance, allergies, disabilities, interests).  
- `cluster_pois` (`ClusterPOIsTool`)  
  - Group POIs to ensure diversity and avoid over-clustering similar activity types.  
- `sequence_day` (`SequenceDayTool`)  
  - Create daily schedules and assign POIs into Morning / Afternoon / Evening blocks.  
- `optimize_route` (`OptimizeRouteTool`)  
  - Apply route optimization (TSP-style) to minimize travel distance/time for a day.  
- `deserialize_events` (`DeserializeEventsTool`)  
  - Convert optimized schedules into the database-ready itinerary/event schema.  

**Output:**  

`Itinerary` — complete structured schedule (days, time blocks, events, travel segments, costs), suitable for insertion into `itineraries` and related tables.  

---

# Full End-to-End Pipeline (Raw Diagram)

**User Input**

↓  

**Orchestrator Agent**

- Receive raw user message  
- Detect overall user intent at a high level (trip planning, modify trip, etc.)  
- Route request to **Task Agent**: `route_task(task_type = "task", payload = user_input)`  

↓  

**Task Agent**

- Load user profile + long-term preferences (`retrieve_user_profile`)  
- Load current chat-session context (`retrieve_chat_context`)  
- Extract required fields (destination, dates, party size, mobility needs, budget, preferences) via `parse_user_intent`  
- Detect missing or ambiguous info (`missing_info`)  
- If needed, clarify with the user (`ask_for_clarification`) until the task spec is complete  
- Produce and persist a fully-specified “Trip Requirements Object” / `UserIntent` in context  

↓  

**Orchestrator Agent**

- Validate Task Agent output (e.g., `missing_info` empty) via context/state  
- If clarification was sent, stop and let the user respond  
- If requirements are complete, route to **Research Agent**:  
  - `route_task(task_type = "research", payload = context snapshot)`  

↓  

**Research Agent**

- Query database for events, restaurants, local activities  
- Fetch external POIs (APIs, web data) *(future)*  
- Validate hours, pricing, availability, seasonal closures where possible  
- Normalize and return a **Candidate POI List**, saved to context (`researched_events`)  

↓  

**Orchestrator Agent**

- Validate Research output (non-empty candidate set, sane structure)  
- Route to **Constraint Agent**:  
  - `route_task(task_type = "constraint", payload = researched_events + user constraints)`  

↓  

**Constraint Agent**

- Apply user constraints: budget, timing, accessibility, preferences  
- Calculate travel time between POIs  
- Filter infeasible items  
- Return **“Feasible POI Set + Constraint Notes”** into context (`constrained_events`, `constraints`)  

↓  

**Orchestrator Agent**

- Validate constraint results (has feasible options)  
- Route to **Optimizer Agent**:  
  - `route_task(task_type = "optimize", payload = feasible_pois + user profile)`  

↓  

**Optimizer Agent**

- Rank POIs by user preferences (food, art, nightlife, nature, etc.)  
- Build multi-day schedule  
- Optimize route, travel flow, and energy levels  
- Produce **“Itinerary Draft v1”**, saved into context (`active_itinerary` / `events`).  

↓  

**Orchestrator Agent**

- Validate pipeline is complete (itinerary present in context)  
- Call `respond_to_user` to:  
  - Insert the final itinerary message for the user, or  
  - Ask for more info if itinerary is missing/empty  
- (Controllers then persist itineraries / events as needed.)  

↓  

**User Receives Itinerary**

↓  

**User Feedback?**

- **Yes** → Orchestrator routes feedback to the correct agent:  
  - Missing info → **Task Agent** (`route_task(task_type = "task", ...)`)  
  - Add/remove activities → **Research** or **Constraint Agent** (new research/constraint passes)  
  - Reorder schedule → **Optimizer Agent** (new optimization pass)  
- **No** → Save final documents to DB:  
  - `itineraries`  
  - `event_list`  
  - `user_preference_updates`  