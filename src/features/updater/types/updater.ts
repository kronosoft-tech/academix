export interface UpdateInfo {
  version: string;
  releaseNotes: string;
  date: string;
  mandatory: boolean;
}

export interface UpdaterState {
  status:
    | 'idle'
    | 'checking'
    | 'available'
    | 'downloading'
    | 'installing'
    | 'error';
  updateInfo: UpdateInfo | null;
  downloadProgress: number;
  error: string | null;
  dismissedVersion: string | null;
}

export interface UpdaterActions {
  checkForUpdate: () => Promise<void>;
  startDownload: () => Promise<void>;
  dismiss: () => void;
  reset: () => void;
}

export interface DownloadProgress {
  downloaded: number;
  total: number | null;
  percentage: number;
}

export interface ArtifactEntry {
  url: string;
  signature: string;
}
