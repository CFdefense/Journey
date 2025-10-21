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

//function that calls api on given itinerary Id, returns the time block used by itinerary component
export async function fetchItinerary(itineraryID: number): Promise<TimeBlock[]> {
  try {
    const itinerary = await apiItineraryDetails(itineraryID);
    return populateItinerary(itinerary);
  } catch (err){
      console.log("Error", err);
      return [
        { time: "Morning", events: [] },
        { time: "Afternoon", events: [] },
        { time: "Evening", events: [] }
      ];
    }
}

export function populateItinerary(apiItinerary: ApiItinerary): TimeBlock[] {
  // if there is no event days, create empty time blocks on the itinerary
  if (!apiItinerary.event_days || apiItinerary.event_days.length === 0) {
    return [
      { time: "Morning", events: [] },
      { time: "Afternoon", events: [] },
      { time: "Evening", events: [] },
    ];
  }

  //takes day event day 1, the only day we have as of now
  const day: EventDay = apiItinerary.event_days[0];

  //return array of time blocks
  return [
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
  ];
}
