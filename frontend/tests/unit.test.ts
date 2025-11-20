import { describe, expect, test, vi, beforeEach } from "vitest";
import {
	checkIfValidName,
	checkIfValidPassword,
	checkIfPasswordsMatch
} from "../src/helpers/account";
import {
	fetchItinerary,
	populateItinerary,
	convertToApiFormat,
	sanitize,
	getTimeBlockFromTimestamp,
	getDateFromTimestamp,
	canDropEventInTimeBlock,
	getDropErrorMessage
} from "../src/helpers/itinerary";
import type {
	Itinerary as ApiItinerary,
	DayItinerary
} from "../src/models/itinerary";
import * as itineraryApi from "../src/api/itinerary";
import {
	apiLogin,
	apiSignUp,
	apiLogout,
	apiValidate,
	apiCurrent,
	apiUpdateAccount,
	apiChats,
	apiMessages,
	apiSendMessage,
	apiNewChatId,
	apiDeleteChat,
	apiRenameChat,
	apiItineraryDetails,
	apiSaveItineraryChanges,
	apiGetSavedItineraries,
	test_state
} from "./testApi";
import { customFetch } from "./customFetch";
import type { CurrentResponse } from "../src/models/account";
import type {
	ChatsResponse,
	MessagePageResponse,
	SendMessageResponse
} from "../src/models/chat";
import type {
	Itinerary,
	SaveResponse,
	SavedItinerariesResponse,
	Event
} from "../src/models/itinerary";

// Mock the API module
vi.mock("../src/api/itinerary");
// Mock customFetch for testApi tests
vi.mock("./customFetch");

describe("Unit Tests", () => {
	test("Test Names", () => {
		expect(checkIfValidName("First", "Last")).toBe(null);
		expect(checkIfValidName("", "Last")).toBe(
			"First and last name are required."
		);
		expect(checkIfValidName("First", "")).toBe(
			"First and last name are required."
		);
		expect(checkIfValidName("F".repeat(51), "Last")).toBe(
			"Names must be 50 characters or fewer."
		);
		expect(checkIfValidName("First", "L".repeat(51))).toBe(
			"Names must be 50 characters or fewer."
		);
	});
	test("Test Passwords", () => {
		expect(checkIfValidPassword("1234567")).toBe(
			"Password must be at least 8 characters long."
		);
		expect(checkIfValidPassword("a".repeat(129))).toBe(
			"Password must be 128 characters or fewer."
		);
		expect(checkIfValidPassword("12345678\u{1F600}")).toBe(
			"Password must contain only ASCII characters."
		);
		expect(checkIfValidPassword("abcdefgh")).toBe(
			"Password must contain at least one uppercase letter."
		);
		expect(checkIfValidPassword("ABCDEFGH")).toBe(
			"Password must contain at least one lowercase letter."
		);
		expect(checkIfValidPassword("ABCDefgh")).toBe(
			"Password must contain at least one number."
		);
		expect(checkIfValidPassword("ABCdef123")).toBe(null);
	});
	test("Test Passwords Match", () => {
		expect(checkIfPasswordsMatch("ABCdef123", "abcDEF123")).toBe(
			"Passwords do not match."
		);
		expect(checkIfPasswordsMatch("ABCdef123", "ABCdef123")).toBe(null);
	});
	test("Input sanitizer", () => {
		expect(sanitize("")).toBeNull();
		expect(sanitize(null)).toBeNull();
		expect(sanitize(" trim ")).toBe("trim");
		expect(sanitize("valid string")).toBe("valid string");
	});
});

describe("Itinerary Helper Tests", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	test("Test populateItinerary with valid data", () => {
		const apiItinerary: ApiItinerary = {
			id: 1,
			start_date: "2025-01-01",
			end_date: "2025-01-01",
			chat_session_id: null,
			title: "Test Trip",
			event_days: [
				{
					date: "2025-01-01",
					morning_events: [
						{
							id: 1,
							event_name: "Breakfast",
							event_description: "Morning meal",
							street_address: "123 Main St",
							postal_code: 12345,
							city: "TestCity",
							country: "TestCountry",
							event_type: "food",
							user_created: false,
							hard_start: null,
							hard_end: null,
							timezone: null
						}
					],
					afternoon_events: [],
					evening_events: []
				}
			]
		};

		const result = populateItinerary(apiItinerary);
		expect(result).toHaveLength(1);
		expect(result[0].date).toBe("2025-01-01");
		expect(result[0].timeBlocks[0].events[0].event_name).toBe("Breakfast");
	});

	test("Test populateItinerary with empty event_days", () => {
		const apiItinerary: ApiItinerary = {
			id: 1,
			start_date: "2025-01-01",
			end_date: "2025-01-01",
			chat_session_id: null,
			title: "Test Trip",
			event_days: []
		};

		const result = populateItinerary(apiItinerary);
		expect(result[0].date).toBe("");
	});

	test("Test convertToApiFormat", () => {
		const days: DayItinerary[] = [
			{
				date: "2025-01-01",
				timeBlocks: [
					{
						time: "Morning",
						events: [
							{
								id: 1,
								event_name: "Breakfast",
								event_description: "Morning meal",
								street_address: "123 Main St",
								postal_code: 12345,
								city: "New York",
								country: "USA",
								event_type: "dining",
								user_created: false,
								hard_start: null,
								hard_end: null,
								timezone: null
							}
						]
					},
					{
						time: "Afternoon",
						events: []
					},
					{
						time: "Evening",
						events: []
					}
				]
			}
		];

		const result = convertToApiFormat(
			days,
			1,
			"2025-01-01",
			"2025-01-01",
			"Test Trip",
			100
		);

		expect(result.id).toBe(1);
		expect(result.start_date).toBe("2025-01-01");
		expect(result.end_date).toBe("2025-01-01");
		expect(result.title).toBe("Test Trip");
		expect(result.chat_session_id).toBe(100);
		expect(result.event_days[0].date).toBe("2025-01-01");
		expect(result.event_days[0].morning_events[0].event_name).toBe(
			"Breakfast"
		);
		expect(result.event_days[0].morning_events[0].event_description).toBe(
			"Morning meal"
		);
		expect(result.event_days[0].morning_events[0].city).toBe("New York");
	});

	test("Test fetchItinerary success", async () => {
		const mockApiResponse = {
			status: 200,
			result: {
				id: 1,
				start_date: "2025-01-01",
				end_date: "2025-01-01",
				chat_session_id: null,
				title: "Test Trip",
				event_days: [
					{
						date: "2025-01-01",
						morning_events: [],
						noon_events: [],
						afternoon_events: [],
						evening_events: []
					}
				]
			}
		};

		vi.mocked(itineraryApi.apiItineraryDetails).mockResolvedValue(
			mockApiResponse
		);

		const result = await fetchItinerary(1);
		expect(result).toHaveLength(1);
	});

	test("Test fetchItinerary with null result", async () => {
		const mockApiResponse = {
			status: 404,
			result: null
		};

		vi.mocked(itineraryApi.apiItineraryDetails).mockResolvedValue(
			mockApiResponse
		);

		const result = await fetchItinerary(1);
		expect(result[0].date).toBe("");
	});

	test("Test fetchItinerary with error", async () => {
		vi.mocked(itineraryApi.apiItineraryDetails).mockRejectedValue(
			new Error("Network error")
		);

		const result = await fetchItinerary(1);
		expect(result[0].date).toBe("");
	});

	test("Test fetchItinerary with error", async () => {
		vi.mocked(itineraryApi.apiItineraryDetails).mockRejectedValue(new Error("Network error"));

		const result = await fetchItinerary(1);
		expect(result[0].date).toBe("");
	});
	
	test("Test getTimeBlockFromTimestamp - Morning", () => {
		expect(getTimeBlockFromTimestamp("2025-01-01T08:00:00")).toBe("Morning");
		expect(getTimeBlockFromTimestamp("2025-01-01T04:00:00Z")).toBe("Morning");
		expect(getTimeBlockFromTimestamp("2025-01-01T11:59:59Z")).toBe("Morning");
	});

	test("Test getTimeBlockFromTimestamp - Afternoon", () => {
		expect(getTimeBlockFromTimestamp("2025-01-01T12:00:00")).toBe("Afternoon");
		expect(getTimeBlockFromTimestamp("2025-01-01T15:30:00Z")).toBe("Afternoon");
		expect(getTimeBlockFromTimestamp("2025-01-01T17:59:59Z")).toBe("Afternoon");
	});

	test("Test getTimeBlockFromTimestamp - Evening", () => {
		expect(getTimeBlockFromTimestamp("2025-01-01T18:00:00")).toBe("Evening");
		expect(getTimeBlockFromTimestamp("2025-01-01T22:00:00Z")).toBe("Evening");
		expect(getTimeBlockFromTimestamp("2025-01-01T03:59:59Z")).toBe("Evening");
	});

	test("Test getTimeBlockFromTimestamp - Invalid timestamp", () => {
		expect(getTimeBlockFromTimestamp("invalid")).toBe(null);
		expect(getTimeBlockFromTimestamp("")).toBe(null);
	});

	test("Test getDateFromTimestamp - Valid timestamps", () => {
		expect(getDateFromTimestamp("2025-01-01T08:00:00")).toBe("2025-01-01");
		expect(getDateFromTimestamp("2025-01-01T08:00:00Z")).toBe("2025-01-01");
		expect(getDateFromTimestamp("2025-12-31T23:59:59Z")).toBe("2025-12-31");
	});

	test("Test getDateFromTimestamp - Invalid timestamp", () => {
		expect(getDateFromTimestamp("invalid")).toBe("");
		expect(getDateFromTimestamp("")).toBe("");
	});

	test("Test canDropEventInTimeBlock - No hard_start", () => {
		const event: Event = {
		id: 1,
		event_name: "Flexible Event",
		event_description: "Can move anywhere",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: null,
		hard_end: null,
		timezone: null
		};
		
		expect(canDropEventInTimeBlock(event, "Morning", "2025-01-01", 0)).toBe(true);
		expect(canDropEventInTimeBlock(event, "Afternoon", "2025-01-02", 1)).toBe(true);
	});

	test("Test canDropEventInTimeBlock - Unassigned events (targetTimeIndex -1)", () => {
		const event: Event = {
		id: 1,
		event_name: "Any Event",
		event_description: "Test",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: "2025-01-01T08:00:00Z",
		hard_end: null,
		timezone: null
		};
		
		expect(canDropEventInTimeBlock(event, "Afternoon", "2025-01-02", -1)).toBe(true);
	});

	test("Test canDropEventInTimeBlock - Matching time block and date", () => {
		const event: Event = {
		id: 1,
		event_name: "Fixed Event",
		event_description: "Must be at specific time",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: "2025-01-01T08:00:00Z",
		hard_end: null,
		timezone: null
		};
		
		expect(canDropEventInTimeBlock(event, "Morning", "2025-01-01", 0)).toBe(true);
	});

	test("Test canDropEventInTimeBlock - Wrong time block", () => {
		const event: Event = {
		id: 1,
		event_name: "Fixed Event",
		event_description: "Must be in morning",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: "2025-01-01T08:00:00Z",
		hard_end: null,
		timezone: null
		};
		
		expect(canDropEventInTimeBlock(event, "Afternoon", "2025-01-01", 1)).toBe(false);
	});

	test("Test canDropEventInTimeBlock - Wrong date", () => {
		const event: Event = {
		id: 1,
		event_name: "Fixed Event",
		event_description: "Must be on specific date",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: "2025-01-01T08:00:00Z",
		hard_end: null,
		timezone: null
		};
		
		expect(canDropEventInTimeBlock(event, "Morning", "2025-01-02", 0)).toBe(false);
	});

	test("Test getDropErrorMessage - Event with hard_start", () => {
		const event: Event = {
		id: 1,
		event_name: "Concert",
		event_description: "Must attend at specific time",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "entertainment",
		user_created: false,
		hard_start: "2025-01-01T19:00:00Z",
		hard_end: null,
		timezone: null
		};
		
		const message = getDropErrorMessage(event);
		expect(message).toBe('"Concert" has a fixed start time and must be placed in the Evening block on 2025-01-01.');
	});

	test("Test getDropErrorMessage - Event without hard_start", () => {
		const event: Event = {
		id: 1,
		event_name: "Flexible Event",
		event_description: "Can be anywhere",
		street_address: "123 Main St",
		postal_code: 12345,
		city: "TestCity",
		country: "TestCountry",
		event_type: "food",
		user_created: false,
		hard_start: null,
		hard_end: null,
		timezone: null
		};
		
		expect(getDropErrorMessage(event)).toBe(null);
	});
});

describe("testApi Unit Tests", () => {
	const mockCurrentResponse: CurrentResponse = {
		email: "test@test.com",
		first_name: "Test",
		last_name: "User",
		budget_preference: null,
		risk_preference: null,
		food_allergies: "",
		disabilities: ""
	};

	const mockChatsResponse: ChatsResponse = {
		chat_sessions: []
	};

	const mockMessagePageResponse: MessagePageResponse = {
		prev_message_id: null,
		message_page: [
			{
				id: 1,
				timestamp: "2025-01-01T10:00:00",
				text: "Hello",
				is_user: true,
				itinerary_id: null
			}
		]
	};

	const mockSendMessageResponse: SendMessageResponse = {
		user_message_id: 1,
		bot_message: {
			id: 2,
			timestamp: "2025-01-01T10:00:00",
			text: "Response",
			is_user: false,
			itinerary_id: null
		}
	};

	const mockItinerary: Itinerary = {
		id: 1,
		start_date: "2025-01-01",
		end_date: "2025-01-01",
		chat_session_id: null,
		title: "Test Trip",
		event_days: []
	};

	const mockSaveResponse: SaveResponse = {
		id: 1
	};

	const mockSavedItinerariesResponse: SavedItinerariesResponse = {
		itineraries: []
	};

	beforeEach(() => {
		vi.clearAllMocks();
		test_state.dev_mode = true;
	});

	test("apiLogin success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiLogin({
			email: "test@test.com",
			password: "Password123"
		});
		expect(result.status).toBe(200);
	});

	test("apiLogin error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiLogin({
			email: "test@test.com",
			password: "Password123"
		});
		expect(result.status).toBe(-1);
	});

	test("apiSignUp success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiSignUp({
			email: "test@test.com",
			first_name: "Test",
			last_name: "User",
			password: "Password123"
		});
		expect(result.status).toBe(200);
	});

	test("apiSignUp error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiSignUp({
			email: "test@test.com",
			first_name: "Test",
			last_name: "User",
			password: "Password123"
		});
		expect(result.status).toBe(-1);
	});

	test("apiLogout success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiLogout();
		expect(result.status).toBe(200);
	});

	test("apiLogout error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiLogout();
		expect(result.status).toBe(-1);
	});

	test("apiValidate success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiValidate();
		expect(result.status).toBe(200);
	});

	test("apiValidate error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiValidate();
		expect(result.status).toBe(-1);
	});

	test("apiUpdateAccount success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockCurrentResponse
		} as Response);
		const result = await apiUpdateAccount({
			email: null,
			first_name: null,
			last_name: null,
			password: null,
			current_password: null,
			budget_preference: null,
			risk_preference: null,
			food_allergies: null,
			disabilities: null
		});
		expect(result.status).toBe(200);
		expect(result.result).toEqual(mockCurrentResponse);
	});

	test("apiUpdateAccount error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiUpdateAccount({
			email: null,
			first_name: null,
			last_name: null,
			password: null,
			current_password: null,
			budget_preference: null,
			risk_preference: null,
			food_allergies: null,
			disabilities: null
		});
		expect(result.status).toBe(-1);
	});

	test("apiCurrent success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockCurrentResponse
		} as Response);
		const result = await apiCurrent();
		expect(result.status).toBe(200);
		expect(result.result).toEqual(mockCurrentResponse);
	});

	test("apiCurrent not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 401,
			ok: false
		} as Response);
		const result = await apiCurrent();
		expect(result.status).toBe(401);
		expect(result.result).toBeNull();
	});

	test("apiCurrent error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiCurrent();
		expect(result.status).toBe(-1);
	});

	test("apiChats success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockChatsResponse
		} as Response);
		const result = await apiChats();
		expect(result.status).toBe(200);
		expect(result.result).toEqual(mockChatsResponse);
	});

	test("apiChats not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 401,
			ok: false
		} as Response);
		const result = await apiChats();
		expect(result.status).toBe(401);
		expect(result.result).toBeNull();
	});

	test("apiChats error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiChats();
		expect(result.status).toBe(-1);
	});

	test("apiMessages success with timestamp processing", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockMessagePageResponse
		} as Response);
		const result = await apiMessages({
			chat_session_id: 1,
			message_id: null
		});
		expect(result.status).toBe(200);
		expect(result.result?.message_page[0].timestamp).toBe(
			"2025-01-01T10:00:00Z"
		);
	});

	test("apiMessages not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 404,
			ok: false
		} as Response);
		const result = await apiMessages({
			chat_session_id: 1,
			message_id: null
		});
		expect(result.status).toBe(404);
		expect(result.result).toBeNull();
	});

	test("apiMessages error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiMessages({
			chat_session_id: 1,
			message_id: null
		});
		expect(result.status).toBe(-1);
	});

	test("apiSendMessage success with timestamp processing", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockSendMessageResponse
		} as Response);
		const result = await apiSendMessage({
			chat_session_id: 1,
			text: "Hello",
			itinerary_id: null
		});
		expect(result.status).toBe(200);
		expect(result.result?.bot_message.timestamp).toBe(
			"2025-01-01T10:00:00Z"
		);
	});

	test("apiSendMessage not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 400,
			ok: false
		} as Response);
		const result = await apiSendMessage({
			chat_session_id: 1,
			text: "Hello",
			itinerary_id: null
		});
		expect(result.status).toBe(400);
		expect(result.result).toBeNull();
	});

	test("apiSendMessage error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiSendMessage({
			chat_session_id: 1,
			text: "Hello",
			itinerary_id: null
		});
		expect(result.status).toBe(-1);
	});

	test("apiNewChatId success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => ({ chat_session_id: 123 })
		} as Response);
		const result = await apiNewChatId();
		expect(result.status).toBe(200);
		expect(result.result).toBe(123);
	});

	test("apiNewChatId not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 401,
			ok: false
		} as Response);
		const result = await apiNewChatId();
		expect(result.status).toBe(401);
		expect(result.result).toBeNull();
	});

	test("apiNewChatId error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiNewChatId();
		expect(result.status).toBe(-1);
	});

	test("apiDeleteChat success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiDeleteChat(123);
		expect(result.status).toBe(200);
		expect(result.result).toBeNull();
	});

	test("apiDeleteChat not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 404,
			ok: false
		} as Response);
		const result = await apiDeleteChat(123);
		expect(result.status).toBe(404);
		expect(result.result).toBeNull();
	});

	test("apiDeleteChat error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiDeleteChat(123);
		expect(result.status).toBe(-1);
	});

	test("apiRenameChat success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true
		} as Response);
		const result = await apiRenameChat({ id: 123, new_title: "New Title" });
		expect(result.status).toBe(200);
		expect(result.result).toBeNull();
	});

	test("apiRenameChat not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 404,
			ok: false
		} as Response);
		const result = await apiRenameChat({ id: 123, new_title: "New Title" });
		expect(result.status).toBe(404);
		expect(result.result).toBeNull();
	});

	test("apiRenameChat error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiRenameChat({ id: 123, new_title: "New Title" });
		expect(result.status).toBe(-1);
	});

	test("apiItineraryDetails success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockItinerary
		} as Response);
		const result = await apiItineraryDetails(1);
		expect(result.status).toBe(200);
		expect(result.result).toEqual(mockItinerary);
	});

	test("apiItineraryDetails error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiItineraryDetails(1);
		expect(result.status).toBe(-1);
	});

	test("apiSaveItineraryChanges success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockSaveResponse
		} as Response);
		const result = await apiSaveItineraryChanges({
			id: 1,
			start_date: "2025-01-01",
			end_date: "2025-01-01",
			chat_session_id: null,
			title: "Test Trip",
			event_days: []
		});
		expect(result.result).toEqual(mockSaveResponse);
	});

	test("apiSaveItineraryChanges not ok", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 400,
			ok: false,
			text: async () => "Bad Request"
		} as Response);
		expect(
			(
				await apiSaveItineraryChanges({
					id: 1,
					start_date: "2025-01-01",
					end_date: "2025-01-01",
					chat_session_id: null,
					title: "Test Trip",
					event_days: []
				})
			).status
		).toBe(400);
	});

	test("apiSaveItineraryChanges error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		expect(
			(
				await apiSaveItineraryChanges({
					id: 1,
					start_date: "2025-01-01",
					end_date: "2025-01-01",
					chat_session_id: null,
					title: "Test Trip",
					event_days: []
				})
			).status
		).toBe(-1);
	});

	test("apiGetSavedItineraries success", async () => {
		vi.mocked(customFetch).mockResolvedValue({
			status: 200,
			ok: true,
			json: async () => mockSavedItinerariesResponse
		} as Response);
		const result = await apiGetSavedItineraries();
		expect(result.status).toBe(200);
		expect(result.result).toEqual(mockSavedItinerariesResponse);
	});

	test("apiGetSavedItineraries error", async () => {
		vi.mocked(customFetch).mockRejectedValue(new Error("Network error"));
		const result = await apiGetSavedItineraries();
		expect(result.status).toBe(-1);
	});
});
