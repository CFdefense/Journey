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
            <div className="demo-media-placeholder">Product Demo</div>
          </div>
          <div className="demo-text reveal delay-1">
            <p>
              Start with a vibe—"foodie trip to Lisbon"—and instantly see a
              curated plan with budget options, transit tips, and hidden gems.
              Tweak anything, and watch the itinerary adapt in real time.
            </p>
            <Link to="/signup" className="demo-cta">
              Try the Demo
            </Link>
          </div>
        </div>
      </div>
    </section>
  );
}
