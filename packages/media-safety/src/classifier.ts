import {
  MediaClassificationSchema,
  type MediaClassification,
} from '@kidos/contracts';

export type MediaInput = {
  kind: 'image' | 'video_frame';
  bytes: Uint8Array;
  context?: {
    title?: string;
    domain?: string;
    accountId?: string;
  };
};

export type MediaClassifier = (input: MediaInput) => Promise<unknown>;

export type ClassifyMediaOptions = {
  remoteEnabled: boolean;
  localClassifier: MediaClassifier;
  remoteClassifier?: MediaClassifier;
  localConfidenceThreshold?: number;
};

const failClosedLocal: MediaClassification = {
  category: 'uncertain',
  confidence: 0,
  source: 'local',
  risk: 'high',
};

async function tryRemote(
  input: MediaInput,
  options: ClassifyMediaOptions,
  fallback: MediaClassification,
): Promise<MediaClassification> {
  if (!options.remoteEnabled || !options.remoteClassifier) {
    return fallback;
  }

  try {
    const remoteRaw = await options.remoteClassifier(input);
    const remoteParsed = MediaClassificationSchema.safeParse(remoteRaw);

    if (!remoteParsed.success || remoteParsed.data.source !== 'remote') {
      return fallback;
    }

    return remoteParsed.data;
  } catch {
    return fallback;
  }
}

export async function classifyMedia(
  input: MediaInput,
  options: ClassifyMediaOptions,
): Promise<MediaClassification> {
  const threshold = options.localConfidenceThreshold ?? 0.8;

  let localRaw: unknown;
  try {
    localRaw = await options.localClassifier(input);
  } catch {
    return tryRemote(input, options, failClosedLocal);
  }

  const localParsed = MediaClassificationSchema.safeParse(localRaw);

  if (!localParsed.success || localParsed.data.source !== 'local') {
    return tryRemote(input, options, failClosedLocal);
  }

  const local = localParsed.data;
  const needsEscalation =
    local.category === 'uncertain' ||
    local.risk === 'high' ||
    local.confidence < threshold;

  if (!needsEscalation) {
    return local;
  }

  return tryRemote(input, options, local);
}
