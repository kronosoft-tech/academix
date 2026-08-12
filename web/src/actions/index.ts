import { contact } from './contact';
import { register } from './register';
import { login } from './login';
import { logout } from './logout';
import { passwordReset } from './password-reset';
import { adminLogin } from './admin-login';
import { cancelSub } from './cancel-subscription';
import { pqrs } from './pqrs';

export const server = {
  contact,
  register,
  login,
  logout,
  passwordReset,
  adminLogin,
  cancelSub,
  pqrs,
};
