// itinerary.ts - these are models relating to itineraries and events used throught the codebase.

type Itinerary = {
	id: number;
	/// %Y-%m-%d
	start_date: string;
	/// %Y-%m-%d
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
	/// %Y-%m-%d
	date: string;
};

type Event = {
	id: number;
};