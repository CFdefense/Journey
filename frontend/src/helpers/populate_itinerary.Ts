import type { Itinerary as ApiItinerary, EventDay, Event as ApiEvent } from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

export interface Event {
  id: string;
  title: string;
  desc?: string;
  street_address?: string;
  postal_code?: number;
  city?: string;
  event_type?: string;
}

export interface TimeBlock {
  time: string;
  events: Event[];
}

export interface DayItinerary {
  date: string;
  timeBlocks: TimeBlock[];
}

//function that calls api on given itinerary Id, returns the time block used by itinerary component
export async function fetchItinerary(itineraryID: number): Promise<DayItinerary[]> {
  try {
    const apiResponse = await apiItineraryDetails(itineraryID);

    console.log(apiResponse);

    if (!apiResponse.result) {
      throw new Error(`Invalid itinerary result (status ${apiResponse.status})`);
    }

    return populateItinerary(apiResponse.result);
  } catch (err){
      console.log("Error", err);
      return [
        {
          date: "N/A",
          timeBlocks: [
            { time: "Morning", events: [] },
            { time: "Afternoon", events: [] },
            { time: "Evening", events: [] }
          ],
        },
      ];
    }
}

export function populateItinerary(apiItinerary: ApiItinerary): DayItinerary[] {
  if (!apiItinerary.event_days || apiItinerary.event_days.length === 0) {
    return [
      {
        date: "N/A",
        timeBlocks: [
          { time: "Morning", events: [] },
          { time: "Afternoon", events: [] },
          { time: "Evening", events: [] },
        ],
      },
    ];
  }

  return apiItinerary.event_days.map((day: EventDay) => ({
    date: day.date,
    timeBlocks: [
      {
        time: "Morning",
        events: day.morning_events.map((ev) => ({
          id: ev.id.toString(),
          title: ev.event_name,
          desc: ev.event_description,
          street_address: ev.street_address,
          postal_code: ev.postal_code,
          city: ev.city,
          event_type: ev.event_type,
        })),
      },
      {
        time: "Afternoon",
        events: day.afternoon_events.map((ev) => ({
          id: ev.id.toString(),
          title: ev.event_name,
          desc: ev.event_description,
          street_address: ev.street_address,
          postal_code: ev.postal_code,
          city: ev.city,
          event_type: ev.event_type,
        })),
      },
      {
        time: "Evening",
        events: day.evening_events.map((ev) => ({
          id: ev.id.toString(),
          title: ev.event_name,
          desc: ev.event_description,
          street_address: ev.street_address,
          postal_code: ev.postal_code,
          city: ev.city,
          event_type: ev.event_type,
        })),
      },
    ],
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
      const morningBlock = day.timeBlocks.find(tb => tb.time === "Morning");
      const afternoonBlock = day.timeBlocks.find(tb => tb.time === "Afternoon");
      const eveningBlock = day.timeBlocks.find(tb => tb.time === "Evening");

      return {
        date: day.date,
        morning_events: morningBlock?.events.map(convertEventToApi) || [],
        noon_events: [], // Empty if not used
        afternoon_events: afternoonBlock?.events.map(convertEventToApi) || [],
        evening_events: eveningBlock?.events.map(convertEventToApi) || [],
      };
    }),
  };
}

function convertEventToApi(event: Event): ApiEvent {
  return {
    id: parseInt(event.id),
    event_name: event.title,
    event_description: event.desc || "",
    street_address: event.street_address || "",
    postal_code: event.postal_code || 0,
    city: event.city || "",
    event_type: event.event_type || "",
  };
}
