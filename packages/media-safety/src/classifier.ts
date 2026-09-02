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

export async function classifyMedia(
  input: MediaInput,
  options: ClassifyMediaOptions,
): Promise<MediaClassification> {
  const threshold = options.localConfidenceThreshold ?? 0.8;
  const localRaw = await options.localClassifier(input);
  const localParsed = MediaClassificationSchema.safeParse(localRaw);

  if (!localParsed.success || localParsed.data.source !== 'local') {
    return {
      category: 'uncertain',
      confidence: 0,
      source: 'local',
      risk: 'high',
    };
  }

  const local = localParsed.data;
  const needsEscalation =
    local.category === 'uncertain' ||
    local.risk === 'high' ||
    local.confidence < threshold;

  if (!needsEscalation || !options.remoteEnabled || !options.remoteClassifier) {
    return local;
  }

  try {
    const remoteRaw = await options.remoteClassifier(input);
    const remoteParsed = MediaClassificationSchema.safeParse(remoteRaw);

    if (!remoteParsed.success || remoteParsed.data.source !== 'remote') {
      return local;
    }

    return remoteParsed.data;
  } catch {
    return local;
  }
}
