import { Link } from "react-router-dom";

export default function DemoSection() {
  return (
    <section className="section demo-section" id="demo">
      <div className="section-inner">
        <h2 className="section-title reveal">See It In Action</h2>
        <p className="section-subtitle reveal delay-1">
          A two-minute look at how Journey plans a weekend getaway.
        </p>
        <div className="demo-container">
          <div className="demo-media reveal">
            <img
              src="https://placehold.co/1200x675/30A0E0/FFFFFF?text=Product+Demo"
              alt="Product demo placeholder"
            />
          </div>
          <div className="demo-text reveal delay-1">
            <p>
              Start with a vibe—"foodie trip to Lisbon"—and instantly see a curated
              plan with budget options, transit tips, and hidden gems. Tweak anything,
              and watch the itinerary adapt in real time.
            </p>
            <Link to="/signup" className="demo-cta">Try the Demo</Link>
          </div>
        </div>
      </div>
    </section>
  );
}


