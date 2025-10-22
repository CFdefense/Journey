import { Link } from "react-router-dom";
// Import all necessary API functions and types
import { 
  apiLogout, 
  apiUpdateAccount, 
  apiGetProfile, 
  type UpdateRequest, 
  type UpdateResponse 
} from "../api/account";
import { useContext, type Context, useState, useEffect } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";

// Define the types for the preference options for clarity
type BudgetOption = 'VeryLowBudget' | 'LowBudget' | 'MediumBudget' | 'HighBudget' | 'LuxuryBudget';
type RiskOption = 'ChillVibes' | 'LightFun' | 'Adventurer' | 'RiskTaker';

export default function Account() {
  // --- Component State ---
  // The 'username' field (which corresponds to 'email' in the API) will be disabled for editing
  // as the API uses 'email' but the form shows 'username'. Assuming 'username' is what the user sees.
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [budget, setBudget] = useState<BudgetOption>("MediumBudget");
  const [riskTolerance, setRiskTolerance] = useState<RiskOption>("LightFun");
  const [disabilities, setDisabilities] = useState("");
  const [foodPreferences, setFoodPreferences] = useState("");
  const [statusMessage, setStatusMessage] = useState<{ message: string, type: 'success' | 'error' } | null>(null);

  const { setAuthorized } = useContext<GlobalState>(GlobalContext as Context<GlobalState>);

  // --- Utility Data ---
  const budgetOptions: BudgetOption[] = ['VeryLowBudget', 'LowBudget', 'MediumBudget', 'HighBudget', 'LuxuryBudget'];
  const riskOptions: RiskOption[] = ['ChillVibes', 'LightFun', 'Adventurer', 'RiskTaker'];

  // --- Fetch Initial Data on Load ---
  useEffect(() => {
    const fetchProfile = async () => {
      try {
        const data: UpdateResponse = await apiGetProfile();
        
        // Populate state from API response
        setEmail(data.email || "");
        
        // The password should NEVER be returned from the API, so we leave it empty for a new input
        setPassword(""); 

        // Populate preferences, converting nulls to empty strings or default enum values
        setBudget((data.budget_preference as BudgetOption) || "MediumBudget");
        setRiskTolerance((data.risk_preference as RiskOption) || "LightFun");
        setDisabilities(data.disabilities || "");
        setFoodPreferences(data.food_allergies || "");
        
      } catch (e) {
        console.error("Failed to load profile:", e);
        setStatusMessage({ message: "Failed to load account details. Please log in again.", type: 'error' });
      }
    };

    fetchProfile();
  }, [setAuthorized]);


  // --- Logout Logic (unchanged) ---
  const onLogout = async () => {
    console.log("Logging out");
    try {
      await apiLogout();
    } catch (e) {
      console.error("Logout error:", e);
    } finally {
      window.location.href = "/"; 
      setAuthorized(false);
    }
  };

  // --- Handle Update Logic (The new core function) ---
  const handleUpdate = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatusMessage(null); // Clear previous messages

    const payload: UpdateRequest = {
      // Username is typically the email in backend, we should not change it 
      // unless we know the API supports it, so we exclude it or ensure it's not editable.
      // email: email, // Assuming email is the username field and is read-only
      
      // Only include password if the user typed something new
      ...(password && { password: password }), 
      
      // Include all preferences
      budget_preference: budget,
      risk_preference: riskTolerance,
      disabilities: disabilities,
      food_allergies: foodPreferences,
    };
    
    // Remove undefined/empty optional fields from payload if necessary,
    // though the provided structure handles optional fields with '?'

    console.log("Sending update payload:", payload);

    try {
      // Call the API
      await apiUpdateAccount(payload);
      
      // On success: 
      setStatusMessage({ message: "Account settings updated successfully! 🎉", type: 'success' });
      // Reset the password field after a successful update for security
      setPassword(""); 

    } catch (error) {
      console.error("Update failed:", error);
      let errorMessage = "An unknown error occurred during update.";

      if (error instanceof Error) {
        errorMessage = error.message;
      }
      setStatusMessage({ message: `Update failed: ${errorMessage}`, type: 'error' });
    }
  };


  // --- Styling (kept from previous step) ---
  const sectionStyle: React.CSSProperties = { 
    marginTop: "20px", 
    border: "1px solid #ccc", 
    padding: "20px", 
    maxWidth: "550px", 
    marginBottom: "20px" 
  };
  const inputGroupStyle: React.CSSProperties = { marginBottom: "15px" };
  const labelStyle: React.CSSProperties = { display: "block", marginBottom: "5px", fontWeight: "bold" };
  const inputStyle: React.CSSProperties = { width: "100%", padding: "8px", boxSizing: "border-box" };
  const statusStyle: React.CSSProperties = {
    padding: '10px',
    borderRadius: '4px',
    marginBottom: '20px',
    color: 'white',
    fontWeight: 'bold'
  };

  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link> | <Link to="/home">Home</Link> |{" "}
        <Link to="/view">View</Link>
      </nav>

      <h1>Account Settings</h1>

      {/* Status Message Display */}
      {statusMessage && (
        <div style={{ 
          ...statusStyle, 
          backgroundColor: statusMessage.type === 'success' ? '#28a745' : '#dc3545',
          maxWidth: "550px"
        }}>
          {statusMessage.message}
        </div>
      )}

      {/* Main Update Form for All Settings */}
      <form onSubmit={handleUpdate}>

        {/* --- 1. Account Details Section --- */}
        <div style={sectionStyle}>
          <h2>👤 Login Details</h2>
          
          {/* Email/Username Field (ReadOnly) */}
          <div style={inputGroupStyle}>
            <label htmlFor="email" style={labelStyle}>Username (Email):</label>
            <input
              type="email"
              id="email"
              value={email}
              // Set to readOnly because changing the primary login ID (email) 
              // is often a separate, complex process.
              readOnly 
              style={{ ...inputStyle, backgroundColor: '#f4f4f4' }} 
            />
            <small style={{ color: '#666' }}>Your username is linked to your email and cannot be changed here.</small>
          </div>

          {/* Password Field */}
          <div style={inputGroupStyle}>
            <label htmlFor="password" style={labelStyle}>New Password (Leave blank to keep current):</label>
            <input
              type="password"
              id="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              style={inputStyle}
              placeholder="Enter new password"
            />
            <small style={{ color: '#666' }}>Only enter a password if you wish to change it.</small>
          </div>
        </div>

        {/* --- 2. User Preferences Section --- */}
        <div style={sectionStyle}>
          <h2>⚙️ Activity Preferences</h2>
          
          {/* Budget Dropdown */}
          <div style={inputGroupStyle}>
            <label htmlFor="budget" style={labelStyle}>Budget:</label>
            <select
              id="budget"
              value={budget}
              onChange={(e) => setBudget(e.target.value as BudgetOption)}
              style={inputStyle}
            >
              {budgetOptions.map((option) => (
                <option key={option} value={option}>{option}</option>
              ))}
            </select>
          </div>

          {/* Risk Tolerance Dropdown */}
          <div style={inputGroupStyle}>
            <label htmlFor="riskTolerance" style={labelStyle}>Risk Tolerance:</label>
            <select
              id="riskTolerance"
              value={riskTolerance}
              onChange={(e) => setRiskTolerance(e.target.value as RiskOption)}
              style={inputStyle}
            >
              {riskOptions.map((option) => (
                <option key={option} value={option}>{option}</option>
              ))}
            </select>
          </div>

          {/* Disabilities Text Area */}
          <div style={inputGroupStyle}>
            <label htmlFor="disabilities" style={labelStyle}>Disabilities/Accessibility Needs:</label>
            <textarea
              id="disabilities"
              value={disabilities}
              onChange={(e) => setDisabilities(e.target.value)}
              style={{ ...inputStyle, minHeight: '80px' }}
              placeholder="e.g., Wheelchair user, needs assistance with stairs, visual impairment."
            />
          </div>

          {/* Food Preferences/Allergies Text Area */}
          <div style={inputGroupStyle}>
            <label htmlFor="foodPreferences" style={labelStyle}>Food Preferences/Allergies:</label>
            <textarea
              id="foodPreferences"
              value={foodPreferences}
              onChange={(e) => setFoodPreferences(e.target.value)}
              style={{ ...inputStyle, minHeight: '80px' }}
              placeholder="e.g., Gluten-free, no shellfish, vegan, prefers Italian cuisine."
            />
          </div>

        </div>

        {/* Global Update Button for the whole form */}
        <button 
          type="submit" 
          style={{ padding: "12px 20px", backgroundColor: "#007bff", color: "white", border: "none", cursor: "pointer", fontWeight: "bold" }}
        >
          Update All Settings
        </button>

      </form>
      
      <hr style={{ margin: "40px 0" }} />

      {/* Logout Section */}
      <div style={{ maxWidth: "550px" }}>
        <h2>🚪 Session Management</h2>
        <button onClick={onLogout} style={{ padding: "10px 15px", backgroundColor: "#dc3545", color: "white", border: "none", cursor: "pointer" }}>
          Logout
        </button>
      </div>
    </div>
  );
}