import type {
	Itinerary as ApiItinerary,
	EventDay,
	Event
} from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

export interface TimeBlock {
	time: string;
	events: Event[];
}

export interface DayItinerary {
	date: Date;
	timeBlocks: TimeBlock[];
}

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
				date: new Date(),
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
				date: new Date(),
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
	startDate: Date,
	endDate: Date,
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