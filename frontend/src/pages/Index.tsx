import { Link } from "react-router-dom";
import Navbar from "../components/Navbar";
import "../styles/Index.css";

export default function Index() {
  return (
    <div className="index-page">
      <Navbar page="index" />
      <div className="stars"></div>
      <div className="index-content">
        <h1 className="hero-title">Journey</h1>
        <p className="hero-tagline">
          Let our intelligent AI agents plan your next adventure
        </p>
        <Link to="/signup" className="cta-button">
          Start Your Journey
        </Link>
        <div className="earth-container">
          <img src="/earth.png" alt="Earth" className="earth" />
          <div className="plane-orbit">
            <img src="/plane.jpg" alt="Plane" className="plane" />
          </div>
        </div>
      </div>
    </div>
  );
}