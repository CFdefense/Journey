import { useNavigate } from "react-router-dom";
import { useState, useEffect } from "react";
import { apiCurrent } from "../api/account";
import { apiGetSavedItineraries } from "../api/itinerary";
import Navbar from "../components/Navbar";
import { useLocation } from "react-router-dom";
import "../styles/Account.css";
import type { EventDay, Itinerary } from "../models/itinerary";
import userPfp from "../assets/user-pfp-temp.png";

export default function Itineraries() {
  const navigate = useNavigate();
  const location = useLocation();
  const [itineraries, setItineraries] = useState<Itinerary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [firstName, setFirstName] = useState<string>("");
  const navbarAvatarUrl = userPfp;
  const [profileImageUrl, setProfileImageUrl] = useState<string>(navbarAvatarUrl);
  const [tripsPlanned, setTripsPlanned] = useState<number | null>(null);
  const [accountCreated, setAccountCreated] = useState<string | null>(null);

  useEffect(() => {
    fetchItineraries();
    async function fetchAccount() {
      const currentResult = await apiCurrent();
      const account = currentResult.result;

      if (account && currentResult.status === 200) {
        setFirstName(account.first_name || "");
        const maybeTrips = (account as any).trips_planned;
        setTripsPlanned(typeof maybeTrips === "number" ? maybeTrips : 5);
        const maybeCreated = (account as any).created_at;
        setAccountCreated(
          maybeCreated
            ? new Date(maybeCreated).toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
                year: "numeric"
              })
            : new Date().toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
                year: "numeric"
              })
        );
      }
    }
    fetchAccount();
  }, []);

  const fetchItineraries = async () => {
    try {
      const result = await apiGetSavedItineraries();
      const data = result.result;
      if (data && data.itineraries) {
        // Check for the itineraries array
        console.log("Fetched itineraries:", data.itineraries);

        // --- THIS IS THE KEY CHANGE ---
        // Don't flatten the array, just set the itineraries
        setItineraries(data.itineraries as Itinerary[]);
        // -----------------------------
      }
    } catch (err) {
      console.error("Error fetching itineraries:", err);
      setError(
        err instanceof Error ? err.message : "Failed to load itineraries"
      );
    } finally {
      setLoading(false);
    }
  };

  const getLocation = (day: EventDay) => {
    // Check all event arrays for the first event with a city
    const allEvents = [
      ...(day.afternoon_events || []),
      ...(day.evening_events || []),
      ...(day.morning_events || []),
      ...(day.noon_events || [])
    ];

    console.log("All events for location:", allEvents);

    if (allEvents.length > 0 && allEvents[0]?.city) {
      return allEvents[0].city;
    }
    return "Unknown Location";
  };

  const getTotalEvents = (day: EventDay) => {
    const total =
      (day.morning_events?.length || 0) +
      (day.noon_events?.length || 0) +
      (day.afternoon_events?.length || 0) +
      (day.evening_events?.length || 0);
    console.log("Total events:", total);
    return total;
  };
  /*
  const getFirstEventName = (day: EventDay) => {
    const allEvents = [
      ...(day.afternoon_events || []),
      ...(day.evening_events || []),
      ...(day.morning_events || []),
      ...(day.noon_events || [])
    ];

    console.log("All events for name:", allEvents);
    console.log("date:", day);

    if (allEvents.length > 0 && allEvents[0]?.event_name) {
      return allEvents[0].event_name;
    }
    return 'Day Trip';
  };
  */
  const handleCardClick = (itineraryId: number) => {
    navigate("/view", { state: { itineraryId: itineraryId } });
  };
  return (
    <div className="auth-page auth-page--account">
      <Navbar page="view" firstName={firstName} />
      <div className="auth-content">
        <div className="account-wrapper">
          {/* Collapsible Sidebar ... (all your sidebar code is fine) ... */}
          <aside className="sidebar">
            {/* ... all your sidebar buttons ... */}
            <div className="sidebar-content">
              <button
                className="sidebar-item"
                onClick={() => navigate("/account")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </div>
                <span className="sidebar-label">Account Information</span>
              </button>

              <button
                className="sidebar-item"
                onClick={() => navigate("/account/preferences")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <circle cx="12" cy="12" r="3" />
                    <path d="M12 1v6m0 6v6m9-9h-6m-6 0H3" />
                  </svg>
                </div>
                <span className="sidebar-label">Preference Update</span>
              </button>

              <button
                className="sidebar-item"
                onClick={() => navigate("/account/itineraries")}
              >
                <div className="sidebar-icon">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <polyline points="14 2 14 8 20 8" />
                    <line x1="16" y1="13" x2="8" y2="13" />
                    <line x1="16" y1="17" x2="8" y2="17" />
                    <polyline points="10 9 9 9 8 9" />
                  </svg>
                </div>
                <span className="sidebar-label">Saved Itineraries</span>
              </button>
            </div>
          </aside>

          <main className="main-content">
            <div className="account-container">
              <div className="account-box" style={{ maxWidth: "800px" }}>
                <div className="hs-hero-card">
                  <div className="profile-header">
                    <div className="avatar-wrapper">
                      <img
                        src={profileImageUrl}
                        alt={`${firstName || "User"}`}
                        className="avatar"
                        onError={() => setProfileImageUrl(navbarAvatarUrl)}
                      />
                    </div>
                    <div className="profile-meta">
                      <h1 className="profile-name">
                        {firstName || "Your Name"}
                      </h1>
                      <p className="profile-email">Saved Itineraries</p>
                    </div>
                  </div>
                  <div className="hs-stats">
                    <div className="hs-stat">
                      <div className="hs-stat__value">
                        {tripsPlanned ?? 5}
                      </div>
                      <div className="hs-stat__label">Trips planned</div>
                    </div>
                    <div className="hs-stat">
                      <div className="hs-stat__value">{accountCreated}</div>
                      <div className="hs-stat__label">Account created</div>
                    </div>
                  </div>
                </div>

                {loading ? (
                  <div className="empty-state">
                    <p>Loading itineraries...</p>
                  </div>
                ) : error ? (
                  <div className="empty-state">
                    <p style={{ color: "#dc3545" }}>{error}</p>
                  </div>
                ) : itineraries.length === 0 ? (
                  <div className="empty-state">
                    <p>No saved itineraries found</p>
                    <p
                      style={{
                        fontSize: "0.9rem",
                        marginTop: "8px",
                        marginBottom: "16px"
                      }}
                    >
                      Your saved travel plans will appear here
                    </p>
                    <button
                      className="btn-primary"
                      onClick={() => navigate("/home")}
                    >
                      Create Itinerary
                    </button>
                  </div>
                ) : (
                  <div className="itineraries-list">
                    {/* --- CHANGE 2: Map over itineraries --- */}
                    {itineraries.map((itinerary) => {
                      {
                        /* Safety check: get the first day, if it exists */
                      }
                      const firstDay = itinerary.event_days?.[0];
                      return (
                        <div
                          key={itinerary.id}
                          className="itinerary-card"
                          onClick={() => handleCardClick(itinerary.id)}
                          style={{ cursor: "pointer" }}
                        >
                          <h3>{itinerary.title || "My Itinerary"}</h3>
                          {firstDay && (
                            <>
                              <p>
                                <strong>Location:</strong>{" "}
                                {getLocation(firstDay)}
                              </p>
                              <p>
                                <strong>Total Events (Day 1):</strong>{" "}
                                {getTotalEvents(firstDay)}
                              </p>
                            </>
                          )}
                          <p>
                            <strong>Dates:</strong>{" "}
                            {itinerary.start_date
                              ? `${new Date(itinerary.start_date).toLocaleDateString()} - ${new Date(itinerary.end_date).toLocaleDateString()}`
                              : "Date not available"}
                          </p>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          </main>
        </div>
        {/* Bottom tab bar */}
        <footer className="account-bottom-bar">
          <div className="account-bottom-inner">
            <button
              type="button"
              className={`bottom-tab ${location.pathname === "/account" ? "active" : ""}`}
              onClick={() => navigate("/account")}
            >
              Account
            </button>
            <button
              type="button"
              className={`bottom-tab ${location.pathname.includes("/account/preferences") ? "active" : ""}`}
              onClick={() => navigate("/account/preferences")}
            >
              Preferences
            </button>
            <button
              type="button"
              className={`bottom-tab ${location.pathname.includes("/account/itineraries") ? "active" : ""}`}
              onClick={() => navigate("/account/itineraries")}
            >
              Itineraries
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
