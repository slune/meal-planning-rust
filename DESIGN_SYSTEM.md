# Design System - Boy Scout Meal Planner

## Overview

This document defines the design system for the Boy Scout Meal Planner application, including colors, typography, spacing, and component patterns.

## Colors

### Primary Palette

```css
--color-primary: theme('colors.blue.600');      /* #2563eb - Primary actions */
--color-danger: theme('colors.red.600');        /* #dc2626 - Delete, errors */
--color-success: theme('colors.green.600');     /* #16a34a - Success states */
--color-warning: theme('colors.yellow.600');    /* #ca8a04 - Warnings */
--color-info: theme('colors.blue.500');         /* #3b82f6 - Info messages */
```

### Text Colors

```css
--color-text-primary: theme('colors.slate.800');     /* #1e293b - Headings, primary text */
--color-text-secondary: theme('colors.slate.600');   /* #475569 - Secondary text */
```

### Background Colors

- `bg-slate-50` (#f8fafc) - Page background
- `bg-white` (#ffffff) - Cards, modals, forms
- `bg-slate-100` (#f1f5f9) - Disabled states

### Semantic Colors

Use Tailwind's color utilities consistently:

- **Success**: `green-*` (50, 100, 600, 700)
- **Error**: `red-*` (50, 100, 600, 700)
- **Warning**: `yellow-*` (50, 100, 600, 700)
- **Info**: `blue-*` (50, 100, 500, 600)
- **Neutral**: `slate-*` (100, 200, 300, 600, 700, 800)

**IMPORTANT**: Always use `slate-*` colors, never `gray-*`. This ensures consistency across the application.

## Typography

### Heading Hierarchy

```css
h1, .heading-1 { @apply text-3xl font-bold text-slate-800; }
h2, .heading-2 { @apply text-2xl font-bold text-slate-800; }
h3, .heading-3 { @apply text-xl font-bold text-slate-700; }
```

### Font Sizes

- **Large**: `text-lg` (1.125rem / 18px)
- **Base**: `text-base` (1rem / 16px) - Default
- **Small**: `text-sm` (0.875rem / 14px)
- **Extra Small**: `text-xs` (0.75rem / 12px)

### Font Weights

- **Bold**: `font-bold` (700) - Headings, labels, buttons
- **Semibold**: `font-semibold` (600) - Secondary emphasis
- **Medium**: `font-medium` (500) - Subtle emphasis
- **Normal**: `font-normal` (400) - Body text

### Font Family

System font stack for optimal performance:
```
-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif
```

## Spacing

### Container Padding

- Page container: `p-8` (2rem / 32px)
- Card padding: `p-8` (2rem / 32px)

### Vertical Spacing

- **Section gaps**: `space-y-6` (1.5rem / 24px)
- **Form fields**: `space-y-4` (1rem / 16px)
- **List items**: `space-y-3` (0.75rem / 12px)
- **Tight spacing**: `space-y-2` (0.5rem / 8px)

### Horizontal Spacing

- **Button groups**: `gap-2` or `gap-3`
- **Grid gaps**: `gap-6` (1.5rem / 24px)
- **Inline elements**: `gap-2` (0.5rem / 8px)

### Margin Scale

Use Tailwind's spacing scale consistently:
- `m-2` (0.5rem / 8px)
- `m-4` (1rem / 16px)
- `m-6` (1.5rem / 24px)
- `m-8` (2rem / 32px)

## Components

### Buttons

#### Variants

```rust
<Button variant=ButtonVariant::Primary>
  "Primary Action"
</Button>

<Button variant=ButtonVariant::Secondary>
  "Secondary Action"
</Button>

<Button variant=ButtonVariant::Danger>
  "Delete"
</Button>
```

#### States

- **Default**: Blue/slate/red background with white text
- **Hover**: Darker background, scale-105 transform, larger shadow
- **Active**: scale-95 transform
- **Disabled**: 50% opacity, not-allowed cursor
- **Loading**: Spinner with aria-busy="true"

#### Sizes

- **Small**: `btn-sm` - px-4 py-2 text-sm
- **Medium**: Default - px-6 py-3 text-base
- **Large**: `btn-lg` - px-8 py-4 text-lg

#### Icon Buttons

Always include `aria-label` for accessibility:

```rust
<IconButton
    variant=ButtonVariant::Danger
    aria_label="Delete camp"
    on_click=/* ... */
>
  "🗑️"
</IconButton>
```

### Cards

```css
.card {
  @apply bg-white rounded-xl shadow-md p-8;
  @apply border border-slate-200 transition-all duration-200;
}

.card:hover {
  @apply shadow-xl border-blue-300;
}
```

#### Card Grid Layout

```rust
<div class="card-grid">
  // Cards here
</div>
```

Uses responsive grid: 1 column (mobile), 2 columns (tablet), 3 columns (desktop).

### Modals

#### Confirm Modal

Used for destructive actions requiring user confirmation:

```rust
<ConfirmModal
    show=show_modal.into()
    on_confirm=move || { /* action */ }
    on_cancel=move || set_show_modal.set(false)
    title="Confirm Deletion"
    message="This action cannot be undone. Are you sure?"
    confirm_text="Delete"
    cancel_text="Cancel"
    variant="danger"
/>
```

#### Alert Modal

Used for informational alerts:

```rust
<AlertModal
    show=show_alert.into()
    on_close=move || set_show_alert.set(false)
    title="Success"
    message="Your changes have been saved."
    close_text="OK"
/>
```

#### Keyboard Shortcuts

- **ESC**: Close modal
- **Enter**: Confirm action (ConfirmModal only)

### Toast Notifications

#### Usage

```rust
use_toast().success("Camp created successfully!");
use_toast().error("Failed to save changes");
use_toast().info("Loading data...");
use_toast().warning("Connection is slow");
```

#### Helper Functions

```rust
toast_success("Operation completed!");
toast_error("An error occurred");
toast_info("Did you know...");
toast_warning("Please check your input");
```

#### Auto-Dismiss Times

- **Success**: 4 seconds
- **Error**: 6 seconds (longer for reading error messages)
- **Info**: 4 seconds
- **Warning**: 5 seconds

#### Positioning

Toasts appear in the top-right corner, stacking vertically with 12px gaps.

### Forms

#### Form Field

```rust
<FormField
    label="Camp Name"
    required=true
    error=error_message.get()
    help_text=Some("Enter a descriptive name")
>
    <Input
        value=name.into()
        on_input=move |val| set_name.set(val)
        placeholder="Summer Camp 2024"
        required=true
        has_error=error_message.get().is_some()
    />
</FormField>
```

#### Input Types

- **Text Input**: `<Input input_type="text" />`
- **Number Input**: `<Input input_type="number" min="0" step="1" />`
- **TextArea**: `<TextArea rows=4 />`
- **Select**: `<Select>...</Select>`
- **Checkbox**: `<Checkbox label="..." />`

#### Validation

Always validate on both client and server:

**Client-side**:
```rust
if name.trim().is_empty() {
    toast_error("Name is required");
    return;
}
```

**Server-side** (in API layer):
```rust
if start_date >= end_date {
    return Err(sqlx::Error::Decode("Invalid date range".into()));
}
```

### Loading States

#### Spinner

```rust
<Spinner size="medium".to_string() />
<Spinner size="small".to_string() />
<Spinner size="large".to_string() />
```

#### Loading Overlay

For full-page loading:

```rust
<LoadingOverlay
    show=is_loading.into()
    message="Generating report..."
/>
```

#### Skeleton Loader

For content placeholders:

```rust
<SkeletonLoader rows=5 />
<CardSkeleton count=3 />
```

### Alerts

Use CSS classes for inline alerts:

```rust
<div class="alert-error">
  "An error occurred while saving"
</div>

<div class="alert-success">
  "Changes saved successfully"
</div>

<div class="alert-info">
  "You have 3 camps scheduled"
</div>
```

### Empty States

Consistent pattern for empty data:

```rust
<div class="card text-center py-16 bg-gradient-to-br from-slate-50 to-blue-50 border-2 border-dashed border-slate-300">
    <div class="text-7xl mb-6">"🏕️"</div>
    <h3 class="text-2xl font-bold text-slate-800 mb-3">"No camps yet"</h3>
    <p class="text-lg text-slate-600 mb-8">
      "Get started by creating your first camp"
    </p>
    <button class="btn btn-primary">"+ Create Camp"</button>
</div>
```

#### Empty State Icons

- Camps: 🏕️
- Recipes: 🍳
- Ingredients: 🥕
- Meals: 📅
- Reports: 📊

## Accessibility

### ARIA Labels

#### Buttons

All icon-only buttons must have `aria-label`:

```rust
<button aria-label="Delete camp">🗑️</button>
```

#### Form Fields

- Use `aria-required="true"` for required fields
- Use `aria-invalid="true"` when field has error
- Use `aria-describedby` to link error messages

```rust
<input
    aria-required="true"
    aria-invalid=has_error
    aria-describedby="field-error"
/>
<div id="field-error" role="alert">{error_message}</div>
```

#### Loading States

```rust
<div class="spinner" role="status" aria-label="Loading">
    <span class="sr-only">"Loading..."</span>
</div>
```

### Keyboard Navigation

- **Tab**: Navigate between interactive elements
- **Enter**: Activate buttons, submit forms
- **Space**: Toggle checkboxes, activate buttons
- **ESC**: Close modals, cancel actions

### Focus Management

- Modals should trap focus
- After modal closes, return focus to trigger element
- Visible focus indicators (blue ring)

### Screen Reader Support

- Use semantic HTML (`<nav>`, `<main>`, `<article>`, `<button>`)
- Include `.sr-only` text for icon-only elements
- Use `role="status"` for non-critical updates
- Use `role="alert"` for errors and important messages

## Responsive Design

### Breakpoints

- **Mobile**: < 640px
- **Tablet**: 640px - 1024px
- **Desktop**: > 1024px

### Grid Layouts

```rust
class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6"
```

### Navigation

Desktop: Horizontal navigation bar
Mobile: Consider hamburger menu (Phase 8 - optional)

## Animation & Transitions

### Duration

- **Fast**: `duration-150` (150ms) - Hover effects
- **Normal**: `duration-200` (200ms) - Default transitions
- **Slow**: `duration-300` (300ms) - Modals, drawers

### Easing

- `ease-out` - Elements entering
- `ease-in` - Elements exiting
- `ease-in-out` - State changes

### Transform

- **Hover**: `scale-105` (5% larger)
- **Active**: `scale-95` (5% smaller)
- **Slide In**: `translateY(-20px)` → `translateY(0)`

## Best Practices

### Do's

✅ Use semantic HTML elements
✅ Include ARIA labels for all interactive elements
✅ Validate input on both client and server
✅ Show loading states during async operations
✅ Provide success feedback after actions
✅ Use modals for confirmations, toasts for notifications
✅ Keep button text action-oriented ("Create Camp", not "Submit")
✅ Reset forms after successful submission
✅ Use consistent spacing and colors throughout

### Don'ts

❌ Don't use `window.confirm()` or `window.alert()`
❌ Don't use `gray-*` colors (use `slate-*` instead)
❌ Don't create icon-only buttons without `aria-label`
❌ Don't skip form validation
❌ Don't leave users guessing after actions (show feedback)
❌ Don't use inline styles (use CSS classes)
❌ Don't hardcode colors (use design tokens)
❌ Don't forget to handle error states
❌ Don't mix different UI patterns for the same action

## Component Library Imports

```rust
use crate::components::{
    // Modals
    ConfirmModal, AlertModal,

    // Toasts
    use_toast, toast_success, toast_error, toast_info, toast_warning,

    // Buttons
    Button, IconButton, ButtonVariant, ButtonSize,

    // Loading
    Spinner, LoadingOverlay, SkeletonLoader, CardSkeleton, LoadingSpinner,

    // Forms
    FormField, Input, TextArea, Select, Checkbox,
    validate_required, validate_email, validate_min_length,
    validate_number_range, validate_positive_number,
};
```

## Testing Checklist

- [ ] All buttons keyboard accessible (Tab, Enter, Space)
- [ ] Modals close with ESC key
- [ ] Focus returns to trigger after modal closes
- [ ] Error messages have `role="alert"`
- [ ] Loading spinners have `role="status"`
- [ ] Icon buttons have `aria-label`
- [ ] Form fields have proper labels and validation
- [ ] Colors have sufficient contrast (WCAG AA)
- [ ] App works with screen reader
- [ ] All interactions provide feedback (toast/modal/visual change)
