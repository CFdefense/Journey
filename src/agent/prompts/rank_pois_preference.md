You are scoring a single Point of Interest (POI) for a user based on their preferences, budget, risk tolerance, allergies, and accessibility needs.

## POI:
```json
{{POI_JSON}}
```

## User Profile:
```json
{{USER_PROFILE_JSON}}
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
1. Analyze how well this POI matches the user's interests, budget, accessibility needs, and dietary restrictions.
2. Return ONLY a JSON object with:
   ```json
   {
     "score": number
   }
   ```
   - `score = 0` → best / highest priority
   - Larger numbers → worse match (lower priority)

## Scoring Criteria (in priority order):
1. **Safety**: If the POI conflicts with disabilities or severe allergies, set `score` to a very high value like `9999`.
2. **Budget**: Prefer POIs within user budget (consider `price_level`) and give them lower scores.
3. **Interests**: Prefer POIs matching user interests (check `types` / category fields) and give them lower scores.
4. **Accessibility**: Prefer POIs that meet accessibility needs and give them lower scores.
5. **Dietary**: Consider `serves_vegetarian_food` and similar fields if relevant to the user.

## Output Format:
Return ONLY a valid JSON object with the `score` field. Do NOT include any explanatory text or extra keys.