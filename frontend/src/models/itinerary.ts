export type Itinerary = {
	id: number;
	/// Destination's local timezone - %Y-%m-%d
	start_date: string;
	/// Destination's local timezone - %Y-%m-%d
	end_date: string;
	/// List of days containing events for that day
	/// * Days are guaranteed to be sorted in chronological order
	event_days: EventDay[];
	chat_session_id: number | null;
	title: string;
};

export type EventDay = {
	morning_events: Event[];
	noon_events: Event[];
	afternoon_events: Event[];
	evening_events: Event[];
	/// Destination's local timezone - %Y-%m-%d
	date: string;
};

export type Event = {
	id: number;
	street_address: string;
	postal_code: number;
	city: string;
	event_type: string;
	event_description: string;
	event_name: string;
	user_created: boolean;
	account_id: number | null;
	/// Destination's local timezone - %Y-%m-%d
	hard_start: Date | null; /// for testing sake this is what will have the 13:00 value for phillies game
	/// Destination's local timezone - %Y-%m-%d
	hard_end: Date | null;
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
