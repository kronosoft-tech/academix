import { useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { useUpdaterStore } from './useUpdaterStore';
import type { UpdateInfo } from '../types/updater';

const DOWNLOAD_TIMEOUT_MS = 300_000; // 300 seconds
const MAX_RETRIES = 3;
const BACKOFF_BASE_MS = 1000;

interface UpdateAvailablePayload {
  version: string;
  release_notes: string;
  date: string;
  mandatory: boolean;
}

export function useUpdater() {
  const store = useUpdaterStore();
  const retryCount = useRef(0);

  // Subscribe to backend update-available event
  useEffect(() => {
    const unlisten = listen<UpdateAvailablePayload>(
      'update-available',
      (event) => {
        const payload = event.payload;
        const { dismissedVersion, setUpdateInfo, setStatus } =
          useUpdaterStore.getState();

        // Map snake_case Rust payload to camelCase TypeScript interface
        const info: UpdateInfo = {
          version: payload.version,
          releaseNotes: payload.release_notes,
          date: payload.date,
          mandatory: payload.mandatory,
        };

        // Suppress if user previously dismissed this version
        if (dismissedVersion === info.version) return;

        setUpdateInfo(info);
        setStatus('available');
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const checkForUpdate = useCallback(async () => {
    const { setStatus, setUpdateInfo, setError, dismissedVersion } =
      useUpdaterStore.getState();
    setStatus('checking');

    try {
      const result = await invoke<UpdateInfo | null>('check_for_update');
      if (result && result.version !== dismissedVersion) {
        setUpdateInfo(result);
        setStatus('available');
      } else {
        setStatus('idle');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  const startDownload = useCallback(async () => {
    const { setStatus, setProgress, setError } = useUpdaterStore.getState();
    retryCount.current = 0;

    const attemptDownload = async (): Promise<void> => {
      setStatus('downloading');
      setProgress(0);

      try {
        const update = await check();
        if (!update) {
          setError('No update available');
          return;
        }

        let lastProgressUpdate = Date.now();
        const timeoutId = setTimeout(() => {
          setError('Download timed out after 300 seconds');
        }, DOWNLOAD_TIMEOUT_MS);

        await update.downloadAndInstall((event) => {
          if (event.event === 'Started') {
            setProgress(0);
          } else if (event.event === 'Progress') {
            const now = Date.now();
            if (now - lastProgressUpdate >= 2000 || event.data.chunkLength === 0) {
              // Approximate progress based on chunks received
              setProgress(
                Math.min(99, useUpdaterStore.getState().downloadProgress + 1)
              );
              lastProgressUpdate = now;
            }
          } else if (event.event === 'Finished') {
            setProgress(100);
          }
        });

        clearTimeout(timeoutId);
        setStatus('installing');

        // Auto-restart after successful install
        await relaunch();
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e);

        // Signature failures are non-retryable
        if (errorMsg.toLowerCase().includes('signature')) {
          setError(`Signature verification failed: ${errorMsg}`);
          return;
        }

        // Retry with exponential backoff (1s, 2s, 4s)
        retryCount.current += 1;
        if (retryCount.current <= MAX_RETRIES) {
          const delay = BACKOFF_BASE_MS * Math.pow(2, retryCount.current - 1);
          await new Promise((resolve) => setTimeout(resolve, delay));
          return attemptDownload();
        }

        setError(errorMsg);
      }
    };

    await attemptDownload();
  }, []);

  return {
    ...store,
    checkForUpdate,
    startDownload,
  };
}
