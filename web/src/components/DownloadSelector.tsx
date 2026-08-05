import { useState, useEffect } from 'react';
import { Download, ChevronDown } from 'lucide-react';

interface DownloadOption {
  os: string;
  label: string;
  architectures: { arch: string; label: string }[];
}

const downloadOptions: DownloadOption[] = [
  {
    os: 'windows',
    label: 'Windows',
    architectures: [
      { arch: 'x64', label: 'x64 (64-bit)' },
      { arch: 'arm64', label: 'ARM64' },
    ],
  },
  {
    os: 'macos',
    label: 'macOS',
    architectures: [
      { arch: 'x64', label: 'Intel' },
      { arch: 'arm64', label: 'Apple Silicon' },
    ],
  },
  {
    os: 'linux',
    label: 'Linux',
    architectures: [
      { arch: 'x64', label: 'x64 (64-bit)' },
      { arch: 'arm64', label: 'ARM64' },
    ],
  },
];

function detectOS(): string {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes('win')) return 'windows';
  if (ua.includes('mac')) return 'macos';
  if (ua.includes('linux')) return 'linux';
  return 'windows';
}

export default function DownloadSelector() {
  const [detectedOS, setDetectedOS] = useState<string>('windows');
  const [selectedOS, setSelectedOS] = useState<string>('windows');
  const [selectedArch, setSelectedArch] = useState<string>('x64');
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    const os = detectOS();
    setDetectedOS(os);
    setSelectedOS(os);
  }, []);

  const currentOption = downloadOptions.find((o) => o.os === selectedOS);
  const downloadUrl = `/api/download?os=${selectedOS}&arch=${selectedArch}&version=latest`;

  return (
    <div className="max-w-lg mx-auto text-center">
      <div className="mb-8">
        <a
          href={downloadUrl}
          className="inline-flex items-center gap-3 px-8 py-4 text-lg font-semibold rounded-lg bg-emerald-600 text-white hover:bg-emerald-500 transition-colors"
          aria-label={`Descargar Academix para ${currentOption?.label} ${selectedArch}`}
        >
          <Download className="w-5 h-5" />
          Descargar para {currentOption?.label}
        </a>
        <p className="mt-3 text-sm text-slate-400">
          {currentOption?.label} ({selectedArch}) — Detectado automáticamente
        </p>
      </div>

      <button
        onClick={() => setShowAll(!showAll)}
        className="inline-flex items-center gap-2 text-sm text-slate-300 hover:text-white transition-colors"
        aria-expanded={showAll}
        aria-controls="download-options"
      >
        Otras plataformas
        <ChevronDown className={`w-4 h-4 transition-transform ${showAll ? 'rotate-180' : ''}`} />
      </button>

      {showAll && (
        <div id="download-options" className="mt-6 space-y-4">
          {downloadOptions.map((option) => (
            <div
              key={option.os}
              className="p-4 rounded-xl bg-slate-900 border border-slate-800"
            >
              <p className="font-medium text-white mb-3">{option.label}</p>
              <div className="flex flex-wrap justify-center gap-2">
                {option.architectures.map((arch) => (
                  <a
                    key={`${option.os}-${arch.arch}`}
                    href={`/api/download?os=${option.os}&arch=${arch.arch}&version=latest`}
                    className="inline-flex items-center gap-2 px-4 py-2 text-sm rounded-lg border border-slate-700 text-slate-300 hover:border-emerald-600 hover:text-white transition-colors"
                    aria-label={`Descargar para ${option.label} ${arch.label}`}
                  >
                    <Download className="w-3.5 h-3.5" />
                    {arch.label}
                  </a>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
