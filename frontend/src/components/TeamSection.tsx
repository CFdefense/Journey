export default function TeamSection() {
  return (
    <section className="section team-section" id="team">
      <div className="section-inner">
        <h2 className="section-title reveal">Meet the Team</h2>
        <p className="section-subtitle reveal delay-1">The adventurers behind Journey.</p>
        <div className="team-grid">
          <div className="team-card reveal">
            <img className="team-avatar" src="/ellie.jpeg" alt="Gabrielle (Ellie) Knapp - Project Manager" />
            <div className="team-name">Gabrielle (Ellie) Knapp</div>
            <div className="team-role">Project Manager</div>
          </div>
          <div className="team-card reveal delay-1">
            <img className="team-avatar" src="/ethan.jpeg" alt="Ethan Morton - Software Engineer" />
            <div className="team-name">Ethan Morton</div>
            <div className="team-role">Software Engineer</div>
          </div>
          <div className="team-card reveal delay-2">
            <img className="team-avatar" src="/peter.jpeg" alt="Peter Arvanitis - Software Engineer" />
            <div className="team-name">Peter Arvanitis</div>
            <div className="team-role">Software Engineer</div>
          </div>
          <div className="team-card reveal delay-3">
            <img className="team-avatar" src="/nick.jpeg" alt="Nick Longo - Software Engineer" />
            <div className="team-name">Nick Longo</div>
            <div className="team-role">Software Engineer</div>
          </div>
          <div className="team-card reveal delay-4">
            <img className="team-avatar" src="/christian.jpeg" alt="Christian Farrell - Software Engineer" />
            <div className="team-name">Christian Farrell</div>
            <div className="team-role">Software Engineer</div>
          </div>
        </div>
      </div>
    </section>
  );
}


