import { describe, expect, it } from 'vitest';
import { tauriKidOSApi } from './kidos-api';

describe('KidOS lockdown API', () => {
  it('exposes only named lockdown operations', () => {
    expect(tauriKidOSApi.lockdownStatus).toBeTypeOf('function');
    expect(tauriKidOSApi.configureWindowsLockdown).toBeTypeOf('function');
    expect(tauriKidOSApi.requestParentMaintenanceUnlock).toBeTypeOf('function');
    expect(tauriKidOSApi.removeWindowsLockdown).toBeTypeOf('function');
    expect('executeCommand' in tauriKidOSApi).toBe(false);
    expect('runShell' in tauriKidOSApi).toBe(false);
  });
});
