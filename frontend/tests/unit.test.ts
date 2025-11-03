import { describe, expect, test,  vi, beforeEach } from "vitest";
import { checkIfValidName, checkIfValidPassword, checkIfPasswordsMatch } from "../src/helpers/account";
import { 
  fetchItinerary, 
  populateItinerary, 
  convertToApiFormat 
} from "../src/helpers/itinerary";
import type { Itinerary as ApiItinerary } from "../src/models/itinerary";
import * as itineraryApi from "../src/api/itinerary";

// Mock the API module
vi.mock("../src/api/itinerary");

describe("Unit Tests", () => {
	test("Test Names", () => {
		expect(checkIfValidName("First", "Last")).toBe(null);
		expect(checkIfValidName("", "Last")).toBe("First and last name are required.");
		expect(checkIfValidName("First", "")).toBe("First and last name are required.");
		expect(checkIfValidName("F".repeat(51), "Last")).toBe("Names must be 50 characters or fewer.");
		expect(checkIfValidName("First", "L".repeat(51))).toBe("Names must be 50 characters or fewer.");
	});
	test("Test Passwords", () => {
		expect(checkIfValidPassword("1234567")).toBe("Password must be at least 8 characters long.");
		expect(checkIfValidPassword("a".repeat(129))).toBe("Password must be 128 characters or fewer.");
		expect(checkIfValidPassword("12345678\u{1F600}")).toBe("Password must contain only ASCII characters.");
		expect(checkIfValidPassword("abcdefgh")).toBe("Password must contain at least one uppercase letter.");
		expect(checkIfValidPassword("ABCDEFGH")).toBe("Password must contain at least one lowercase letter.");
		expect(checkIfValidPassword("ABCDefgh")).toBe("Password must contain at least one number.");
		expect(checkIfValidPassword("ABCdef123")).toBe(null);
	});
	test("Test Passwords Match", () => {
		expect(checkIfPasswordsMatch("ABCdef123", "abcDEF123")).toBe("Passwords do not match.");
		expect(checkIfPasswordsMatch("ABCdef123", "ABCdef123")).toBe(null);
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
              event_type: "food",
              user_created: false,
              account_id: null,
              hard_start: null,
              hard_end: null
            },
          ],
          noon_events: [],
          afternoon_events: [],
          evening_events: [],
        },
      ],
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
      event_days: [],
    };

    const result = populateItinerary(apiItinerary);
    expect(result[0].date).toBe("N/A");
  });

 test("Test convertToApiFormat", () => {
  const days = [
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
              event_type: "dining",
              user_created: false,
              account_id: null,
              hard_start: null,
              hard_end: null,
            },
          ],
        },
        {
          time: "Afternoon",
          events: [],
        },
        {
          time: "Evening",
          events: [],
        },
      ],
    },
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
  expect(result.event_days[0].morning_events[0].event_name).toBe("Breakfast");
  expect(result.event_days[0].morning_events[0].event_description).toBe("Morning meal");
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
            evening_events: [],
          },
        ],
      },
    };

    vi.mocked(itineraryApi.apiItineraryDetails).mockResolvedValue(mockApiResponse);

    const result = await fetchItinerary(1);
    expect(result).toHaveLength(1);
  });

  test("Test fetchItinerary with null result", async () => {
    const mockApiResponse = {
      status: 404,
      result: null,
    };

    vi.mocked(itineraryApi.apiItineraryDetails).mockResolvedValue(mockApiResponse);

    const result = await fetchItinerary(1);
    expect(result[0].date).toBe("N/A");
  });

  test("Test fetchItinerary with error", async () => {
    vi.mocked(itineraryApi.apiItineraryDetails).mockRejectedValue(new Error("Network error"));

    const result = await fetchItinerary(1);
    expect(result[0].date).toBe("N/A");
  });
});