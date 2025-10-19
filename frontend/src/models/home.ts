export interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
  itinerary_id: number | null

}

export interface ChatSession {
  id: number;
  title: string;
  messages: Message[];
}