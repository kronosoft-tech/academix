/// <reference types="astro/client" />

declare namespace App {
  interface Locals {
    user?: { id: string; email: string; role: string; type: 'customer' };
    admin?: { id: string; email: string; role: string; type: 'admin' };
  }
}
