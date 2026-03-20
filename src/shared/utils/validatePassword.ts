export function validatePassword(password: string): { valid: boolean; message: string } {
  if (!password) {
    return { valid: false, message: "La contraseña es requerida" };
  }
  if (password.length < 6) {
    return { valid: false, message: "La contraseña debe tener al menos 6 caracteres" };
  }
  return { valid: true, message: "" };
}

export function validatePasswordStrength(password: string): {
  valid: boolean;
  score: number;
  message: string;
} {
  if (!password) {
    return { valid: false, score: 0, message: "La contraseña es requerida" };
  }

  let score = 0;
  const messages: string[] = [];

  if (password.length >= 8) {
    score += 1;
  } else {
    messages.push("al menos 8 caracteres");
  }

  if (/[a-z]/.test(password)) {
    score += 1;
  } else {
    messages.push("una minúscula");
  }

  if (/[A-Z]/.test(password)) {
    score += 1;
  } else {
    messages.push("una mayúscula");
  }

  if (/[0-9]/.test(password)) {
    score += 1;
  } else {
    messages.push("un número");
  }

  if (/[^a-zA-Z0-9]/.test(password)) {
    score += 1;
  } else {
    messages.push("un carácter especial");
  }

  const valid = score >= 3;
  const message = valid
    ? "Contraseña segura"
    : `Agregue ${messages.join(", ")}`;

  return { valid, score, message };
}
