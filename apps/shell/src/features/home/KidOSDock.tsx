import type { KidOSDestination } from './KidOSSidebar';

export default function KidOSDock({ onNavigate }: { onNavigate(destination: KidOSDestination): void }) {
  const items: Array<[KidOSDestination, string, string]> = [['browser','Browser','🌐'],['apps','Apps','▦'],['create','Create','✏️'],['ai','AI','🤖'],['wellbeing','Wellbeing','☀️']];
  return <footer className="kidos-dock" data-testid="kidos-dock" aria-label="KidOS dock">{items.map(([destination,label,icon]) => <button type="button" key={destination} onClick={() => onNavigate(destination)}><span>{icon}</span><small>{label}</small></button>)}</footer>;
}
