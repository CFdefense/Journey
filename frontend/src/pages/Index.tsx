import { useEffect } from "react";
// Link no longer used directly in this page after refactor
import HeroSection from "../components/HeroSection";
import AboutSection from "../components/AboutSection";
import ValuesSection from "../components/ValuesSection";
import ReviewsSection from "../components/ReviewsSection";
import DemoSection from "../components/DemoSection";
import TeamSection from "../components/TeamSection";
import Navbar from "../components/Navbar";
import "../styles/Index.css";

export default function Index() {
  useEffect(() => {
    // Ensure all key elements participate in the reveal animation
    const ensureRevealOn = [
      ".value-card",
      ".review-card",
      ".team-card",
      ".demo-media",
      ".demo-text",
      ".about-text",
      ".section-title",
      ".section-subtitle"
    ];

    ensureRevealOn.forEach((selector) => {
      document.querySelectorAll<HTMLElement>(selector).forEach((el) => {
        if (!el.classList.contains("reveal")) el.classList.add("reveal");
      });
    });

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
      <HeroSection />

      <div className="index-sections">
        <AboutSection />

        <div className="separator" />

        <ValuesSection />

        <div className="separator" />

        <ReviewsSection />

        <div className="separator" />

        <DemoSection />

        <div className="separator" />

        <TeamSection />
      </div>
    </div>
  );
}