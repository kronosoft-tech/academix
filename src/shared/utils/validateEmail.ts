export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validateEmailWithMessage(email: string): { valid: boolean; message: string } {
  if (!email) {
    return { valid: false, message: "El correo electrónico es requerido" };
  }
  if (!validateEmail(email)) {
    return { valid: false, message: "Ingrese un correo electrónico válido" };
  }
  return { valid: true, message: "" };
}
