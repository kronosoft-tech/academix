import { Download, X, RefreshCw } from 'lucide-react';
import { useUpdater } from '../hooks/useUpdater';
import { DownloadProgress } from './DownloadProgress';
import { truncateReleaseNotes } from '../utils/validators';

export function UpdateNotification() {
  const { status, updateInfo, downloadProgress, error, startDownload, dismiss, checkForUpdate } = useUpdater();

  // Only render when relevant
  if (status !== 'available' && status !== 'downloading' && status !== 'error') {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-50 w-96 bg-white border border-gray-200 rounded-lg shadow-lg p-4" role="alert">
      {/* Header */}
      <div className="flex items-start justify-between mb-2">
        <h3 className="text-sm font-semibold text-gray-900">
          Update Available: v{updateInfo?.version}
        </h3>
        {status !== 'downloading' && (
          <button
            onClick={dismiss}
            className="text-gray-400 hover:text-gray-600 p-1"
            aria-label="Dismiss update notification"
          >
            <X size={16} />
          </button>
        )}
      </div>

      {/* Release notes */}
      {status === 'available' && (
        <p className="text-xs text-gray-600 mb-3 max-h-24 overflow-y-auto">
          {updateInfo?.releaseNotes
            ? truncateReleaseNotes(updateInfo.releaseNotes)
            : 'Release notes unavailable'}
        </p>
      )}

      {/* Download progress */}
      {status === 'downloading' && (
        <div className="mb-3">
          <DownloadProgress progress={downloadProgress} />
        </div>
      )}

      {/* Error state */}
      {status === 'error' && error && (
        <p className="text-xs text-red-600 mb-3">{error}</p>
      )}

      {/* Actions */}
      <div className="flex gap-2">
        {status === 'available' && (
          <button
            onClick={startDownload}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded hover:bg-blue-700 transition-colors"
          >
            <Download size={14} />
            Download
          </button>
        )}

        {status === 'error' && (
          <button
            onClick={checkForUpdate}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded hover:bg-blue-700 transition-colors"
          >
            <RefreshCw size={14} />
            Retry
          </button>
        )}
      </div>
    </div>
  );
}
