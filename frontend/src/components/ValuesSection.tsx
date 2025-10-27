export default function ValuesSection() {
  return (
    <section className="section values-section" id="values">
      <div className="section-inner">
        <h2 className="section-title reveal">Our Values</h2>
        <p className="section-subtitle reveal delay-1">
          The principles that guide how we build and how we serve travelers.
        </p>
        <div className="values-grid">
          <div className="value-card reveal">
            <div className="value-icon">AI</div>
            <div className="value-title">Assistive Intelligence</div>
            <p className="value-text">
              We design agents that collaborate with you—transparent,
              controllable, and helpful at every step.
            </p>
          </div>
          <div className="value-card reveal delay-1">
            <div className="value-icon">TR</div>
            <div className="value-title">Trust & Privacy</div>
            <p className="value-text">
              Your data is yours. We prioritize security and clear consent over
              growth hacks.
            </p>
          </div>
          <div className="value-card reveal delay-2">
            <div className="value-icon">DL</div>
            <div className="value-title">Delight</div>
            <p className="value-text">
              Travel should feel magical. We sweat the details to make planning
              effortless.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
