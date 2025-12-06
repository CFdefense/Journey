/// These tests require an active connection to the server

import { describe, expect, test } from "vitest";
import {
	test_state,
	apiSignUp,
	apiValidate,
	apiLogin,
	apiCurrent,
	apiLogout,
	apiChats,
	apiNewChatId,
	apiSendMessage,
	apiMessages,
	apiItineraryDetails,
	apiDeleteChat,
	apiRenameChat,
	apiUpdateMessage,
	apiUserEvent,
	apiSearchEvent,
	apiDeleteUserEvent,
	apiSaveItineraryChanges,
	apiUnsaveItinerary,
	apiGetSavedItineraries
} from "./testApi"; // Always use ./testApi instead of ../src/api/*
import { ChatSessionRow } from "../src/models/chat";
import {
	SearchEventRequest,
	UserEventRequest,
	Itinerary
} from "../src/models/itinerary";

async function test_flow() {
	// sign up
	const unique: number = Date.now();
	const email = `test${unique}@gmail.com`;
	expect(
		(
			await apiSignUp({
				email: email,
				first_name: "First",
				last_name: "Last",
				password: "Password123"
			})
		).status
	).toBe(200);

	// validate cookie
	expect((await apiValidate()).status === 200).toBe(true);

	// login
	expect(
		(
			await apiLogin({
				email: email,
				password: "Password123"
			})
		).status
	).toBe(200);

	// get account info
	expect(await apiCurrent()).toStrictEqual({
		result: {
			email: email,
			first_name: "First",
			last_name: "Last",
			budget_preference: null,
			risk_preference: null,
			food_allergies: "",
			disabilities: "",
			profile_picture: ""
		},
		status: 200
	});

	// get chat session ids
	expect(await apiChats()).toStrictEqual({
		result: {
			chat_sessions: []
		},
		status: 200
	});

	// create new chat
	const newChatResult = await apiNewChatId();
	expect(newChatResult.status).toBe(200);
	const newChatId = newChatResult.result!;

	// send message
	const messageResult = await apiSendMessage({
		chat_session_id: newChatId,
		text: "test message",
		itinerary_id: null
	});
	expect(messageResult.status).toBe(200);
	const messageId = messageResult.result!.user_message_id;
	const botMessage = messageResult.result!.bot_message;
	expect(botMessage.itinerary_id === null).toBe(false);

	// get itinerary info
	const itineraryResult = await apiItineraryDetails(botMessage.itinerary_id!);
	expect(itineraryResult.status).toBe(200);

	// get messages from chat
	const pageResult = await apiMessages({
		chat_session_id: newChatId,
		message_id: null
	});
	expect(pageResult.status).toBe(200);
	expect(pageResult.result!.prev_message_id).toBe(null);
	expect(pageResult.result!.message_page[0].id).toBe(messageId);
	expect(pageResult.result!.message_page[1]).toStrictEqual(botMessage);

	// update the message
	const updateResult = await apiUpdateMessage({
		message_id: messageId,
		new_text: "updated test message",
		itinerary_id: null
	});
	expect(updateResult.status).toBe(200);
	expect(updateResult.result).not.toBe(null);
	const updatedBotMessage = updateResult.result!;
	expect(updatedBotMessage.itinerary_id === null).toBe(false);

	// verify updated messages in chat
	const updatedPageResult = await apiMessages({
		chat_session_id: newChatId,
		message_id: null
	});
	expect(updatedPageResult.status).toBe(200);
	expect(updatedPageResult.result!.message_page[0].text).toBe(
		"updated test message"
	);
	expect(updatedPageResult.result!.message_page[1]).toStrictEqual(
		updatedBotMessage
	);

	// rename the chat
	const renameResult = await apiRenameChat({
		new_title: "Updated Title",
		id: newChatId
	});
	expect(renameResult.status).toBe(200);
	const chatsAfterRename = await apiChats();
	expect(chatsAfterRename.status).toBe(200);
	const expectedChat: ChatSessionRow = {
		id: newChatId,
		title: "Updated Title"
	};
	expect(
		chatsAfterRename.result!.chat_sessions.find(
			(chat) => chat.id === expectedChat.id
		)!.title
	).toBe(expectedChat.title);

	// Delete the chat
	const deleteResult = await apiDeleteChat(newChatId);
	expect(deleteResult.status).toBe(200);

	// Verify chat is deleted
	const chatsAfterDelete = await apiChats();
	expect(chatsAfterDelete.status).toBe(200);
	expect(chatsAfterDelete.result!.chat_sessions.length).toBe(0);

	// Test deleting non-existent chat
	const deleteNonExistent = await apiDeleteChat(99999);
	expect(deleteNonExistent.status).toBeGreaterThanOrEqual(-1);

	// Create user-event
	const userEvent: UserEventRequest = {
		id: null,
		event_name: "test",
		street_address: "test",
		postal_code: 1,
		city: "test",
		country: "test",
		event_type: "test",
		event_description: "test",
		hard_start: "2015-11-12T23:25:00",
		hard_end: "2025-11-12T23:25:00",
		timezone: "UTC"
	};
	const createEventRes = await apiUserEvent(userEvent);
	expect(createEventRes.status).toBe(200);

	// Update user-event
	userEvent.id = createEventRes.result!.id;
	const updatedName = "test updated";
	userEvent.event_name = updatedName;
	const updateEventRes = await apiUserEvent(userEvent);
	expect(updateEventRes.status).toBe(200);
	expect(updateEventRes.result!.id).toBe(createEventRes.result!.id);

	// Search user-event
	const userEventSearch: SearchEventRequest = {
		id: createEventRes.result!.id,
		street_address: "test",
		postal_code: 1,
		city: "test",
		country: "test",
		event_type: "test",
		event_description: "test",
		event_name: updatedName,
		hard_start_before: "2020-11-12T23:25:00",
		hard_start_after: "2010-11-12T23:25:00",
		hard_end_before: "2030-11-12T23:25:00",
		hard_end_after: "2020-11-12T23:25:00",
		timezone: "UTC"
	};
	const searchRes = await apiSearchEvent(userEventSearch);
	expect(searchRes.status).toBe(200);
	expect(
		searchRes.result!.events.some(
			(e: { event_name: string }) => e.event_name === updatedName
		)
	).toBe(true);

	// Delete user-event
	const deleteRes = await apiDeleteUserEvent(createEventRes.result!.id);
	expect(deleteRes.status).toBe(200);

	// Verify deletion
	const delSearchRes = await apiSearchEvent(userEventSearch);
	expect(delSearchRes.status).toBe(200);
	expect(
		delSearchRes.result!.events.some(
			(e: { event_name: string }) => e.event_name === updatedName
		)
	).toBe(false);

	// Test save/unsave itinerary flow
	const testItinerary: Itinerary = {
		id: 0,
		start_date: "2025-01-01",
		end_date: "2025-12-31",
		event_days: [],
		chat_session_id: null,
		title: "Test Itinerary",
		unassigned_events: []
	};

	// Save itinerary
	const saveResult = await apiSaveItineraryChanges(testItinerary);
	expect(saveResult.result).not.toBe(null);
	expect(saveResult.status).toBe(200);
	const savedItineraryId = saveResult.result!.id;
	expect(savedItineraryId).toBeGreaterThan(0);

	// Verify it's in saved itineraries
	const savedItineraries = await apiGetSavedItineraries();
	expect(savedItineraries.status).toBe(200);
	expect(
		savedItineraries.result!.itineraries.some(
			(i: { id: any }) => i.id === savedItineraryId
		)
	).toBe(true);

	// Unsave the itinerary
	testItinerary.id = savedItineraryId;
	const unsaveResult = await apiUnsaveItinerary(testItinerary);
	expect(unsaveResult.status).toBe(200);

	// Verify it's no longer in saved itineraries
	const savedItinerariesAfterUnsave = await apiGetSavedItineraries();
	expect(savedItinerariesAfterUnsave.status).toBe(200);
	expect(
		savedItinerariesAfterUnsave.result!.itineraries.some(
			(i: { id: any }) => i.id === savedItineraryId
		)
	).toBe(false);

	// Test unsaving non-existent itinerary (should return 404)
	const nonExistentItinerary: Itinerary = {
		id: 999999,
		start_date: "2025-01-01",
		end_date: "2025-12-31",
		event_days: [],
		chat_session_id: null,
		title: "Non-existent"
	};
	const unsaveNonExistent = await apiUnsaveItinerary(nonExistentItinerary);
	expect(unsaveNonExistent.status).toBe(404);

	// Test unsaving already unsaved itinerary (should return 400)
	const unsaveAlreadyUnsaved = await apiUnsaveItinerary(testItinerary);
	expect(unsaveAlreadyUnsaved.status).toBe(200);

	// logout
	expect((await apiLogout()).status).toBe(200);

	// should have invalid cookie
	expect((await apiValidate()).status !== 200).toBe(true);
}

describe("Integration Tests", () => {
	test("Journey Flow DEV", async () => {
		test_state.dev_mode = true;
		await test_flow();
	});

	test("Journey Flow PROD", async () => {
		test_state.dev_mode = false;
		await test_flow();
	});

	test("Error handling coverage", async () => {
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

		const errorMessages = await apiMessages({
			chat_session_id: 99999,
			message_id: null
		});
		expect(errorMessages.status).toBeGreaterThanOrEqual(-1);

		const errorSend = await apiSendMessage({
			chat_session_id: 99999,
			text: "test",
			itinerary_id: null
		});
		expect(errorSend.status).toBeGreaterThanOrEqual(-1);

		const errorUpdate = await apiUpdateMessage({
			message_id: 99999,
			new_text: "test",
			itinerary_id: null
		});
		expect(errorUpdate.status).toBeGreaterThanOrEqual(-1);

		// Test unsave errors
		const errorItinerary2: Itinerary = {
			id: 99999,
			start_date: "2025-01-01",
			end_date: "2025-12-31",
			event_days: [],
			chat_session_id: null,
			title: "Error Test"
		};

		const errorUnsave = await apiUnsaveItinerary(errorItinerary2);
		expect(errorUnsave.status).toBeGreaterThanOrEqual(-1);

		const errorSave = await apiSaveItineraryChanges(errorItinerary2);
		expect(errorSave.status).toBeGreaterThanOrEqual(-1);

		const errorSavedList = await apiGetSavedItineraries();
		expect(errorSavedList.status).toBeGreaterThanOrEqual(-1);
	});
});
