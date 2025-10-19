import { useEffect } from "react";
// Link no longer used directly in this page after refactor
import HeroSection from "../components/HeroSection";
import AboutSection from "../components/AboutSection";
import ValuesSection from "../components/ValuesSection";
import ReviewsSection from "../components/ReviewsSection";
import DemoSection from "../components/DemoSection";
import TeamSection from "../components/TeamSection";
import Navbar from "../components/Navbar";
import Footer from "../components/Footer";
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

  // Theme gradient based on scroll position
  useEffect(() => {
    const colorStops = [
      "#000000", // black
      "#006BBB", // blue
      "#FA8072", // salmon / coral
      "#FFC872", // light orange / gold
    ];

    let ticking = false;
    const updateTheme = () => {
      const doc = document.documentElement;
      const maxScroll = doc.scrollHeight - window.innerHeight;
      const progress = maxScroll > 0 ? Math.min(1, Math.max(0, window.scrollY / maxScroll)) : 0;

      const segments = colorStops.length - 1;
      const position = progress * segments;
      const i = Math.min(segments - 1, Math.floor(position));
      const t = position - i;

      const root = document.querySelector<HTMLElement>(".index-page");
      if (root) {
        root.style.setProperty('--from', colorStops[i]);
        root.style.setProperty('--to', colorStops[i + 1]);
        root.style.setProperty('--t', `${t * 100}%`);
      }
    };

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          updateTheme();
          ticking = false;
        });
        ticking = true;
      }
    };

    updateTheme();
    window.addEventListener('scroll', onScroll, { passive: true });
    window.addEventListener('resize', onScroll);
    return () => {
      window.removeEventListener('scroll', onScroll);
      window.removeEventListener('resize', onScroll);
    };
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
      <Footer />
    </div>
  );
}