import { useEffect } from "react";
import { Link } from "react-router-dom";
import Navbar from "../components/Navbar";
import "../styles/Index.css";

export default function Index() {
  useEffect(() => {
    const elements = Array.from(document.querySelectorAll<HTMLElement>(".reveal"));
    if (elements.length === 0) return;

    const observer = new IntersectionObserver(
      entries => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.classList.add("in-view");
            observer.unobserve(entry.target);
          }
        });
      },
      { root: null, rootMargin: "0px", threshold: 0.15 }
    );

    elements.forEach(el => observer.observe(el));
    return () => observer.disconnect();
  }, []);
  return (
    <div className="index-page">
      <Navbar page="index" />
      <div className="stars"></div>
      <div className="index-content reveal">
        <h1 className="hero-title reveal">Journey</h1>
        <p className="hero-tagline reveal delay-1">
          Let our intelligent AI agents plan your next adventure
        </p>
        <Link to="/signup" className="cta-button reveal delay-2">
          Start Your Journey
        </Link>
        <div className="earth-container">
          <img src="/earth.png" alt="Earth" className="earth" />
          <div className="plane-orbit">
            <img src="/plane.jpg" alt="Plane" className="plane" />
          </div>
        </div>
      </div>

      <div className="index-sections">
        {/* About Us */}
        <section className="section about-section" id="about">
          <div className="section-inner">
            <h2 className="section-title">About Us</h2>
            <p className="section-subtitle">
              We are building delightful, trustworthy AI trip planners that turn
              ideas into itineraries in minutes. Our mission is to remove the
              friction from travel planning so you can focus on experiencing the world.
            </p>
            <div className="about-content">
              <div className="about-text reveal">
                <p>
                  Welcome to Journey, where people-powered reviews and insights fuel an AI-engined
                  travel tool designed to put together your dream vacation. Give us all your specifics.
                  Flying to Tokyo with your twin toddlers and wife with a gluten allergy? That’s our thing.
                  Road trip to Maine with your brother who hates highways and can’t smell lobster without puking?
                  That’s our thing. Only in Peru for five days and you want to hike Macchu Pichu but also see the
                  Atacama while staying on a tight budget? That’s. Our. Thing.
                </p>
              </div>
            </div>
          </div>
        </section>

        <div className="separator" />

        {/* Company Values */}
        <section className="section values-section" id="values">
          <div className="section-inner">
            <h2 className="section-title">Our Values</h2>
            <p className="section-subtitle">
              The principles that guide how we build and how we serve travelers.
            </p>
            <div className="values-grid">
              <div className="value-card reveal">
                <div className="value-icon">AI</div>
                <div className="value-title">Assistive Intelligence</div>
                <p className="value-text">
                  We design agents that collaborate with you—transparent, controllable,
                  and helpful at every step.
                </p>
              </div>
              <div className="value-card reveal delay-1">
                <div className="value-icon">TR</div>
                <div className="value-title">Trust & Privacy</div>
                <p className="value-text">
                  Your data is yours. We prioritize security and clear consent over growth hacks.
                </p>
              </div>
              <div className="value-card reveal delay-2">
                <div className="value-icon">DL</div>
                <div className="value-title">Delight</div>
                <p className="value-text">
                  Travel should feel magical. We sweat the details to make planning effortless.
                </p>
              </div>
            </div>
          </div>
        </section>

        <div className="separator" />

        {/* Reviews */}
        <section className="section reviews-section" id="reviews">
          <div className="section-inner">
            <h2 className="section-title">Loved by Early Travelers</h2>
            <p className="section-subtitle">
              Real feedback from people who planned their trips with Journey.
            </p>
            <div className="reviews-grid">
              <div className="review-card reveal">
                <div className="review-stars">★★★★★</div>
                <div className="review-text">
                  "Planned my Tokyo week in an hour. The suggestions were spot on and
                  I felt in control the whole time."
                </div>
                <div className="review-author">— Casey, Japan</div>
              </div>
              <div className="review-card reveal delay-1">
                <div className="review-stars">★★★★★</div>
                <div className="review-text">
                  "Way less back-and-forth. The itinerary adapted to our kids' needs
                  without me juggling a thousand tabs."
                </div>
                <div className="review-author">— Priya, Portugal</div>
              </div>
              <div className="review-card reveal delay-2">
                <div className="review-stars">★★★★★</div>
                <div className="review-text">
                  "Loved the transparency. I knew where recommendations came from and
                  could swap things easily."
                </div>
                <div className="review-author">— Luis, Mexico</div>
              </div>
            </div>
          </div>
        </section>

        <div className="separator" />

        {/* Demo */}
        <section className="section demo-section" id="demo">
          <div className="section-inner">
            <h2 className="section-title">See It In Action</h2>
            <p className="section-subtitle">
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

        <div className="separator" />

        {/* Meet the Team */}
        <section className="section team-section" id="team">
          <div className="section-inner">
            <h2 className="section-title">Meet the Team</h2>
            <p className="section-subtitle">
              The adventurers behind Journey.
            </p>
            <div className="team-grid">
              <div className="team-card reveal">
                <img
                  className="team-avatar"
                  src="/ellie.jpeg"
                  alt="Gabrielle (Ellie) Knapp - Project Manager"
                />
                <div className="team-name">Gabrielle (Ellie) Knapp</div>
                <div className="team-role">Project Manager</div>
              </div>
              <div className="team-card reveal delay-1">
                <img
                  className="team-avatar"
                  src="/ethan.jpeg"
                  alt="Ethan Morton - Software Engineer"
                />
                <div className="team-name">Ethan Morton</div>
                <div className="team-role">Software Engineer</div>
              </div>
              <div className="team-card reveal delay-2">
                <img
                  className="team-avatar"
                  src="/peter.jpeg"
                  alt="Peter Arvanitis - Software Engineer"
                />
                <div className="team-name">Peter Arvanitis</div>
                <div className="team-role">Software Engineer</div>
              </div>
              <div className="team-card reveal delay-3">
                <img
                  className="team-avatar"
                  src="/nick.jpeg"
                  alt="Nick Longo - Software Engineer"
                />
                <div className="team-name">Nick Longo</div>
                <div className="team-role">Software Engineer</div>
              </div>
              <div className="team-card reveal delay-4">
                <img
                  className="team-avatar"
                  src="/christian.jpeg"
                  alt="Christian Farrell - Software Engineer"
                />
                <div className="team-name">Christian Farrell</div>
                <div className="team-role">Software Engineer</div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}