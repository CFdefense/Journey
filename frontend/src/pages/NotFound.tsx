import { useNavigate, Link } from "react-router-dom";
import "../styles/NotFound.css";

export default function NotFoundPage() {
  const navigate = useNavigate();

  return (
    <div className="lost-tourist-wrapper">
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>| <Link to="/home">Home</Link>|{" "}
        <Link to="/account">Account</Link>
      </nav>
      <div className="lost-tourist-card">
        <img
          src="./src/assets/car-on-map.jpg"
          alt="Lost tourist"
          className="tourist-image"
        />

        <h1 className="lost-tourist-title">
          Uh-oh… You’ve wandered off the map!
        </h1>

        <p className="lost-tourist-description">
          Looks like this page isn’t on our itinerary.
        </p>

        <div className="lost-tourist-buttons">
          <button className="btn-journey" onClick={() => navigate("/home")}>
            Start a New Journey
          </button>
        </div>
      </div>
    </div>
  );
}
