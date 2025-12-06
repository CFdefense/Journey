import type {
	Itinerary as ApiItinerary,
	EventDay,
	DayItinerary,
	Event
} from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

import { toast } from "../components/Toast";

// Function that calls api on given itinerary Id, returns the time block used by itinerary component
export async function fetchItinerary(
	itineraryID: number
): Promise<DayItinerary[]> {
	const apiResponse = await apiItineraryDetails(itineraryID);

	// Handle 401, redirect to login
	if (apiResponse.status === 401) {
		toast.error("Unauthorized user, please log in.");
		return [];
	}

	// Other status'
	if (apiResponse.status === 404) {
		toast.error("Itinerary not found.");
		return [];
	}

	if (!apiResponse.result || apiResponse.status !== 200) {
		toast.error("Unable to load itinerary. Please try again.");
		return [
				{
					date: "",
					timeBlocks: [
						{ time: "Morning", events: [] },
						{ time: "Noon", events: [] },
						{ time: "Afternoon", events: [] },
						{ time: "Evening", events: [] }
					]
				}
			];
	}
	return populateItinerary(apiResponse.result);
}

export function populateItinerary(apiItinerary: ApiItinerary): DayItinerary[] {
	if (!apiItinerary.event_days || apiItinerary.event_days.length === 0) {
		return [
			{
				date: "",
				timeBlocks: [
					{ time: "Morning", events: [] },
					{ time: "Afternoon", events: [] },
					{ time: "Evening", events: [] }
				]
			}
		];
	}

	return apiItinerary.event_days.map((day: EventDay) => ({
		date: day.date,
		timeBlocks: [
			{
				time: "Morning",
				events: day.morning_events
			},
			{
				time: "Afternoon",
				events: day.afternoon_events
			},
			{
				time: "Evening",
				events: day.evening_events
			}
		]
	}));
}

// Extract unassigned events from the itinerary
export function getUnassignedEvents(apiItinerary: ApiItinerary): Event[] {
	return apiItinerary.unassigned_events || [];
}

export function convertToApiFormat(
	days: DayItinerary[],
	itineraryId: number,
	startDate: string,
	endDate: string,
	title: string,
	chatSessionId: number | null = null,
	unassignedEvents: Event[] = []
): ApiItinerary {
	return {
		id: itineraryId,
		start_date: startDate,
		end_date: endDate,
		chat_session_id: chatSessionId,
		title: title,
		unassigned_events: unassignedEvents,
		event_days: days.map((day) => {
			const morningBlock = day.timeBlocks.find(
				(tb) => tb.time === "Morning"
			);
			const afternoonBlock = day.timeBlocks.find(
				(tb) => tb.time === "Afternoon"
			);
			const eveningBlock = day.timeBlocks.find(
				(tb) => tb.time === "Evening"
			);

			return {
				date: day.date,
				morning_events: morningBlock?.events || [],
				afternoon_events: afternoonBlock?.events || [],
				evening_events: eveningBlock?.events || []
			};
		})
	};
}

export function sanitize(v: string | null): string | null {
	const str = v?.trim();
	return str && str !== "" ? str : null;
}

export function getTimeBlockFromTimestamp(utcTimestamp: string): string | null {
	// Ensure the timestamp is treated as UTC by adding 'Z' if not present
	let timestamp = utcTimestamp;
	if (!timestamp.endsWith("Z") && !timestamp.includes("+")) {
		timestamp = timestamp + "Z";
	}

	const date = new Date(timestamp);

	// Checks if the date is valid instead of using the try catch.
	if (isNaN(date.getTime())) {
		return null;
	}

	const hours = date.getUTCHours();

	if (hours >= 4 && hours < 12) {
		return "Morning";
	} else if (hours >= 12 && hours < 18) {
		return "Afternoon";
	} else if (hours >= 18 || hours < 4) {
		return "Evening";
	}
	return null;
}

export function getDateFromTimestamp(utcTimestamp: string): string {
	// Ensure the timestamp is treated as UTC by adding 'Z' if not present
	let timestamp = utcTimestamp;
	if (!timestamp.endsWith("Z") && !timestamp.includes("+")) {
		timestamp = timestamp + "Z";
	}

	const date = new Date(timestamp);

	// Same date check
	if (isNaN(date.getTime())) {
		return "";
	}
	return date.toISOString().split("T")[0];
}

export function canDropEventInTimeBlock(
	event: Event,
	targetTimeBlock: string,
	targetDate: string,
	targetTimeIndex: number
): boolean {
	// Always allows it to be dropped into unassigned events
	if (targetTimeIndex === -1) {
		return true;
	}

	// If there is no hardstart, let it drop wherever
	if (!event.hard_start) {
		return true;
	}

	// Get the required time block and date from hard_start
	const requiredTimeBlock = getTimeBlockFromTimestamp(event.hard_start);
	const requiredDate = getDateFromTimestamp(event.hard_start);

	// If we couldn't parse the timestamp, allow the drop (fail open)
	if (!requiredTimeBlock || !requiredDate) {
		return true;
	}

	// Check if both time block and date match
	return requiredTimeBlock === targetTimeBlock && requiredDate === targetDate;
}

// Lets you know where the event is allowed to be dropped
export function getDropErrorMessage(event: Event): string | null {
	if (!event.hard_start) return null;

	const requiredTimeBlock = getTimeBlockFromTimestamp(event.hard_start);
	const requiredDate = getDateFromTimestamp(event.hard_start);

	if (requiredTimeBlock && requiredDate) {
		return `"${event.event_name}" has a fixed start time and must be placed in the ${requiredTimeBlock} block on ${requiredDate}.`;
	}

	return null;
}
