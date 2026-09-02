import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import App from '../../apps/shell/src/App';
import type { KidOSApi } from '../../apps/shell/src/lib/kidos-api';

afterEach(cleanup);

describe('KidOS restricted safe mode', () => {
  it('does not expose creation or protected web when Guardian reports restricted safe mode', async () => {
    const api: KidOSApi = {
      async planWorkspace() {
        throw new Error('creation must remain unavailable');
      },
      async evaluateNavigation() {
        throw new Error('navigation must remain unavailable');
      },
      async evaluateDownload() {
        throw new Error('downloads must remain unavailable');
      },
      async guardianStatus() {
        return 'restricted_safe_mode';
      },
    };

    render(<App api={api} />);

    expect(await screen.findByText('Restricted safe mode')).toBeTruthy();
    expect(screen.queryByLabelText('Ask KidOS')).toBeNull();
    expect(screen.queryByLabelText('Protected web address')).toBeNull();
  });
});
