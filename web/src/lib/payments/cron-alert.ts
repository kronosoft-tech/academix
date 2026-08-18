import { sendEmail } from '../email';

const SUPPORT_EMAIL = import.meta.env.SUPPORT_EMAIL || 'support@academix.app';

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}

/**
 * Alert support about an unexpected cron failure: prominent log + email.
 * NEVER throws — a failed alert must not mask the cron error that triggered
 * it (the caller always responds 500 regardless).
 */
export async function sendCronAlert(handler: string, error: unknown): Promise<void> {
  console.error(`[cron:${handler}] FAILED:`, error);

  const detail = error instanceof Error ? error.stack ?? error.message : String(error);

  try {
    await sendEmail({
      to: SUPPORT_EMAIL,
      subject: `[Academix Cron] ${handler} failed`,
      html: `
        <h2>Cron <code>${escapeHtml(handler)}</code> failed</h2>
        <p>An unexpected error occurred while running the <code>${escapeHtml(handler)}</code> cron.</p>
        <pre style="background:#f6f6f6;padding:12px;border-radius:6px;overflow:auto;">${escapeHtml(detail)}</pre>
      `,
    });
  } catch {
    // Best-effort alert — the 500 from the handler is preserved regardless.
  }
}