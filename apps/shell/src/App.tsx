export default function App() {
  return (
    <main className="kidos-shell">
      <nav className="topbar" aria-label="KidOS status">
        <strong>KidOS</strong>
        <span className="safety-pill">Protected</span>
      </nav>

      <section className="hero">
        <p className="eyebrow">Your creative computer</p>
        <h1>What do you want to create?</h1>
        <p className="intro">
          Tell KidOS what you want to make and it will build a safe workspace for you.
        </p>
        <form className="creation-box" onSubmit={(event) => event.preventDefault()}>
          <label htmlFor="creation-request">Ask KidOS</label>
          <div className="creation-row">
            <input
              id="creation-request"
              name="creation-request"
              placeholder="Make a story about a robot exploring Mars…"
              autoComplete="off"
            />
            <button type="submit">Create</button>
          </div>
        </form>
      </section>

      <section className="starter-grid" aria-label="Starter workspaces">
        <article><span>✍️</span><h2>Story</h2><p>Write, plan, and bring ideas to life.</p></article>
        <article><span>🎨</span><h2>Draw & Present</h2><p>Create pictures, posters, and presentations.</p></article>
        <article><span>💻</span><h2>Beginner Coding</h2><p>Build simple projects while learning how code works.</p></article>
      </section>
    </main>
  );
}
