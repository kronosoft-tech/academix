# Tasks: Color Palette Customization

## Project: Academix
## Change: color-pallete-customization

---

## Phase 1: Foundation / Infrastructure

### 1.1 Add @property CSS declarations to index.css
- Task: Add @property declarations in src/index.css for reactivity
- Description: Define CSS custom properties with @property syntax to enable reactive updates
- Files: src/index.css
- Verify: TypeScript compilation passes

### 1.2 Create src/theme/theme.ts
- Task: Create theme utility functions
- Description: Implement updateCSSVariable(), loadColors(), and getDefaultTheme() functions
- Files: src/theme/theme.ts (new)
- Verify: File exports functions without errors

---

## Phase 2: Settings Page Implementation

### 2.1 Create SettingsPage.tsx with 5 color pickers
- Task: Create new settings page component
- Description: Build SettingsPage with color pickers for primary, secondary, accent, background, text categories using Tailwind color grid
- Files: src/pages/settings/SettingsPage.tsx (new)
- Verify: Page renders 5 color picker sections

### 2.2 Integrate theme utilities in SettingsPage
- Task: Connect SettingsPage to theme.ts
- Description: Import and use updateCSSVariable() and loadColors() functions
- Files: src/pages/settings/SettingsPage.tsx
- Verify: Color selection updates CSS variables

---

## Phase 3: Accounting UI

### 3.1 Replace buttons with dropdown in AccountingPage
- Task: Modify AccountingPage to use dropdown for color-based actions
- Description: Replace action buttons with dropdown that uses theme colors
- Files: src/features/accounting/routes/AccountingPage.tsx
- Verify: Dropdown displays theme colors correctly

---

## Phase 4: Navigation Integration

### 4.1 Add Settings route in App.tsx
- Task: Add route for Settings page
- Description: Import SettingsPage and add /settings route
- Files: src/app/App.tsx
- Verify: Route matches /settings path

### 4.2 Add Settings link in MainLayout
- Task: Add Settings navigation link
- Description: Add Settings link to navigation in MainLayout
- Files: src/app/layouts/MainLayout.tsx
- Verify: Link appears in navigation menu

---

## Phase 5: Testing

### 5.1 Verify colors apply globally
- Task: Test global color application
- Description: Change colors in settings, verify they apply across all pages
- Verify: Colors reflect in UI components

### 5.2 Verify persistence works
- Task: Test localStorage persistence
- Description: Change colors, reload app, verify colors persist
- Verify: Colors load correctly after page refresh