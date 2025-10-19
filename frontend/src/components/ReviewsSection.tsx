export default function ReviewsSection() {
  return (
    <section className="section reviews-section" id="reviews">
      <div className="section-inner">
        <h2 className="section-title reveal">Loved by Early Travelers</h2>
        <p className="section-subtitle reveal delay-1">
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
  );
}


