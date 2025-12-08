# Itinerary TypeScript Model

These are the TypeScript type definitions for itineraries and events used throughout the codebase.

```typescript
type Itinerary = {
	id: number;
	/// Date format: YYYY-MM-DD
	start_date: string;
	/// Date format: YYYY-MM-DD
	end_date: string;
	event_days: EventDay[];
	chat_session_id: number | null;
	title: string;
	/// Events not assigned to any specific time slot
	unassigned_events: Event[];
};

type EventDay = {
	morning_events: Event[];
	afternoon_events: Event[];
	evening_events: Event[];
	/// Date format: YYYY-MM-DD
	date: string;
};

type Period = {
	open: { day: number; hour: number; minute: number; };
	close: { day: number; hour: number; minute: number; };
};

type Event = {
	id: number;
	event_name: string;
	event_description: string | null;
	street_address: string | null;
	city: string | null;
	country: string | null;
	postal_code: number | null;
	lat: number | null;
	lng: number | null;
	event_type: string | null;
	/// ISO 8601 datetime string (if event has fixed start time)
	hard_start: string | null;
	/// ISO 8601 datetime string (if event has fixed end time)
	hard_end: string | null;
	/// Timezone identifier (e.g., "America/New_York")
	timezone: string | null;
	wheelchair_accessible_parking: boolean | null;
	wheelchair_accessible_entrance: boolean | null;
	wheelchair_accessible_restroom: boolean | null;
	wheelchair_accessible_seating: boolean | null;
	serves_vegetarian_food: boolean | null;
	/// Price level: 0=Unspecified, 1=Free, 2=Inexpensive, 3=Moderate, 4=Expensive, 5=VeryExpensive
	price_level: number | null;
	utc_offset_minutes: number | null;
	/// Comma-separated list of place types
	types: string | null;
	weekday_descriptions: string | null;
	secondary_hours_type: number | null;
	/// ISO 8601 datetime string
	next_open_time: string | null;
	/// ISO 8601 datetime string
	next_close_time: string | null;
	open_now: boolean | null;
	periods: Period[];
	/// Array of dates in YYYY-MM-DD format
	special_days: string[];
	/// Optional ranking score (lower is better, 0 is best)
	rank?: number;
	/// Block index for ordering within time period
	block_index: number | null;
};
```

