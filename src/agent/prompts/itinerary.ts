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
	event_name: string;
	event_description: string | null;
	street_address: string | null;
	city: string | null;
	country: string | null;
	postal_code: number | null;
	lat: number | null;
	lng: number | null;
	event_type: string | null;
	user_created: boolean;
	/// ISO 8601
	hard_start: string | null;
	/// ISO 8601
	hard_end: string | null;
	/// Timezone of hard start and hard end (see TIMEZONES below)
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
};

type Period = {
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

/// This is a subset of IANA canonical timezone identifiers.
/// There is exactly 1 identifier for each unique timezone to eliminate duplicate timezones.
const TIMEZONES: string[] = [
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