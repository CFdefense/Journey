import { Link } from "react-router-dom";

export default function HeroSection() {
  return (
    <div className="index-content reveal">
      <h1 className="hero-title reveal">Journey</h1>
      <p className="hero-tagline reveal delay-1">
        Let our intelligent AI agents plan your next adventure
      </p>
      <Link to="/signup" className="cta-button reveal delay-2">
        Start Your Journey <span className="arrow">→</span>
      </Link>
      <div className="earth-container">
        <img src="/earth.png" alt="Earth" className="earth" />
        <div className="plane-orbit">
          <img src="/plane.jpg" alt="Plane" className="plane" />
        </div>
      </div>
    </div>
  );
}
