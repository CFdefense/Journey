export type Itinerary = {
	id: number,
    date: string,
    morning_events: [EventRow],
    noon_events: [EventRow],
    afternoon_events: [EventRow],
    evening_events: [EventRow]
}

export type SavedResponse = {
    itineraries: [Itinerary]
}

export type EventRow = {
	id: number,
	street_address: string,
    postal_code: number,
    city: string,
    event_type: string,
    event_description: string,
    event_name: string
}