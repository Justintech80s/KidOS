import { type FormEvent, useState } from 'react';

export default function KidOSTopBar({ onSafeSearch }: { onSafeSearch(query: string): void }) {
  const [query, setQuery] = useState('');
  const date = new Intl.DateTimeFormat('en-US', { weekday: 'short', month: 'short', day: 'numeric' }).format(new Date());

  function submit(event: FormEvent) {
    event.preventDefault();
    const value = query.trim();
    if (value) onSafeSearch(value);
  }

  return (
    <header className="kidos-topbar">
      <form className="kidos-global-search" onSubmit={submit}>
        <span aria-hidden="true">⌕</span>
        <input aria-label="Search KidOS safely" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search the safe web..." />
        <button type="submit">Search</button>
      </form>
      <div className="kidos-topbar-meta">
        <span className="kidos-topbar-protected">🛡 Protected</span>
        <div className="kidos-clock" aria-label="Date">{date}</div>
        <div className="kidos-topbar-avatar" aria-label="Profile: Alex">🧑🏽</div>
      </div>
    </header>
  );
}
