import nodemailer from 'nodemailer';

interface SendEmailOptions {
  to: string;
  subject: string;
  html: string;
}

const GMAIL_USER = import.meta.env.GMAIL_USER;
const GMAIL_APP_PASSWORD = import.meta.env.GMAIL_APP_PASSWORD;

function createTransporter() {
  if (!GMAIL_USER || !GMAIL_APP_PASSWORD) {
    return null;
  }

  return nodemailer.createTransport({
    service: 'gmail',
    auth: {
      user: GMAIL_USER,
      pass: GMAIL_APP_PASSWORD,
    },
  });
}

export async function sendEmail(options: SendEmailOptions): Promise<boolean> {
  const transporter = createTransporter();

  if (!transporter) {
    // In dev/preview without credentials, skip silently
    return true;
  }

  try {
    await transporter.sendMail({
      from: GMAIL_USER,
      to: options.to,
      subject: options.subject,
      html: options.html,
    });

    return true;
  } catch {
    return false;
  }
}
