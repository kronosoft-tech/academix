import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { UpdateNotification } from './UpdateNotification';

const mockStartDownload = vi.fn();
const mockDismiss = vi.fn();
const mockCheckForUpdate = vi.fn();

vi.mock('../hooks/useUpdater', () => ({
  useUpdater: vi.fn(),
}));

import { useUpdater } from '../hooks/useUpdater';
const mockedUseUpdater = vi.mocked(useUpdater);

function mockUpdaterState(overrides: Partial<ReturnType<typeof useUpdater>> = {}) {
  const defaults: ReturnType<typeof useUpdater> = {
    status: 'idle',
    updateInfo: null,
    downloadProgress: 0,
    error: null,
    dismissedVersion: null,
    startDownload: mockStartDownload,
    dismiss: mockDismiss,
    checkForUpdate: mockCheckForUpdate,
    setStatus: vi.fn(),
    setUpdateInfo: vi.fn(),
    setProgress: vi.fn(),
    setError: vi.fn(),
    reset: vi.fn(),
  };
  mockedUseUpdater.mockReturnValue({ ...defaults, ...overrides });
}

describe('UpdateNotification', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders when update is available with version and release notes', () => {
    mockUpdaterState({
      status: 'available',
      updateInfo: {
        version: '2.1.0',
        releaseNotes: 'Bug fixes and improvements',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByText(/v2\.1\.0/)).toBeInTheDocument();
    expect(screen.getByText('Bug fixes and improvements')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /download/i })).toBeInTheDocument();
  });

  it('shows "Release notes unavailable" when releaseNotes is empty', () => {
    mockUpdaterState({
      status: 'available',
      updateInfo: {
        version: '3.0.0',
        releaseNotes: '',
        date: '2024-06-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    expect(screen.getByText('Release notes unavailable')).toBeInTheDocument();
  });

  it('does not render when status is idle', () => {
    mockUpdaterState({ status: 'idle' });

    const { container } = render(<UpdateNotification />);

    expect(container.innerHTML).toBe('');
  });

  it('does not render when status is checking', () => {
    mockUpdaterState({ status: 'checking' });

    const { container } = render(<UpdateNotification />);

    expect(container.innerHTML).toBe('');
  });

  it('does not render when status is installing', () => {
    mockUpdaterState({ status: 'installing' });

    const { container } = render(<UpdateNotification />);

    expect(container.innerHTML).toBe('');
  });

  it('calls dismiss when dismiss button is clicked', () => {
    mockUpdaterState({
      status: 'available',
      updateInfo: {
        version: '2.0.0',
        releaseNotes: 'New features',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    const dismissButton = screen.getByLabelText('Dismiss update notification');
    fireEvent.click(dismissButton);

    expect(mockDismiss).toHaveBeenCalledTimes(1);
  });

  it('calls startDownload when Download button is clicked', () => {
    mockUpdaterState({
      status: 'available',
      updateInfo: {
        version: '2.0.0',
        releaseNotes: 'Improvements',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    const downloadButton = screen.getByRole('button', { name: /download/i });
    fireEvent.click(downloadButton);

    expect(mockStartDownload).toHaveBeenCalledTimes(1);
  });

  it('shows error message and Retry button in error state', () => {
    mockUpdaterState({
      status: 'error',
      error: 'Network connection failed',
      updateInfo: {
        version: '2.0.0',
        releaseNotes: '',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    expect(screen.getByText('Network connection failed')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /download/i })).not.toBeInTheDocument();
  });

  it('calls checkForUpdate when Retry button is clicked', () => {
    mockUpdaterState({
      status: 'error',
      error: 'Timeout',
      updateInfo: {
        version: '2.0.0',
        releaseNotes: '',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    const retryButton = screen.getByRole('button', { name: /retry/i });
    fireEvent.click(retryButton);

    expect(mockCheckForUpdate).toHaveBeenCalledTimes(1);
  });

  it('does not show dismiss button while downloading', () => {
    mockUpdaterState({
      status: 'downloading',
      downloadProgress: 45,
      updateInfo: {
        version: '2.0.0',
        releaseNotes: '',
        date: '2024-01-01',
        mandatory: false,
      },
    });

    render(<UpdateNotification />);

    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.queryByLabelText('Dismiss update notification')).not.toBeInTheDocument();
  });
});
