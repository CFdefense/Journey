import { Link, useNavigate } from "react-router-dom";
import { useState, useEffect } from "react";
import { apiGetSavedItineraries } from "../api/account";
import "../styles/Account.css";

interface Event {
  city: string;
  event_description: string;
  event_name: string;
  event_type: string;
  postal_code: number;
  street_address: string;
}

interface EventDay {
  date: string;
  morning_events: Event[];
  noon_events: Event[];
  afternoon_events: Event[];
  evening_events: Event[];
}

export default function Itineraries() {
  const navigate = useNavigate();
  const [eventDays, setEventDays] = useState<EventDay[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchItineraries();
  }, []);

  const fetchItineraries = async () => {
    try {
      const result = await apiGetSavedItineraries();
      const data = result.result;
      if (data){
        console.log("Fetched itineraries:", data);
        console.log("Event days:", data.itineraries);
        const allEventDays = data.itineraries.flatMap((itinerary: any) => {
            // Assuming each item in 'data.itineraries' has an 'event_days' array
            return itinerary.event_days || [];
        });
        console.log("Flattened Event Days for State:", allEventDays);
        setEventDays(allEventDays);
      }
    } catch (err) {
      console.error('Error fetching itineraries:', err);
      setError(err instanceof Error ? err.message : 'Failed to load itineraries');
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
    return 'Unknown Location';
  };

  const getTotalEvents = (day: EventDay) => {
    const total = (
      (day.morning_events?.length || 0) +
      (day.noon_events?.length || 0) +
      (day.afternoon_events?.length || 0) +
      (day.evening_events?.length || 0)
    );
    console.log("Total events:", total);
    return total;
  };

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

  const handleCardClick = (date: string) => {
    navigate('/view', { state: { date } });
  };

  return (
    <div className="auth-page auth-page--account">
      <nav className="auth-navbar">
        <div className="auth-navbar-content">
          <div style={{display: 'flex', gap: '16px', alignItems: 'center'}}>
            <Link to="/">Index</Link>
            <span>|</span>
            <Link to="/home">Home</Link>
            <span>|</span>
            <Link to="/view">View</Link>
          </div>
        </div>
      </nav>
      
      <div className="auth-content">
        <div className="account-wrapper">
          
          {/* Collapsible Sidebar */}
          <aside className="sidebar">
            <div className="sidebar-toggle">
              <span></span>
              <span></span>
              <span></span>
            </div>
            
            <div className="sidebar-content">
              <button 
                className="sidebar-item"
                onClick={() => navigate('/account')}
              >
                <div className="sidebar-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                    <circle cx="12" cy="7" r="4" />
                  </svg>
                </div>
                <span className="sidebar-label">Account Information</span>
              </button>
              
              <button 
                className="sidebar-item"
                onClick={() => navigate('/account/preferences')}
              >
                <div className="sidebar-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <circle cx="12" cy="12" r="3" />
                    <path d="M12 1v6m0 6v6m9-9h-6m-6 0H3" />
                  </svg>
                </div>
                <span className="sidebar-label">Preference Update</span>
              </button>
              
              <button 
                className="sidebar-item"
                onClick={() => navigate('/account/itineraries')}
              >
                <div className="sidebar-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
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
              <div className="account-box" style={{ maxWidth: '800px' }}>
                <h1>Saved Itineraries</h1>
                
                {loading ? (
                  <div className="empty-state">
                    <p>Loading itineraries...</p>
                  </div>
                ) : error ? (
                  <div className="empty-state">
                    <p style={{ color: '#dc3545' }}>{error}</p>
                  </div>
                ) : eventDays.length === 0 ? (
                  <div className="empty-state">
                    <p>No saved itineraries yet</p>
                    <p style={{fontSize: '0.9rem', marginTop: '8px'}}>Your saved travel plans will appear here</p>
                  </div>
                ) : (
                  <div className="itineraries-list">
                    {eventDays.map((day, index) => {
                      console.log("Rendering day:", day);
                      return (
                        <div 
                          key={index}
                          className="itinerary-card"
                          onClick={() => handleCardClick(day.date)}
                          style={{ cursor: 'pointer' }}
                        >
                          <h3>{getFirstEventName(day)}</h3>
                          <p><strong>Location:</strong> {getLocation(day)}</p>
                          <p><strong>Date:</strong> {day.date ? new Date(day.date).toLocaleDateString('en-US', { 
                            weekday: 'long', 
                            year: 'numeric', 
                            month: 'long', 
                            day: 'numeric' 
                          }) : 'Date not available'}</p>
                          <p><strong>Total Events:</strong> {getTotalEvents(day)}</p>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}