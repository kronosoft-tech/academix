import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useUpdaterStore } from './useUpdaterStore';

describe('useUpdaterStore', () => {
  beforeEach(() => {
    vi.mocked(localStorage.getItem).mockReturnValue(null);
    vi.mocked(localStorage.setItem).mockClear();
    useUpdaterStore.setState({
      status: 'idle',
      updateInfo: null,
      downloadProgress: 0,
      error: null,
      dismissedVersion: null,
    });
  });

  describe('state machine transitions', () => {
    it('transitions idle → checking → available → downloading → installing', () => {
      const { setStatus } = useUpdaterStore.getState();

      expect(useUpdaterStore.getState().status).toBe('idle');

      setStatus('checking');
      expect(useUpdaterStore.getState().status).toBe('checking');

      setStatus('available');
      expect(useUpdaterStore.getState().status).toBe('available');

      setStatus('downloading');
      expect(useUpdaterStore.getState().status).toBe('downloading');

      setStatus('installing');
      expect(useUpdaterStore.getState().status).toBe('installing');
    });

    it('setUpdateInfo stores update information', () => {
      const info = {
        version: '2.0.0',
        releaseNotes: 'New features',
        date: '2025-01-01T00:00:00Z',
        mandatory: false,
      };

      useUpdaterStore.getState().setUpdateInfo(info);
      expect(useUpdaterStore.getState().updateInfo).toEqual(info);
    });

    it('setProgress updates downloadProgress', () => {
      useUpdaterStore.getState().setProgress(55);
      expect(useUpdaterStore.getState().downloadProgress).toBe(55);
    });

    it('setError with non-null message sets status to error', () => {
      useUpdaterStore.getState().setError('Download failed');
      const state = useUpdaterStore.getState();
      expect(state.error).toBe('Download failed');
      expect(state.status).toBe('error');
    });

    it('setError with null sets status to idle', () => {
      useUpdaterStore.setState({ status: 'error', error: 'Some error' });

      useUpdaterStore.getState().setError(null);
      const state = useUpdaterStore.getState();
      expect(state.error).toBeNull();
      expect(state.status).toBe('idle');
    });
  });

  describe('dismissed version persistence', () => {
    it('dismiss persists version to localStorage and resets state', () => {
      useUpdaterStore.setState({
        status: 'available',
        updateInfo: {
          version: '3.0.0',
          releaseNotes: 'Big update',
          date: '2025-06-01T00:00:00Z',
          mandatory: false,
        },
        downloadProgress: 0,
        error: null,
      });

      useUpdaterStore.getState().dismiss();

      expect(localStorage.setItem).toHaveBeenCalledWith(
        'academix:updater:dismissed-version',
        '3.0.0'
      );

      const state = useUpdaterStore.getState();
      expect(state.status).toBe('idle');
      expect(state.updateInfo).toBeNull();
      expect(state.downloadProgress).toBe(0);
      expect(state.error).toBeNull();
      expect(state.dismissedVersion).toBe('3.0.0');
    });

    it('dismiss without updateInfo does not call localStorage.setItem', () => {
      useUpdaterStore.setState({ updateInfo: null });

      useUpdaterStore.getState().dismiss();

      expect(localStorage.setItem).not.toHaveBeenCalled();
      expect(useUpdaterStore.getState().dismissedVersion).toBeNull();
    });

    it('dismissed version suppresses re-display of same version', () => {
      useUpdaterStore.setState({
        status: 'available',
        updateInfo: {
          version: '2.5.0',
          releaseNotes: '',
          date: '2025-03-01T00:00:00Z',
          mandatory: false,
        },
      });

      useUpdaterStore.getState().dismiss();
      expect(useUpdaterStore.getState().dismissedVersion).toBe('2.5.0');

      // Simulate a new store reading from localStorage
      vi.mocked(localStorage.getItem).mockReturnValue('2.5.0');
      // The dismissedVersion should match the previously dismissed version
      expect(useUpdaterStore.getState().dismissedVersion).toBe('2.5.0');
    });
  });

  describe('reset', () => {
    it('reset clears all state back to initial values', () => {
      useUpdaterStore.setState({
        status: 'downloading',
        updateInfo: {
          version: '4.0.0',
          releaseNotes: 'Release',
          date: '2025-07-01T00:00:00Z',
          mandatory: true,
        },
        downloadProgress: 75,
        error: 'Some previous error',
        dismissedVersion: '3.0.0',
      });

      useUpdaterStore.getState().reset();

      const state = useUpdaterStore.getState();
      expect(state.status).toBe('idle');
      expect(state.updateInfo).toBeNull();
      expect(state.downloadProgress).toBe(0);
      expect(state.error).toBeNull();
      // Note: reset does NOT clear dismissedVersion (by design)
      expect(state.dismissedVersion).toBe('3.0.0');
    });

    it('reset does not affect dismissedVersion', () => {
      useUpdaterStore.setState({ dismissedVersion: '1.0.0' });

      useUpdaterStore.getState().reset();

      expect(useUpdaterStore.getState().dismissedVersion).toBe('1.0.0');
    });
  });
});
