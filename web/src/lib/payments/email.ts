export const prerender = false;

import nodemailer from 'nodemailer';

const transporter = nodemailer.createTransport({
  service: 'gmail',
  auth: {
    user: import.meta.env.GMAIL_USER,
    pass: import.meta.env.GMAIL_APP_PASSWORD,
  },
});

export async function sendTrialReminder(
  email: string,
  daysLeft: number,
  academyName: string
): Promise<void> {
  const subject = `Te quedan ${daysLeft} días de prueba en Academix`;
  const html = `
    <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
      <h2 style="color: #1a1a2e;">Hola, ${academyName}</h2>
      <p style="font-size: 16px; color: #333;">
        Tu periodo de prueba en <strong>Academix</strong> termina en <strong>${daysLeft} día${daysLeft === 1 ? '' : 's'}</strong>.
      </p>
      <p style="font-size: 16px; color: #333;">
        Suscríbete ahora para seguir usando todas las funcionalidades sin interrupción.
      </p>
      <div style="text-align: center; margin: 30px 0;">
        <a href="${import.meta.env.SITE_URL}/pricing"
           style="background-color: #6366f1; color: white; padding: 14px 28px; text-decoration: none; border-radius: 8px; font-size: 16px; font-weight: bold;">
          Ver Planes
        </a>
      </div>
      <p style="font-size: 14px; color: #666;">
        Si tienes alguna pregunta, responde a este correo y te ayudaremos.
      </p>
    </div>
  `;

  await transporter.sendMail({
    from: `"Academix" <${import.meta.env.GMAIL_USER}>`,
    to: email,
    subject,
    html,
  });
}

export async function sendGraceWarning(
  email: string,
  daysLeft: number,
  academyName: string
): Promise<void> {
  const subject = `Tu pago está vencido — ${daysLeft} días para suspensión`;
  const html = `
    <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
      <h2 style="color: #dc2626;">Atención, ${academyName}</h2>
      <p style="font-size: 16px; color: #333;">
        Tu pago en <strong>Academix</strong> está vencido. Te quedan <strong>${daysLeft} día${daysLeft === 1 ? '' : 's'}</strong> antes de que tu cuenta sea suspendida.
      </p>
      <p style="font-size: 16px; color: #333;">
        Actualiza tu método de pago para evitar la interrupción del servicio.
      </p>
      <div style="text-align: center; margin: 30px 0;">
        <a href="${import.meta.env.SITE_URL}/pricing"
           style="background-color: #dc2626; color: white; padding: 14px 28px; text-decoration: none; border-radius: 8px; font-size: 16px; font-weight: bold;">
          Actualizar Pago
        </a>
      </div>
      <p style="font-size: 14px; color: #666;">
        Si crees que esto es un error, responde a este correo.
      </p>
    </div>
  `;

  await transporter.sendMail({
    from: `"Academix" <${import.meta.env.GMAIL_USER}>`,
    to: email,
    subject,
    html,
  });
}

export async function sendPaymentSuccess(
  email: string,
  planName: string,
  amount: number
): Promise<void> {
  const subject = 'Pago confirmado — Academix';
  const formattedAmount = new Intl.NumberFormat('es-CO', {
    style: 'currency',
    currency: 'COP',
    minimumFractionDigits: 0,
  }).format(amount);

  const html = `
    <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
      <h2 style="color: #16a34a;">¡Pago confirmado!</h2>
      <p style="font-size: 16px; color: #333;">
        Tu pago ha sido procesado exitosamente.
      </p>
      <div style="background-color: #f3f4f6; border-radius: 8px; padding: 20px; margin: 20px 0;">
        <p style="margin: 8px 0; font-size: 16px;"><strong>Plan:</strong> ${planName}</p>
        <p style="margin: 8px 0; font-size: 16px;"><strong>Monto:</strong> ${formattedAmount}</p>
      </div>
      <p style="font-size: 14px; color: #666;">
        Gracias por confiar en Academix para la gestión de tu academia.
      </p>
    </div>
  `;

  await transporter.sendMail({
    from: `"Academix" <${import.meta.env.GMAIL_USER}>`,
    to: email,
    subject,
    html,
  });
}
