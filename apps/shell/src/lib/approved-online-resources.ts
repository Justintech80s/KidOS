export type KidOSApprovedResource =
  | 'kiddle'
  | 'khan-kids'
  | 'pbs-kids'
  | 'youtube-kids'
  | 'family-safety'
  | 'khanmigo';

export const APPROVED_RESOURCE_LABELS: Record<KidOSApprovedResource, string> = {
  kiddle: 'Kiddle',
  'khan-kids': 'Khan Academy Kids',
  'pbs-kids': 'PBS KIDS',
  'youtube-kids': 'YouTube Kids',
  'family-safety': 'Microsoft Family Safety',
  khanmigo: 'Khanmigo',
};

const FALLBACK_URLS: Record<KidOSApprovedResource, string> = {
  kiddle: 'https://www.kiddle.co/',
  'khan-kids': 'https://www.khanacademy.org/kids',
  'pbs-kids': 'https://pbskids.org/games/',
  'youtube-kids': 'https://kids.youtube.com/',
  'family-safety': 'https://account.microsoft.com/family',
  khanmigo: 'https://www.khanacademy.org/khan-labs',
};

type DesktopBridge = {
  openApprovedResource?(resource: KidOSApprovedResource): Promise<boolean>;
};

export async function openApprovedResource(resource: KidOSApprovedResource): Promise<boolean> {
  const desktop = (window as Window & { kidosDesktop?: DesktopBridge }).kidosDesktop;
  if (desktop?.openApprovedResource) {
    return desktop.openApprovedResource(resource);
  }

  const opened = window.open(FALLBACK_URLS[resource], '_blank', 'noopener,noreferrer');
  return Boolean(opened);
}
