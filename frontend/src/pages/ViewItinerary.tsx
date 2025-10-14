import Itinerary from '../components/Itinerary.tsx';
import "../styles/Itinerary.css";

export default function ViewItineraryPage() {
  return (
    <div className="view-page">
      <Itinerary />
      <button>Edit with AI</button>
    </div>
  );
}