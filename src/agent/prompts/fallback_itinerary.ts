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

type Event = {
	id: number;
	/// Block index for ordering within time period
	block_index: number | null;
};

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