# Subscription Reminders Specification

## Purpose

Send daily email reminders via nodemailer/Gmail during trial countdown and grace period warnings.

## Requirements

### Requirement: Trial Countdown Reminders

The system MUST send daily email reminders during the trial period showing days remaining.

#### Scenario: Trial reminder sent

- GIVEN a user with status=trialing and trial_ends_at > now
- WHEN the send-reminders cron runs daily
- THEN an email is sent with subject containing days remaining (e.g., "5 days left in your trial")
- AND the email includes a CTA to subscribe

#### Scenario: No reminder after trial ends

- GIVEN a user with status=expired (trial ended)
- WHEN the send-reminders cron runs
- THEN no trial reminder is sent for that user

### Requirement: Grace Period Warnings

The system MUST send daily email warnings during grace period indicating payment is overdue.

#### Scenario: Grace warning sent

- GIVEN a user with status=grace and grace_expires_at > now
- WHEN the send-reminders cron runs
- THEN an email is sent warning of overdue payment with days until access loss
- AND the email includes a link to update payment method

### Requirement: Email Configuration

The system MUST use nodemailer with Gmail App Password credentials from environment variables (GMAIL_USER, GMAIL_APP_PASSWORD).

#### Scenario: Email sent successfully

- GIVEN valid GMAIL_USER and GMAIL_APP_PASSWORD in environment
- WHEN a reminder email is triggered
- THEN the email is delivered via Gmail SMTP

#### Scenario: Email credentials missing

- GIVEN GMAIL_USER or GMAIL_APP_PASSWORD is not set
- WHEN the cron attempts to send
- THEN the error is logged and the cron continues without crashing
