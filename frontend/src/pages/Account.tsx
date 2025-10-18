import { Link } from "react-router-dom";
import { apiLogout } from "../api/account";
import { useContext, type Context, useState } from "react";
import { GlobalContext } from "../helpers/global";
import type { GlobalState } from "../components/GlobalProvider";

type BudgetOption = 'VeryLowBudget' | 'LowBudget' | 'MediumBudget' | 'HighBudget' | 'LuxuryBudget';
type RiskOption = 'ChillVibes' | 'LightFun' | 'Adventurer' | 'RiskTaker';

export default function Account() {
  const { setAuthorized } = useContext<GlobalState>(GlobalContext as Context<GlobalState>);

  const [username, setUsername] = useState("currentUsername");
  const [password, setPassword] = useState("********");
  const [budget, setBudget] = useState<BudgetOption>("MediumBudget");
  const [riskTolerance, setRiskTolerance] = useState<RiskOption>("LightFun");
  const [disabilities, setDisabilities] = useState("None specified.");
  const [foodPreferences, setFoodPreferences] = useState("Vegetarian, no peanuts.");

  const onLogout = async () => {
    console.log("Logging out");
    try {
      await apiLogout();
    } catch (e) {
      console.error("Logout error:", e);
    } finally {
      window.location.href = "/"; //workaround since navigate doesn't work properly
      setAuthorized(false);
    }
  };

  // Placeholder function for handling the form submission
  const handleUpdate = (e: React.FormEvent) => {
    e.preventDefault();
    console.log("Attempting to update account details and preferences...");
    // The logic to connect to the backend API will go here later
    
    // Log current state to show values:
    console.log({
      newUsername: username,
      newBudget: budget,
      newRiskTolerance: riskTolerance,
      newDisabilities: disabilities,
      newFoodPreferences: foodPreferences
    });
  };

  // Options for dropdowns
  const budgetOptions: BudgetOption[] = ['VeryLowBudget', 'LowBudget', 'MediumBudget', 'HighBudget', 'LuxuryBudget'];
  const riskOptions: RiskOption[] = ['ChillVibes', 'LightFun', 'Adventurer', 'RiskTaker'];


  // Basic styling for sections (you'll likely replace this with a CSS framework)
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

  return (
    <div>
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link> | <Link to="/home">Home</Link> |{" "}
        <Link to="/view">View</Link>
      </nav>

<h1>Account Settings</h1>

      {/* Main Update Form for All Settings */}
      <form onSubmit={handleUpdate}>

        {/* --- 1. Account Details Section --- */}
        <div style={sectionStyle}>
          <h2>Login Details</h2>
          
          {/* Username Field */}
          <div style={inputGroupStyle}>
            <label htmlFor="username" style={labelStyle}>Username:</label>
            <input
              type="text"
              id="username"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              style={inputStyle}
              required
            />
          </div>

          {/* Password Field */}
          <div style={inputGroupStyle}>
            <label htmlFor="password" style={labelStyle}>Password:</label>
            <input
              type="password"
              id="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              style={inputStyle}
              required
            />
            <small style={{ color: '#666' }}>Note: Password field shows masked characters for security.</small>
          </div>
        </div>

        {/* --- 2. User Preferences Section --- */}
        <div style={sectionStyle}>
          <h2>Activity Preferences</h2>
          
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
        <button onClick={onLogout} style={{ padding: "10px 15px", backgroundColor: "#dc3545", color: "white", border: "none", cursor: "pointer" }}>
          Logout
        </button>
      </div>
    </div>
  );
}
