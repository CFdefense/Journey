import { useNavigate } from "react-router-dom";
import "../styles/NotFound.css";

export default function NotFoundPage() {
  const navigate = useNavigate();

  return (
    <div className="not-found-wrapper">
      <div className="not-found-content">
        <h1 className="not-found-number">404</h1>
        <p className="not-found-message">
          This page does not exist, or no longer exists
        </p>
        <button 
          className="not-found-link" 
          onClick={() => navigate("/home")}
        >
          Return to Home
        </button>
      </div>
      <div className="not-found-character">
        <img 
          src="/404.png" 
          alt="404 character" 
          className="character-image"
        />
      </div>
    </div>
  );
}
