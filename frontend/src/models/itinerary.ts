export type Itinerary = {
	id: number,
	/// %Y-%m-%d
    start_date: string,
    /// %Y-%m-%d
    end_date: string,
    event_days: EventDay[],
    chat_session_id: number | null
}

export type EventDay = {
	morning_events: Event[],
    noon_events: Event[],
    afternoon_events: Event[],
    evening_events: Event[],
    /// %Y-%m-%d
    date: string
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
    itineraries: Itinerary[]
}

export type SaveResponse = {
	id: number
}