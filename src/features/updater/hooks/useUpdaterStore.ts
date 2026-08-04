import { create } from 'zustand';
import type { UpdateInfo, UpdaterState } from '../types/updater';

type UpdaterStatus = UpdaterState['status'];

interface UpdaterStore extends UpdaterState {
  setStatus: (status: UpdaterStatus) => void;
  setUpdateInfo: (info: UpdateInfo | null) => void;
  setProgress: (progress: number) => void;
  setError: (error: string | null) => void;
  dismiss: () => void;
  reset: () => void;
}

const DISMISSED_VERSION_KEY = 'academix:updater:dismissed-version';

export const useUpdaterStore = create<UpdaterStore>((set) => ({
  // Initial state
  status: 'idle',
  updateInfo: null,
  downloadProgress: 0,
  error: null,
  dismissedVersion: localStorage.getItem(DISMISSED_VERSION_KEY),

  // Actions
  setStatus: (status) => set({ status }),
  setUpdateInfo: (updateInfo) => set({ updateInfo }),
  setProgress: (downloadProgress) => set({ downloadProgress }),
  setError: (error) => set({ error, status: error ? 'error' : 'idle' }),
  dismiss: () =>
    set((state) => {
      const version = state.updateInfo?.version ?? null;
      if (version) {
        localStorage.setItem(DISMISSED_VERSION_KEY, version);
      }
      return {
        status: 'idle',
        updateInfo: null,
        downloadProgress: 0,
        error: null,
        dismissedVersion: version,
      };
    }),
  reset: () =>
    set({
      status: 'idle',
      updateInfo: null,
      downloadProgress: 0,
      error: null,
    }),
}));
