/// These tests require an active connection to the server

import { describe, expect, test } from "vitest";
import {
	test_state,
	apiSignUp, apiValidate, apiLogin, apiCurrent, apiLogout,
	apiChats, apiNewChatId, apiSendMessage, apiMessages,
	apiItineraryDetails,
} from "./testApi"; // Always use ./testApi instead of ../src/api/*

describe("Integration Tests", () => {
	test("Journey Flow DEV", async () => {
		test_state.dev_mode = true;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`
		expect(await apiSignUp({
			email: email,
			first_name: "First",
			last_name: "Last",
			password: "Password123"
		})).toBe(200);

		expect(await apiValidate()).toBe(true);

		expect(await apiLogin({
			email: email,
			password: "Password123"
		})).toBe(200);

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

		expect(await apiLogout()).toBe(200);

		expect(await apiValidate()).toBe(false);
	});
	test("Journey Flow PROD", async () => {
		test_state.dev_mode = false;
		const unique: number = Date.now();
		const email = `test${unique}@gmail.com`
		expect(await apiSignUp({
			email: email,
			first_name: "First",
			last_name: "Last",
			password: "Password123"
		})).toBe(200);

		expect(await apiValidate()).toBe(true);

		expect(await apiLogin({
			email: email,
			password: "Password123"
		})).toBe(200);

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

		expect(await apiLogout()).toBe(200);

		expect(await apiValidate()).toBe(false);
	});
});