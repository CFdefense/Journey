You are ranking a list of Points of Interest based on user preferences, budget, risk tolerance, allergies, and accessibility needs.

## Points of Interest Array:
```json
{}
```

## User's Preferences:
```json
{}
```

## Price Level Reference:
The event's price level is an integer which represents:
- 0 = Unspecified
- 1 = Free
- 2 = Inexpensive
- 3 = Moderate
- 4 = Expensive
- 5 = VeryExpensive

## Your Task:
1. For EACH object in the POI array, add a field called "rank" (integer)
   - 0 = best match for user (highest priority)
   - Higher numbers = worse match (lower priority)
   - Consider: user interests, budget, accessibility needs, dietary restrictions
2. Filter out any events with id = -1
3. Keep ALL other fields intact in each object
4. Sort the array by rank (lowest/best rank first)

## Ranking Criteria (in priority order):
1. **Safety**: Exclude POIs that conflict with disabilities or severe allergies
2. **Budget**: Lower rank for POIs within user budget (consider price_level)
3. **Interests**: Higher rank for POIs matching user interests (check types field)
4. **Accessibility**: Lower rank for POIs meeting accessibility needs
5. **Dietary**: Consider serves_vegetarian_food if relevant to user

## Output Format:
Return ONLY a valid JSON array with the ranked POIs. Do NOT include any explanatory text.

Example:
```json
[
  {{"id": 123, "event_name": "Museum", "rank": 0, ...other fields...}},
  {{"id": 456, "event_name": "Park", "rank": 1, ...other fields...}}
]
```