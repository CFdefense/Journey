import type { Itinerary as ApiItinerary, EventDay } from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

export interface Event {
  id: string;
  title: string;
  desc?: string;
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
    const itinerary = await apiItineraryDetails(itineraryID);
    console.log(itinerary);
    return populateItinerary(itinerary);
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
  // if there is no event days, create empty time blocks on the itinerary
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
        })),
      },
      {
        time: "Afternoon",
        events: day.afternoon_events.map((ev) => ({
          id: ev.id.toString(),
          title: ev.event_name,
          desc: ev.event_description,
        })),
      },
      {
        time: "Evening",
        events: day.evening_events.map((ev) => ({
          id: ev.id.toString(),
          title: ev.event_name,
          desc: ev.event_description,
        })),
      },
    ],
  }));
}
