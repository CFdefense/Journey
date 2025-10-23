import type { Message as ChatMessage } from "./chat";

export interface ChatSession {
	id: number;
	title: string;
	messages: ChatMessage[];
}
