export interface Message {
  id: number;
  text: string;
  sender: "user" | "bot";
}

export interface ChatSession {
  id: number;
  title: string;
  messages: Message[];
}