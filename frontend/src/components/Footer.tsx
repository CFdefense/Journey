import { Link } from "react-router-dom";
import "../styles/Footer.css";

export default function Footer() {
  return (
    <footer className="footer">
      <div className="footer-inner">
        <div className="footer-left">
          <div className="footer-summary">
            <p className="footer-summary-text">
              Journey is your AI travel copilot—plan and coordinate trips in
              minutes. Available worldwide online.
            </p>
          </div>
          <div className="footer-mission">
            <p className="footer-mission-text">
              Our mission is to turn ideas into clear, budget-aware itineraries.
            </p>
          </div>
        </div>

        <div className="footer-center">
          <div
            className="footer-logo-placeholder"
            aria-label="Journey logo placeholder"
          />
          <div className="footer-copyright">
            <div className="footer-company">Journey</div>
            <div className="footer-legal">©2025 · All rights reserved</div>
            <div className="footer-links-inline">
              <Link to="/terms" className="footer-inline-link">
                Terms
              </Link>
              <span className="bullet">·</span>
              <Link to="/privacy" className="footer-inline-link">
                Privacy
              </Link>
            </div>
          </div>
        </div>

        <div className="footer-right">
          <div className="footer-newsletter">
            <form
              className="newsletter-form"
              onSubmit={(e) => {
                e.preventDefault();
                const form = e.currentTarget;
                const data = new FormData(form);
                const email = (data.get("email") || "").toString();
                if (!email) {
                  alert("Please enter your email.");
                  return;
                }

                // Placeholder submit action – integrate with backend later
                alert(`Thanks! We'll keep ${email} in the loop.`);
                form.reset();
              }}
            >
              <input
                type="email"
                name="email"
                className="newsletter-input"
                placeholder="Get updates"
                aria-label="Email address"
                required
              />
              <button
                type="submit"
                className="newsletter-submit"
                aria-label="Submit email"
              >
                <span className="newsletter-arrow" aria-hidden>
                  ›
                </span>
              </button>
            </form>

            <div className="footer-quick-links">
              <Link to="/about" className="footer-link">
                About
              </Link>
              <Link to="/demo" className="footer-link">
                Demo
              </Link>
              <Link to="/linkedin" className="footer-link">
                LinkedIn
              </Link>
              <Link to="/x" className="footer-link">
                X
              </Link>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}
