export type SafetySummaryData = {
  total: number;
  allowed: number;
  blocked: number;
  parentGated: number;
};

type SafetySummaryProps = {
  authorized: boolean;
  summary: SafetySummaryData;
  clearEvents: () => Promise<void>;
};

export default function SafetySummary({ authorized, summary, clearEvents }: SafetySummaryProps) {
  if (!authorized) return null;

  return (
    <section aria-label="Safety summary">
      <h2>Safety summary</h2>
      <p>{summary.total} safety decisions</p>
      <p>{summary.allowed} allowed</p>
      <p>{summary.blocked} blocked</p>
      <p>{summary.parentGated} needed parent approval</p>
      <p>KidOS stores minimal safety events, not a full browsing or search timeline.</p>
      <button type="button" onClick={() => void clearEvents()}>
        Clear safety events
      </button>
    </section>
  );
}
