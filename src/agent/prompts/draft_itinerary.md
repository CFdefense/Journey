You are assembling a list of Points of Interest into a complete itinerary structure.

## Ranked POIs (sorted by preference):
```json
{}
```

## Itinerary Model (TypeScript):
These are the TypeScript type definitions for itineraries and events used throughout the codebase.
```typescript
{}
```

## Diversity Factor: {}
(0.0 = cluster similar activities, 1.0 = maximize variety)

## Trip Context:
```json
{}
```

## Your Task:
Create a complete itinerary following the TypeScript model structure. Distribute POIs across days and time blocks intelligently.

## Rules:
1. **Time Blocks**: Organize POIs into:
   - Morning: 6am-12pm (e.g., breakfast, museums, markets)
   - Afternoon: 12pm-6pm (e.g., lunch, outdoor activities, tours)
   - Evening: 6pm-12am (e.g., dinner, nightlife, entertainment)

2. **Venue Hours**: Check `next_open_time`, `next_close_time`, `open_now`, and `periods` fields
   - Don't schedule POIs when they're closed
   - Consider typical visit durations

3. **Geographic Clustering**: Use lat/lng to group nearby POIs on same day
   - Minimize backtracking between distant locations
   - Keep geographically close events together

4. **Diversity** (based on diversity_factor):
   - Avoid 3+ similar activities in a row (check `types` field)
   - Balance indoor/outdoor activities
   - Mix: cultural, recreational, dining, relaxation

5. **Ranking Priority**: Prefer lower-ranked POIs (rank 0 is best)
   - Try to include all rank 0-2 POIs
   - Use rank 3+ to fill gaps if needed

6. **Hard Times**: If POI has `hard_start` and `hard_end`:
   - MUST schedule at exact time
   - Assign to correct time block based on start time
   - Don't schedule anything conflicting

7. **Date Assignment**: Use trip dates from trip_context
   - Start with start_date, end with end_date
   - Assign POIs to specific dates (YYYY-MM-DD format)

8. **Unassigned Events**: POIs that don't fit should go in `unassigned_events` array

## Output Format:
Return ONLY a valid JSON object matching the Itinerary model (without id, chat_session_id, title - these will be added later).

```json
{{
  "event_days": [
    {{
      "date": "2025-01-15",
      "morning_events": [/* full Event objects */],
      "afternoon_events": [/* full Event objects */],
      "evening_events": [/* full Event objects */]
    }}
  ],
  "unassigned_events": [/* full Event objects that couldn't be scheduled */]
}}
```

**CRITICAL JSON FORMATTING RULES**:
1. Include the COMPLETE Event object (all fields) in each array, not just the id
2. **NO TRAILING COMMAS** - Remove any trailing commas after the last item in arrays or objects
3. Ensure all strings are properly quoted
4. Ensure all property names use double quotes
5. Use null for null values (not "null" string)
6. Validate your JSON structure before returning