export type Itinerary = {
	id: number,
    start_date: string,
    end_date: string,
    morning_events: [Event],
    noon_events: [Event],
    afternoon_events: [Event],
    evening_events: [Event],
    chat_session_id: number | null
}

export type Event = {
	id: number,
	street_address: string,
    postal_code: number,
    city: string,
    event_type: string,
    event_description: string,
    event_name: string
}

export type SavedResponse = {
    itineraries: [Itinerary]
}

export type SaveResponse = {
	id: number
}