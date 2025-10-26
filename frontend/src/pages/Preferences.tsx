import { Link, useNavigate } from "react-router-dom";
import { useContext, useState, type Context } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";
import "../styles/Account.css";

export default function Preferences() {
  const navigate = useNavigate();
  
  const [statusMessage, setStatusMessage] = useState<{type: 'success' | 'error', message: string} | null>(null);
  const [budget, setBudget] = useState("Medium");
  const [riskTolerance, setRiskTolerance] = useState("Medium");
  const [disabilities, setDisabilities] = useState("");
  const [foodPreferences, setFoodPreferences] = useState("");

  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    // Your update logic here
    setStatusMessage({type: 'success', message: 'Preferences updated successfully!'});
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

          {/* Main Content */}
          <main className="main-content">
            <div className="account-container">
              <div className="account-box">
                <h1>Activity Preferences</h1>
                
                {statusMessage && (
                  <div className={`status-message status-message--${statusMessage.type}`}>
                    {statusMessage.message}
                  </div>
                )}

                <form onSubmit={handleUpdate}>
                  <div className="settings-section">
                    <div className="field-group">
                      <label htmlFor="budget">Budget:</label>
                      <select
                        id="budget"
                        value={budget}
                        onChange={(e) => setBudget(e.target.value)}
                      >
                        <option value="Low">Low</option>
                        <option value="Medium">Medium</option>
                        <option value="High">High</option>
                      </select>
                    </div>

                    <div className="field-group">
                      <label htmlFor="riskTolerance">Risk Tolerance:</label>
                      <select
                        id="riskTolerance"
                        value={riskTolerance}
                        onChange={(e) => setRiskTolerance(e.target.value)}
                      >
                        <option value="Low">Low</option>
                        <option value="Medium">Medium</option>
                        <option value="High">High</option>
                      </select>
                    </div>

                    <div className="field-group">
                      <label htmlFor="disabilities">Disabilities/Accessibility Needs:</label>
                      <textarea
                        id="disabilities"
                        value={disabilities}
                        onChange={(e) => setDisabilities(e.target.value)}
                        placeholder="e.g., Wheelchair user, needs assistance with stairs, visual impairment."
                      />
                    </div>

                    <div className="field-group">
                      <label htmlFor="foodPreferences">Food Preferences/Allergies:</label>
                      <textarea
                        id="foodPreferences"
                        value={foodPreferences}
                        onChange={(e) => setFoodPreferences(e.target.value)}
                        placeholder="e.g., Gluten-free, no shellfish, vegan, prefers Italian cuisine."
                      />
                    </div>
                  </div>

                  <button type="submit" className="btn-primary">
                    Update All Settings
                  </button>
                </form>
              </div>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}