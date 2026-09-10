export default function KidOSGreeting({ displayName = 'Alex' }: { displayName?: string }) {
  return <section className="kidos-greeting"><p className="eyebrow">Protected profile</p><h1>Hi, {displayName}! 👋</h1><p>What would you like to explore today?</p></section>;
}
