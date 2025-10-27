import "../styles/FinishAccountPopup.css";
import { useState } from "react";
import { Link } from "react-router-dom";

export function FinishAccountPopup() {
  const [visible, setVisible] = useState(true);

  if (!visible) return null; // Hide the button when closed

  return (
    <div className="finish-popup-container banner-style">
      <div className="finish-account-wrapper">
        <span className="banner-message">
          Complete your profile to get personalized travel recommendations
        </span>
        <Link to="/account">
          <button className="finish-account-setup">Complete Profile</button>
        </Link>
        <span className="close-btn" onClick={() => setVisible(false)}>
          ×
        </span>
      </div>
    </div>
  );
}