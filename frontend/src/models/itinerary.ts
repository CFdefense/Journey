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
	/// Events not assigned to any specific time slot
	unassigned_events: Event[];
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
	event_name: string;
	event_description: string | null;
	street_address: string | null;
	city: string | null;
	country: string | null;
	postal_code: number | null;
	coords: string | null;
	event_type: string | null;
	user_created: boolean;
	hard_start: string | null;
	hard_end: string | null;
	/// Timezone of hard start and hard end
	timezone: string | null;
	place_id: string | null;
	wheelchair_accessible_parking: boolean | null;
	wheelchair_accessible_entrance: boolean | null;
	wheelchair_accessible_restroom: boolean | null;
	wheelchair_accessible_seating: boolean | null;
	serves_vegetarian_food: boolean | null;
	price_level: number | null;
	utc_offset_minutes: number | null;
	website_uri: string | null;
	types: string | null;
	photo_name: string | null;
	photo_width: number | null;
	photo_height: number | null;
	photo_author: string | null;
	photo_author_uri: string | null;
	photo_author_photo_uri: string | null;
	weekday_descriptions: string | null;
	secondary_hours_type: number | null;
	next_open_time: string | null;
	next_close_time: string | null;
	open_now: boolean | null;
	periods: Period[];
	special_days: string[];
	block_index: number | null;
	custom_image: string | null;
};

/// If you use this as a spread initializer, it must come before all other fields to not overwrite them.
/// Example:
/// let some_event: Event = {
///   ...EVENT_DEFAULT,
///   id: 12,
///   event_name: "some event name",
/// };
export const EVENT_DEFAULT: Event = {
	id: 0,
	event_name: "",
	event_description: null,
	street_address: null,
	city: null,
	country: null,
	postal_code: null,
	coords: null,
	event_type: null,
	user_created: false,
	hard_start: null,
	hard_end: null,
	timezone: null,
	place_id: null,
	wheelchair_accessible_parking: null,
	wheelchair_accessible_entrance: null,
	wheelchair_accessible_restroom: null,
	wheelchair_accessible_seating: null,
	serves_vegetarian_food: null,
	price_level: null,
	utc_offset_minutes: null,
	website_uri: null,
	types: null,
	photo_name: null,
	photo_width: null,
	photo_height: null,
	photo_author: null,
	photo_author_uri: null,
	photo_author_photo_uri: null,
	weekday_descriptions: null,
	secondary_hours_type: null,
	next_open_time: null,
	next_close_time: null,
	open_now: null,
	periods: [],
	special_days: [],
	block_index: null,
	custom_image: null
};

export type Period = {
	open_date: string | null;
	open_truncated: boolean | null;
	open_day: number;
	open_hour: number;
	open_minute: number;
	close_date: string | null;
	close_truncated: boolean | null;
	close_day: number | null;
	close_hour: number | null;
	close_minute: number | null;
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
	/// Timezone of hard start and hard end
	timezone: string | null;
	custom_image: string | null;
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
	/// Search where timezone like ...
	timezone: string | null;
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

/// This is a subset of IANA canonical timezone identifiers.
/// There is exactly 1 identifier for each unique timezone to eliminate duplicate timezones.
/// This list may need to be adjusted depending on how 3rd party APIs (like Google Maps) use timezones.
export const TIMEZONES: string[] = [
	"Pacific/Niue",
	"Pacific/Honolulu",
	"Pacific/Marquesas",
	"America/Anchorage",
	"America/Los_Angeles",
	"America/Denver",
	"America/Chicago",
	"America/New_York",
	"America/Halifax",
	"America/St_Johns",
	"America/Sao_Paulo",
	"Atlantic/Stanley",
	"America/Noronha",
	"Atlantic/Azores",
	"UTC",
	"Europe/Berlin",
	"Europe/Athens",
	"Africa/Nairobi",
	"Europe/Moscow",
	"Asia/Tehran",
	"Asia/Dubai",
	"Asia/Kabul",
	"Antarctica/Vostok",
	"Asia/Kolkata",
	"Asia/Kathmandu",
	"Asia/Dhaka",
	"Asia/Yangon",
	"Asia/Jakarta",
	"Asia/Shanghai",
	"Australia/Eucla",
	"Asia/Tokyo",
	"Australia/Adelaide",
	"Australia/Brisbane",
	"Australia/Lord_Howe",
	"Pacific/Norfolk",
	"Pacific/Auckland",
	"Pacific/Chatham",
	"Pacific/Tongatapu",
	"Pacific/Kiritimati"
];

export type UnsaveRequest = {
	id: number;
};
