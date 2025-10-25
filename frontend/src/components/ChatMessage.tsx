import { useEffect, useState } from "react";
import type { Message } from "../models/chat";
import type { Itinerary } from "../models/itinerary";
import { apiItineraryDetails } from "../api/itinerary";

export type ChatMessageParams = {
  message: Message;
  onItinerarySelect: (itineraryId: number) => void;
};

export default function ChatMessage({
  message,
  onItinerarySelect
}: ChatMessageParams) {
  const [itinerary, setItinerary] = useState<Itinerary | null>(null);

  useEffect(() => {
    async function getItinerary() {
      if (message.itinerary_id === null) {
        return;
      }
      const itineraryResult = await apiItineraryDetails(message.itinerary_id);
      // TODO: 401 -> navigate to /login

      if (itineraryResult.result === null || itineraryResult.status !== 200) {
        return; // TODO: handle and display error
      }

      setItinerary(itineraryResult.result);
    }

    getItinerary();
  }, [message.itinerary_id]);

  const formattedTimestamp = new Date(
    message.timestamp.replace(" ", "T")
  ).toLocaleString("en-US", {
    month: "2-digit",
    day: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: true
  });

  return (
    <div
      key={message.id}
      className={`chat-message ${message.is_user ? "user" : "bot"}`}
    >
      <div className="message-text">
        <p>{message.text}</p>
        <span className="timestamp">{formattedTimestamp}</span>
      </div>

      {!message.is_user && message.itinerary_id && (
        <div className="itinerary-info">
          <button
            className="display-itinerary-button"
            onClick={() => {
              console.log("Selected itinerary ID:", message.itinerary_id);
              onItinerarySelect(message.itinerary_id!);
            }}
          >
            Itinerary: {itinerary?.title ?? "No title"}
            {/* Will display this fallback text only if the database does not have an itinerary title for this message */}
          </button>
        </div>
      )}
    </div>
  );
}
