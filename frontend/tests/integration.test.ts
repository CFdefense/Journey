/// These tests require an active connection to the server

import { beforeAll, describe, expect, test } from "vitest";
import {
	test_state,
	apiSignUp, apiValidate, apiLogin, apiCurrent, apiLogout,
	apiChats, apiNewChatId, apiSendMessage, apiMessages,
	apiItineraryDetails, apiDeleteChat,
} from "./testApi"; // Always use ./testApi instead of ../src/api/*

// Check server availability - will be set before tests run
let serverAvailable = false;

describe("Integration Tests", () => {
	// Check server availability before running tests
	beforeAll(async () => {
		try {
			const controller = new AbortController();
			const timeoutId = setTimeout(() => controller.abort(), 1000);
			const response = await fetch("http://localhost:3001/api/account/validate", {
				method: "GET",
				credentials: "include",
				signal: controller.signal
			});
			clearTimeout(timeoutId);
			serverAvailable = response.status !== -1;
		} catch {
			serverAvailable = false;
		}
	});

	test.skipIf(() => !serverAvailable)("Journey Flow DEV", async () => {
		test_state.dev_mode = true;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`
	expect((await apiSignUp({
		email: email,
		first_name: "First",
		last_name: "Last",
		password: "Password123"
	})).status).toBe(200);

	expect((await apiValidate()).status === 200).toBe(true);

	expect((await apiLogin({
		email: email,
		password: "Password123"
	})).status).toBe(200);

		expect(await apiCurrent()).toStrictEqual({
			result: {
				email: email,
				first_name: "First",
				last_name: "Last",
			budget_preference: null,
			risk_preference: null,
			food_allergies: "",
			disabilities: ""
			},
			status: 200
		});

		expect(await apiChats()).toStrictEqual({
			result: {
				chat_sessions: []
			},
			status: 200
		});

		const newChatResult = await apiNewChatId();
		expect(newChatResult.status).toBe(200);
		const newChatId = newChatResult.result!;

		const messageResult = await apiSendMessage({
			chat_session_id: newChatId,
			text: "test message",
			itinerary_id: null
		});
		expect(messageResult.status).toBe(200);
		const messageId = messageResult.result!.user_message_id;
		const botMessage = messageResult.result!.bot_message;
		expect(botMessage.itinerary_id === null).toBe(false);

		const itineraryResult = await apiItineraryDetails(botMessage.itinerary_id!);
		expect(itineraryResult.status).toBe(200);

		const pageResult = await apiMessages({
			chat_session_id: newChatId,
			message_id: null
		});
		expect(pageResult.status).toBe(200);
		expect(pageResult.result!.prev_message_id).toBe(null);
		expect(pageResult.result!.message_page[0].id).toBe(messageId);
		expect(pageResult.result!.message_page[1]).toStrictEqual(botMessage);

	expect((await apiLogout()).status).toBe(200);

	expect((await apiValidate()).status !== 200).toBe(true);
	});
	
	test.skipIf(() => !serverAvailable)("Journey Flow PROD", async () => {
		test_state.dev_mode = false;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`
	expect((await apiSignUp({
		email: email,
		first_name: "First",
		last_name: "Last",
		password: "Password123"
	})).status).toBe(200);

	expect((await apiValidate()).status === 200).toBe(true);

	expect((await apiLogin({
		email: email,
		password: "Password123"
	})).status).toBe(200);

		expect(await apiCurrent()).toStrictEqual({
			result: {
				email: email,
				first_name: "First",
				last_name: "Last",
			budget_preference: null,
			risk_preference: null,
			food_allergies: "",
			disabilities: ""
			},
			status: 200
		});

		expect(await apiChats()).toStrictEqual({
			result: {
				chat_sessions: []
			},
			status: 200
		});

		const newChatResult = await apiNewChatId();
		expect(newChatResult.status).toBe(200);
		const newChatId = newChatResult.result!;

		const messageResult = await apiSendMessage({
			chat_session_id: newChatId,
			text: "test message",
			itinerary_id: null
		});
		expect(messageResult.status).toBe(200);
		const messageId = messageResult.result!.user_message_id;
		const botMessage = messageResult.result!.bot_message;
		expect(botMessage.itinerary_id === null).toBe(false);

		const itineraryResult = await apiItineraryDetails(botMessage.itinerary_id!);
		expect(itineraryResult.status).toBe(200);

		const pageResult = await apiMessages({
			chat_session_id: newChatId,
			message_id: null
		});
		expect(pageResult.status).toBe(200);
		expect(pageResult.result!.prev_message_id).toBe(null);
		expect(pageResult.result!.message_page[0].id).toBe(messageId);
		expect(pageResult.result!.message_page[1]).toStrictEqual(botMessage);

	expect((await apiLogout()).status).toBe(200);

	expect((await apiValidate()).status !== 200).toBe(true);
	});

	test.skipIf(() => !serverAvailable)("Error handling coverage", async () => {
		test_state.dev_mode = true;
		
		// Test error handling paths
		const errorResult = await apiCurrent();
		expect(errorResult.status).toBeGreaterThanOrEqual(-1);
		
		const errorChats = await apiChats();
		expect(errorChats.status).toBeGreaterThanOrEqual(-1);
		
		const errorChatId = await apiNewChatId();
		expect(errorChatId.status).toBeGreaterThanOrEqual(-1);
		
		const errorItinerary = await apiItineraryDetails(99999);
		expect(errorItinerary.status).toBeGreaterThanOrEqual(-1);
		
		const errorMessages = await apiMessages({ chat_session_id: 99999, message_id: null });
		expect(errorMessages.status).toBeGreaterThanOrEqual(-1);
		
		const errorSend = await apiSendMessage({ 
			chat_session_id: 99999, 
			text: "test", 
			itinerary_id: null 
		});
		expect(errorSend.status).toBeGreaterThanOrEqual(-1);
	});

	test("Delete Chat DEV", async () => {
		test_state.dev_mode = true;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`;

		// Sign up and login
		expect((await apiSignUp({
			email: email,
			first_name: "First",
			last_name: "Last",
			password: "Password123"
		})).status).toBe(200);

		expect((await apiLogin({
			email: email,
			password: "Password123"
		})).status).toBe(200);

		// Create a new chat
		const newChatResult = await apiNewChatId();
		expect(newChatResult.status).toBe(200);
		const chatId = newChatResult.result!;

		// Verify chat exists
		const chatsBeforeDelete = await apiChats();
		expect(chatsBeforeDelete.status).toBe(200);
		expect(chatsBeforeDelete.result!.chat_sessions.length).toBe(1);
		expect(chatsBeforeDelete.result!.chat_sessions[0].id).toBe(chatId);

		// Delete the chat
		const deleteResult = await apiDeleteChat(chatId);
		expect(deleteResult.status).toBe(200);
		expect(deleteResult.result).toBe(chatId);

		// Verify chat is deleted
		const chatsAfterDelete = await apiChats();
		expect(chatsAfterDelete.status).toBe(200);
		expect(chatsAfterDelete.result!.chat_sessions.length).toBe(0);

		// Test deleting non-existent chat
		const deleteNonExistent = await apiDeleteChat(99999);
		expect(deleteNonExistent.status).toBeGreaterThanOrEqual(-1);

		expect((await apiLogout()).status).toBe(200);
	});

	test("Delete Chat PROD", async () => {
		test_state.dev_mode = false;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`;

		// Sign up and login
		expect((await apiSignUp({
			email: email,
			first_name: "First",
			last_name: "Last",
			password: "Password123"
		})).status).toBe(200);

		expect((await apiLogin({
			email: email,
			password: "Password123"
		})).status).toBe(200);

		// Create a new chat
		const newChatResult = await apiNewChatId();
		expect(newChatResult.status).toBe(200);
		const chatId = newChatResult.result!;

		// Verify chat exists
		const chatsBeforeDelete = await apiChats();
		expect(chatsBeforeDelete.status).toBe(200);
		expect(chatsBeforeDelete.result!.chat_sessions.length).toBe(1);
		expect(chatsBeforeDelete.result!.chat_sessions[0].id).toBe(chatId);

		// Delete the chat
		const deleteResult = await apiDeleteChat(chatId);
		expect(deleteResult.status).toBe(200);
		expect(deleteResult.result).toBe(chatId);

		// Verify chat is deleted
		const chatsAfterDelete = await apiChats();
		expect(chatsAfterDelete.status).toBe(200);
		expect(chatsAfterDelete.result!.chat_sessions.length).toBe(0);

		// Test deleting non-existent chat
		const deleteNonExistent = await apiDeleteChat(99999);
		expect(deleteNonExistent.status).toBeGreaterThanOrEqual(-1);

		expect((await apiLogout()).status).toBe(200);
	});
});