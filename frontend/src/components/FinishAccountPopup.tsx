import "../styles/FinishAccountPopup.css";
import { useState } from "react";
import { Link } from "react-router-dom";


export function FinishAccountPopup() {
  const [visible, setVisible] = useState(true);

  if (!visible) return null; // Hide the button when closed

  return (
    <div className="finish-popup-container">
      <div className="finish-account-wrapper">
        <Link to="/account">
          <button className="finish-account-setup">Finish Account Setup</button>
        </Link>
        <span className="close-btn" onClick={() => setVisible(false)}>
          ×
        </span>
      </div>
    </div>
  );
}
