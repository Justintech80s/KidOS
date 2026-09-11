import type { KidOSDestination } from './KidOSSidebar';
import type { KidOSSystemStatus } from './system-status';
import KidOSGreeting from './KidOSGreeting';
import KidOSHomeGrid from './KidOSHomeGrid';
import KidOSSafetyStatus from './KidOSSafetyStatus';

export interface KidOSHomeScreenProps {
  status: KidOSSystemStatus;
  onNavigate(destination: KidOSDestination): void;
}

export default function KidOSHomeScreen({ status, onNavigate }: KidOSHomeScreenProps) {
  return (
    <section className="kidos-screen kidos-home-screen" data-testid="kidos-home-screen" aria-label="KidOS Home">
      <KidOSGreeting />
      <KidOSHomeGrid onNavigate={onNavigate} />
      <KidOSSafetyStatus status={status} />
    </section>
  );
}
