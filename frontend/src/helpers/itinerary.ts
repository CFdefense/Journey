import type {
	Itinerary as ApiItinerary,
	EventDay,
	DayItinerary,
	Event
} from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

//function that calls api on given itinerary Id, returns the time block used by itinerary component
export async function fetchItinerary(
	itineraryID: number
): Promise<DayItinerary[]> {
	try {
		const apiResponse = await apiItineraryDetails(itineraryID);

		if (!apiResponse.result) {
			throw new Error(
				`Invalid itinerary result (status ${apiResponse.status})`
			);
		}

		return populateItinerary(apiResponse.result);
	} catch (err) {
		console.log("Error", err);
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

export function convertToApiFormat(
	days: DayItinerary[],
	itineraryId: number,
	startDate: string,
	endDate: string,
	title: string,
	chatSessionId: number | null = null
): ApiItinerary {
	return {
		id: itineraryId,
		start_date: startDate,
		end_date: endDate,
		chat_session_id: chatSessionId,
		title: title,
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
				noon_events: [], // Empty if not used
				afternoon_events: afternoonBlock?.events || [],
				evening_events: eveningBlock?.events || []
			};
		})
	};
}

export function sanitize(v: string | null): string | null {
	return v && v.trim() !== "" ? v : null;
}

export function getTimeBlockFromTimestamp(utcTimestamp: string): string | null {
	try {
		// Ensure the timestamp is treated as UTC by adding 'Z' if not present
		let timestamp = utcTimestamp;
		if (!timestamp.endsWith('Z') && !timestamp.includes('+') && !timestamp.includes('Z')) {
			timestamp = timestamp + 'Z';
		}
		
		const date = new Date(timestamp);
		const hours = date.getUTCHours();
		
		if (hours >= 4 && hours < 12) {
			return "Morning";
		} else if (hours >= 12 && hours < 18) {
			return "Afternoon";
		} else if (hours >= 18 || hours < 4) {
			return "Evening";
		}
		return null;
	} catch {
		return null;
	}
}

export function getDateFromTimestamp(utcTimestamp: string): string {
	try {
		// Ensure the timestamp is treated as UTC by adding 'Z' if not present
		let timestamp = utcTimestamp;
		if (!timestamp.endsWith('Z') && !timestamp.includes('+') && !timestamp.includes('Z')) {
			timestamp = timestamp + 'Z';
		}
		
		const date = new Date(timestamp);
		return date.toISOString().split('T')[0];
	} catch {
		return "";
	}
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
