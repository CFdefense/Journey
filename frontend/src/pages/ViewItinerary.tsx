import { Link } from "react-router-dom";
import Itinerary from "../components/Itinerary.tsx";
import "../styles/Itinerary.css";

export default function ViewItineraryPage() {
  return (
    <div className="view-page">
      {/* Navigation */}
      <nav>
        <Link to="/">Index</Link>| <Link to="/home">Home</Link>|{" "}
        <Link to="/account">Account</Link>
      </nav>
      <Itinerary />
      <button>Edit with AI</button>
    </div>
  );
}
