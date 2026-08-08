# Authenticated User Menu Specification

## Purpose

Auth-aware navigation component that renders contextually: login CTA for anonymous visitors, avatar with dropdown menu for authenticated users. Works on ALL pages (public and protected).

## Requirements

### Requirement: Auth-Aware UserNavbar

The system MUST render a `UserNavbar` component on all pages that adapts based on authentication state.

When the user is NOT authenticated, the navbar MUST display a login button/CTA.

When the user IS authenticated, the navbar MUST display the user's avatar (or initials) with a dropdown menu.

#### Scenario: Anonymous visitor sees login CTA

- GIVEN a visitor who is not authenticated
- WHEN any page loads (public or protected)
- THEN the UserNavbar displays a login button linking to the auth flow

#### Scenario: Authenticated user sees dropdown menu

- GIVEN an authenticated user
- WHEN any page loads
- THEN the UserNavbar displays their avatar/initials
- AND clicking it reveals a dropdown menu

### Requirement: Dropdown Menu Items

The authenticated dropdown menu MUST contain the following navigation items: Dashboard, Mi Academia, Suscripción, Descargar App, and Cerrar sesión.

"Cerrar sesión" MUST clear the auth session and redirect to the home page.

#### Scenario: User navigates via dropdown

- GIVEN an authenticated user with the dropdown menu open
- WHEN they click "Dashboard"
- THEN they are navigated to `/dashboard`

#### Scenario: User logs out

- GIVEN an authenticated user with the dropdown menu open
- WHEN they click "Cerrar sesión"
- THEN the auth session is cleared
- AND the user is redirected to the home page

#### Scenario: Dropdown menu contains all required items

- GIVEN an authenticated user
- WHEN they open the dropdown menu
- THEN it contains exactly: Dashboard, Mi Academia, Suscripción, Descargar App, Cerrar sesión
