export type Itinerary = {
	id: number;
	/// %Y-%m-%d
	start_date: string;
	/// %Y-%m-%d
	end_date: string;
	/// List of days containing events for that day
	/// * Days are guaranteed to be sorted in chronological order
	event_days: EventDay[];
	chat_session_id: number | null;
	title: string;
};

export type EventDay = {
	morning_events: Event[];
	afternoon_events: Event[];
	evening_events: Event[];
	/// %Y-%m-%d
	date: string;
};

export type Event = {
	id: number;
	street_address: string | null;
	postal_code: number | null;
	city: string | null;
	country: string | null;
	event_type: string | null;
	event_description: string | null;
	event_name: string;
	user_created: boolean;
	/// UTC %Y-%m-%dT%H:%M:%S%.f
	hard_start: string | null; /// for testing sake this is what will have the 13:00 value for phillies game
	/// UTC %Y-%m-%dT%H:%M:%S%.f
	hard_end: string | null;
};

export type SavedResponse = {
	/// List of saved itineraries for the user.
	itineraries: Itinerary[];
};

export type SaveResponse = {
	/// ID of the itinerary that was just saved
	/// * May be the same as the itinerary id passed in the request
	id: number;
};

/// A user-created event. It must have a name, and all other fields are optional.
export type UserEventRequest = {
	/// If id is provided, it updates the user-event with that id. Otherwise it creates the event.
	id: number | null;
	street_address: string | null;
	postal_code: number | null;
	city: string | null;
	country: string | null;
	event_type: string | null;
	event_description: string | null;
	event_name: string;
	hard_start: string | null;
	hard_end: string | null;
};

export type UserEventResponse = {
	id: number;
};

/// A set of query filters to search for an event in the DB.
///
/// ## Example
/// If event_name is provided, it will query the DB with something like this:
/// ```sql
/// SELECT * FROM events
/// WHERE name LIKE $1
/// LIMIT 10;
/// ```
export type SearchEventRequest = {
	/// Search where id=...
	id: number | null;
	/// Search where street_address like ...
	street_address: string | null;
	/// Search where postal_code=...
	postal_code: number | null;
	/// Search where city like ...
	city: string | null;
	/// Search where country like ...
	country: string | null;
	/// Search where event_type like ...
	event_type: string | null;
	/// Search where event_description like ...
	event_description: string | null;
	/// Search where event_name like ...
	event_name: string | null;
	/// Search where hard_start < ...
	hard_start_before: string | null;
	/// Search where hard_start > ...
	hard_start_after: string | null;
	/// Search where hard_end < ...
	hard_end_before: string | null;
	/// Search where hard_end > ...
	hard_end_after: string | null;
};

export type SearchEventResponse = {
	events: Event[];
};

// The API returns event days directly, not full itinerary objects
export type SavedItinerariesResponse = {
	itineraries: Itinerary[];
};

export type TimeBlock = {
	time: string;
	events: Event[];
};

export type DayItinerary = {
	date: string;
	timeBlocks: TimeBlock[];
};
