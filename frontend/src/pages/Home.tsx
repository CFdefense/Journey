import ChatWindow from "../components/ChatWindow";
import Itinerary from "../components/Itinerary";
import "../styles/Home.css"; // 👈 add a CSS file for layout

export default function CreateItinerary() {
  return (
    <div className="create-itinerary-page">
      <h1>Where do you plan to explore?</h1>
      <div className="itinerary-layout">
        <ChatWindow />
        <Itinerary />
      </div>
    </div>
  );
}
