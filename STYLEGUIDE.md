# ICPI Style Guide: Terminal Precision

> A data-first design system for serious traders. Built on shadcn/ui, customized for terminal aesthetics.

## Philosophy

ICPI's interface is a **financial data terminal**, not a consumer app. Every design decision prioritizes:
1. **Data legibility** over decoration
2. **Instant feedback** over smooth animations
3. **Information density** over white space
4. **Technical credibility** over friendly approachability

Users should feel like they're accessing a Bloomberg terminal, not a fintech toy.

---

## Color System

### Foundation Colors
```css
--terminal-black: #000000;           /* Pure black backgrounds */
--terminal-void: #0a0a0a;            /* Slightly elevated surfaces */
--terminal-coal: #141414;            /* Input fields, secondary surfaces */
--terminal-ash: #1f1f1f;             /* Hover states on dark surfaces */
```

### Terminal Green Spectrum
```css
--terminal-green: #00FF41;           /* Primary accent - bright matrix green */
--terminal-green-dim: #00CC34;       /* Hover/pressed states */
--terminal-green-glow: #00FF4120;    /* 12% opacity for backgrounds */
--terminal-green-pulse: #00FF4140;   /* 25% opacity for active elements */
--terminal-green-border: #00FF4160;  /* 38% opacity for borders */
```

### Semantic Colors
```css
--terminal-red: #FF0055;             /* Errors, negative values, sells */
--terminal-red-dim: #CC0044;         /* Hover state */
--terminal-red-glow: #FF005520;      /* Error backgrounds */

--terminal-yellow: #FFE600;          /* Warnings, pending states */
--terminal-yellow-dim: #CCB800;      /* Warning hover */
--terminal-yellow-glow: #FFE60020;   /* Warning backgrounds */

--terminal-blue: #00D9FF;            /* Info, links, secondary actions */
--terminal-blue-dim: #00AECC;        /* Info hover */
--terminal-blue-glow: #00D9FF20;     /* Info backgrounds */
```

### Text Colors
```css
--terminal-text-primary: #FFFFFF;    /* White - primary text */
--terminal-text-secondary: #999999;  /* Gray - labels, secondary info */
--terminal-text-tertiary: #666666;   /* Darker gray - disabled, hints */
--terminal-text-muted: #444444;      /* Very dark gray - placeholders */
```

### Usage Rules
- **Never use gradients** - terminal green or nothing
- **No color blending** - hard edges only
- **Data uses semantic colors** - green = positive, red = negative, white = neutral
- **UI uses terminal green only** - buttons, borders, accents
- **Backgrounds are always black or near-black** - no dark grays for main surfaces

---

## Typography

### Font Stack
```css
/* For all numeric data, code, and technical content */
--font-mono: 'IBM Plex Mono', 'Roboto Mono', 'Courier New', monospace;

/* For labels, headers, and UI text */
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

### Type Scale
```css
--text-xs: 11px;      /* Table labels, tiny annotations */
--text-sm: 12px;      /* Secondary labels, helper text */
--text-base: 14px;    /* Body text, form inputs */
--text-lg: 16px;      /* Card titles, section headers */
--text-xl: 20px;      /* Dashboard headers */
--text-2xl: 24px;     /* Page titles */
--text-3xl: 32px;     /* Hero numbers */
--text-4xl: 48px;     /* Featured statistics */
```

### Type Rules
- **All numbers use monospace** - prices, balances, percentages, timestamps
- **All labels use sans-serif** - "Balance", "TVL", "Last Updated"
- **No font weights below 400** - terminal aesthetic demands crispness
- **Use weight 600 for emphasis** - not 700 or 800
- **Letter-spacing on monospace: 0.5px** - improves number readability
- **Line-height for data: 1.2** - tight, dense information
- **Line-height for prose: 1.5** - if you must use paragraphs

---

## Spacing System

Terminal interfaces are **information-dense**. Use tighter spacing than typical web apps.

```css
--space-1: 4px;   /* Tight internal padding */
--space-2: 8px;   /* Standard gap between related elements */
--space-3: 12px;  /* Internal component padding */
--space-4: 16px;  /* Gap between component sections */
--space-6: 24px;  /* Gap between major sections */
--space-8: 32px;  /* Page-level spacing */
```

**Rules:**
- Table cells: 8px horizontal, 6px vertical
- Cards: 16px padding (not 24px)
- Form fields: 12px padding
- Button padding: 12px horizontal, 8px vertical
- Section gaps: 24px minimum

---

## Border & Radius System

### Border Widths
```css
--border-thin: 1px;    /* Standard borders */
--border-thick: 2px;   /* Active/focused states */
```

### Border Radius
```css
--radius-none: 0px;    /* Tables, inputs, most components */
--radius-sm: 2px;      /* Buttons, badges, cards */
```

**Rules:**
- **Default is 0px** - sharp rectangles are the norm
- **Use 2px only for** - buttons, badges, cards, alerts
- **Never use** - rounded-full, radius > 4px
- **Active borders glow** - add subtle box-shadow in terminal green

---

## Component Specifications

### Button

**Base Styles:**
```tsx
className="
  font-sans text-sm font-medium
  px-3 py-2
  border border-terminal-green-border
  bg-terminal-void
  text-terminal-green
  rounded-sm
  transition-none
  hover:bg-terminal-green-glow hover:border-terminal-green
  active:bg-terminal-green active:text-terminal-black
  disabled:opacity-40 disabled:cursor-not-allowed
"
```

**Variants:**

**Primary (Buy/Confirm actions):**
```tsx
bg-terminal-green text-terminal-black border-terminal-green
hover:bg-terminal-green-dim
active:bg-terminal-green-dim
```

**Danger (Sell/Delete actions):**
```tsx
bg-terminal-red text-white border-terminal-red
hover:bg-terminal-red-dim
active:bg-terminal-red-dim
```

**Ghost (Secondary actions):**
```tsx
bg-transparent border-terminal-ash text-terminal-text-secondary
hover:border-terminal-green-border hover:text-terminal-green
active:bg-terminal-green-glow
```

**Terminal (Execute/Run actions):**
```tsx
bg-terminal-black border-terminal-green text-terminal-green
font-mono uppercase tracking-wider text-xs
hover:shadow-[0_0_10px_rgba(0,255,65,0.3)]
active:shadow-[0_0_15px_rgba(0,255,65,0.5)]
/* ASCII arrow prefix: "> Execute" */
```

**Sizing:**
- sm: `px-2 py-1 text-xs`
- base: `px-3 py-2 text-sm`
- lg: `px-4 py-2.5 text-base`

**Rules:**
- No rounded corners for terminal variant
- No loading spinners - use text change: "Execute" → "Executing..." → "Done"
- Click feedback is instant background color change, no scale animation
- Icons: 16px, positioned left, 8px gap

---

### Card

**Base Styles:**
```tsx
className="
  bg-terminal-void
  border border-terminal-ash
  rounded-sm
  p-4
"
```

**Variants:**

**Data Card (Stats, metrics):**
```tsx
border-l-2 border-l-terminal-green border-y-0 border-r-0
bg-terminal-black
p-3
/* Add scan-line background */
background-image: repeating-linear-gradient(
  0deg,
  transparent,
  transparent 2px,
  rgba(0,255,65,0.02) 2px,
  rgba(0,255,65,0.02) 4px
)
```

**Chart Card:**
```tsx
border border-terminal-ash
bg-terminal-void
p-4
/* Grid background */
background-image:
  linear-gradient(rgba(0,255,65,0.05) 1px, transparent 1px),
  linear-gradient(90deg, rgba(0,255,65,0.05) 1px, transparent 1px);
background-size: 20px 20px;
```

**Alert Card:**
```tsx
border-l-4 border-l-terminal-green
border-y border-r border-terminal-green-border
bg-terminal-green-glow
```

**Header:**
```tsx
/* Card title */
text-terminal-text-secondary text-xs uppercase tracking-wider font-sans mb-1

/* Card value */
text-terminal-text-primary text-2xl font-mono font-semibold tracking-wide
```

**Rules:**
- No shadows (except for modals)
- No backdrop blur
- Headers are always uppercase, small, gray
- Primary value is always monospace, large, white
- Secondary info uses tertiary text color

---

### Input

**Base Styles:**
```tsx
className="
  w-full
  px-3 py-2
  bg-terminal-coal
  border border-terminal-ash
  rounded-none
  text-terminal-text-primary
  font-mono text-base
  placeholder:text-terminal-text-muted placeholder:font-sans
  focus:outline-none focus:border-terminal-green focus:ring-1 focus:ring-terminal-green
  disabled:opacity-40 disabled:cursor-not-allowed
"
```

**Variants:**

**Number Input:**
```tsx
/* For amounts, prices, percentages */
text-right
font-mono text-lg
tracking-wide
/* Show 8 decimal places for tokens */
```

**Search Input:**
```tsx
pl-9 /* space for icon */
font-sans /* not monospace */
bg-terminal-black
border-terminal-green-border
placeholder:text-terminal-text-tertiary
/* Icon: 16px magnifying glass, left: 12px */
```

**Terminal Input:**
```tsx
font-mono text-sm
bg-terminal-black
border-none
px-2 py-1
/* Prefix with "> " in green */
before:content-['>'] before:text-terminal-green before:mr-2
```

**States:**

**Error:**
```tsx
border-terminal-red
focus:border-terminal-red focus:ring-terminal-red
/* Error message: text-terminal-red text-xs font-sans mt-1 */
```

**Success:**
```tsx
border-terminal-green
/* Checkmark icon right side */
```

**Rules:**
- No border radius
- Focus ring is 1px, not 2px
- Transition only on border color
- Numeric inputs always right-aligned
- Max/half buttons positioned inside input (right side)

---

### Table

**Base Structure:**
```tsx
<table className="w-full border-collapse">
  <thead>
    <tr className="border-b-2 border-terminal-green">
      <th className="
        text-left px-2 py-1.5
        text-terminal-text-secondary text-xs uppercase tracking-wider font-sans font-medium
      ">
        Token
      </th>
    </tr>
  </thead>
  <tbody>
    <tr className="
      border-b border-terminal-ash
      hover:bg-terminal-ash
      transition-none
    ">
      <td className="
        px-2 py-1.5
        text-terminal-text-primary font-mono text-sm
      ">
        1,234.56789000
      </td>
    </tr>
  </tbody>
</table>
```

**Rules:**
- Headers: sans-serif, uppercase, gray, small
- Data cells: monospace, white, tight spacing
- Hover changes background instantly (no fade)
- No zebra striping - use borders only
- Numbers right-aligned
- Text left-aligned
- Header row gets thick green bottom border
- Last row has no border

**Cell Types:**

**Token Name:**
```tsx
flex items-center gap-2
text-terminal-text-primary font-sans font-medium text-sm
/* Icon: 20px, rounded-full */
```

**Numeric Value:**
```tsx
text-right
font-mono text-sm tracking-wide
text-terminal-text-primary
```

**Percentage Change:**
```tsx
text-right font-mono text-sm
/* Positive: text-terminal-green */
/* Negative: text-terminal-red */
/* With arrow: ▲ or ▼ */
```

**Status:**
```tsx
/* Use Badge component (see below) */
```

---

### Badge

**Base Styles:**
```tsx
className="
  inline-flex items-center
  px-2 py-0.5
  border border-current
  rounded-sm
  font-mono text-xs uppercase tracking-wider
"
```

**Variants:**

**Success:**
```tsx
bg-terminal-green-glow
border-terminal-green
text-terminal-green
```

**Error:**
```tsx
bg-terminal-red-glow
border-terminal-red
text-terminal-red
```

**Warning:**
```tsx
bg-terminal-yellow-glow
border-terminal-yellow
text-terminal-yellow
```

**Info:**
```tsx
bg-terminal-blue-glow
border-terminal-blue
text-terminal-blue
```

**Neutral:**
```tsx
bg-transparent
border-terminal-ash
text-terminal-text-secondary
```

**Status Dot Variant:**
```tsx
/* Add pulsing dot before text */
before:content-['']
before:w-1.5 before:h-1.5 before:rounded-full
before:bg-current
before:mr-1.5
before:animate-pulse
```

**Rules:**
- Always uppercase
- Always monospace
- Always bordered
- Pulse animation only for "live" states
- No icons inside badges
- Max 1-2 words

---

### Progress Bar

**Base Structure:**
```tsx
<div className="w-full bg-terminal-coal border border-terminal-ash h-2">
  <div
    className="h-full bg-terminal-green transition-none"
    style={{ width: '60%' }}
  />
</div>
```

**Variants:**

**Allocation Progress (with segments):**
```tsx
<div className="relative w-full h-3 bg-terminal-black border border-terminal-ash">
  {/* Each segment */}
  <div
    className="absolute top-0 left-0 h-full bg-terminal-green"
    style={{ width: '40%', left: '0%' }}
  />
  <div
    className="absolute top-0 h-full bg-terminal-blue"
    style={{ width: '30%', left: '40%' }}
  />
  {/* Grid lines every 10% */}
  <div className="absolute inset-0 grid grid-cols-10">
    {[...Array(9)].map((_, i) => (
      <div className="border-r border-terminal-ash/30" />
    ))}
  </div>
</div>
```

**Terminal Progress (with percentage label):**
```tsx
<div className="space-y-1">
  <div className="flex justify-between">
    <span className="text-terminal-text-secondary text-xs font-sans uppercase">Loading</span>
    <span className="text-terminal-green text-xs font-mono">68%</span>
  </div>
  <div className="w-full bg-terminal-black border border-terminal-green-border h-1">
    <div className="h-full bg-terminal-green" style={{ width: '68%' }} />
  </div>
</div>
```

**Rules:**
- No rounded corners on container or fill
- No gradients on fill
- Instant width changes (no animation) unless loading
- Always show percentage label for loading states
- Segmented bars use different semantic colors per segment

---

### Tabs

**Base Structure:**
```tsx
<div className="border-b border-terminal-ash">
  <nav className="flex -mb-px gap-6">
    <button className="
      px-1 py-2
      border-b-2 border-transparent
      text-terminal-text-secondary text-sm font-sans font-medium uppercase tracking-wide
      hover:text-terminal-text-primary hover:border-terminal-green-border
      data-[state=active]:border-terminal-green data-[state=active]:text-terminal-green
      transition-none
    ">
      Portfolio
    </button>
  </nav>
</div>
```

**Rules:**
- Underline style only (no filled background tabs)
- Active state: green underline + green text
- Hover: lighter green underline
- No icons in tabs
- Uppercase, sans-serif labels
- No rounded corners
- Instant color changes

---

### Switch

**Base Structure:**
```tsx
<button
  role="switch"
  className="
    relative inline-flex h-5 w-9
    items-center
    border border-terminal-ash
    bg-terminal-coal
    rounded-none
    data-[state=checked]:bg-terminal-green-glow
    data-[state=checked]:border-terminal-green
    transition-none
  "
>
  <span className="
    block h-3 w-3
    bg-terminal-text-secondary
    data-[state=checked]:bg-terminal-green
    data-[state=checked]:translate-x-5
    translate-x-1
    transition-transform duration-100
  " />
</button>
```

**With Label:**
```tsx
<label className="flex items-center gap-3 cursor-pointer">
  <span className="text-terminal-text-secondary text-sm font-sans">Auto Rebalance</span>
  <Switch />
  <span className="text-terminal-green text-xs font-mono font-medium">ON</span>
</label>
```

**Rules:**
- No border radius on container (use 0px)
- Thumb is square with 2px radius
- Instant background color change
- Thumb slides in 100ms
- Show ON/OFF text label next to switch
- ON = green, OFF = gray

---

### Alert

**Base Structure:**
```tsx
<div className="
  p-3
  border-l-4 border-l-terminal-green
  border-y border-r border-terminal-green-border
  bg-terminal-green-glow
  flex items-start gap-3
">
  <Icon className="w-5 h-5 text-terminal-green flex-shrink-0 mt-0.5" />
  <div className="flex-1 space-y-1">
    <h4 className="text-terminal-text-primary text-sm font-sans font-semibold">
      Transaction Pending
    </h4>
    <p className="text-terminal-text-secondary text-xs font-sans leading-relaxed">
      Your swap is being processed. This typically takes 2-5 seconds.
    </p>
  </div>
  <button className="text-terminal-text-tertiary hover:text-terminal-text-primary">
    <X className="w-4 h-4" />
  </button>
</div>
```

**Variants:**

**Success:**
```tsx
border-l-terminal-green
border-terminal-green-border
bg-terminal-green-glow
/* Icon: CheckCircle in terminal-green */
```

**Error:**
```tsx
border-l-terminal-red
border-terminal-red/40
bg-terminal-red-glow
/* Icon: AlertTriangle in terminal-red */
```

**Warning:**
```tsx
border-l-terminal-yellow
border-terminal-yellow/40
bg-terminal-yellow-glow
/* Icon: AlertCircle in terminal-yellow */
```

**Info:**
```tsx
border-l-terminal-blue
border-terminal-blue/40
bg-terminal-blue-glow
/* Icon: Info in terminal-blue */
```

**Rules:**
- Always left-bordered with thick semantic color
- Title is sans-serif, semibold, white
- Body is sans-serif, smaller, gray
- Icon is 20px, aligned to top
- Close button in top-right
- No border radius
- No animations on mount/unmount

---

### Skeleton Loading

**Base Pattern:**
```tsx
<div className="
  bg-terminal-coal
  animate-pulse
  h-6 w-32
" />
```

**Rules:**
- Use terminal-coal, not gray
- Pulse animation: `opacity: 1 → 0.5 → 1` every 2 seconds
- Match exact dimensions of content it replaces
- For numbers: use monospace width calculation
- For text: use realistic line lengths

**Table Skeleton:**
```tsx
<tr>
  <td className="px-2 py-1.5">
    <div className="h-4 w-16 bg-terminal-coal animate-pulse" />
  </td>
  <td className="px-2 py-1.5">
    <div className="h-4 w-24 bg-terminal-coal animate-pulse ml-auto" />
  </td>
</tr>
```

**Card Skeleton:**
```tsx
<div className="bg-terminal-void border border-terminal-ash rounded-sm p-4 space-y-3">
  <div className="h-3 w-20 bg-terminal-coal animate-pulse" />
  <div className="h-8 w-32 bg-terminal-coal animate-pulse" />
  <div className="h-3 w-full bg-terminal-coal animate-pulse" />
</div>
```

---

### Separator

**Base Styles:**
```tsx
/* Horizontal */
<div className="h-px w-full bg-terminal-ash" />

/* Vertical */
<div className="w-px h-full bg-terminal-ash" />
```

**Variants:**

**Section Divider:**
```tsx
<div className="relative h-px w-full bg-terminal-ash my-6">
  <div className="absolute left-0 top-0 w-12 h-px bg-terminal-green" />
</div>
```

**Terminal Line:**
```tsx
<div className="flex items-center gap-2 my-4">
  <div className="h-px flex-1 bg-terminal-green/30" />
  <span className="text-terminal-green text-xs font-mono uppercase px-2">End</span>
  <div className="h-px flex-1 bg-terminal-green/30" />
</div>
```

**ASCII Divider:**
```tsx
<div className="text-terminal-green/30 font-mono text-xs my-4">
  {'─'.repeat(60)}
</div>
```

**Rules:**
- Default is 1px solid line
- No gradients
- Use terminal-ash for subtle dividers
- Use terminal-green for emphasis
- ASCII characters acceptable for terminal aesthetic

---

## Special Effects

### Scan Lines (CRT monitor effect)

Apply to data-heavy cards:
```css
background-image: repeating-linear-gradient(
  0deg,
  transparent,
  transparent 2px,
  rgba(0, 255, 65, 0.02) 2px,
  rgba(0, 255, 65, 0.02) 4px
);
```

### Grid Background (graph paper)

Apply to chart containers:
```css
background-image:
  linear-gradient(rgba(0, 255, 65, 0.05) 1px, transparent 1px),
  linear-gradient(90deg, rgba(0, 255, 65, 0.05) 1px, transparent 1px);
background-size: 20px 20px;
```

### Glow Effect (active states)

Apply to focused/active elements:
```css
box-shadow: 0 0 10px rgba(0, 255, 65, 0.3);

/* Stronger glow for important actions */
box-shadow: 0 0 20px rgba(0, 255, 65, 0.5);
```

### Cursor Effects

```css
/* Default for interactive elements */
cursor: pointer;

/* For terminal-style interactions */
cursor: crosshair; /* on charts */
cursor: text; /* on data tables (for copying) */
```

---

## Animation Rules

**Allowed Animations:**
1. **Pulse** - for loading states (opacity: 1 → 0.5 → 1)
2. **Translate** - for switch toggles, slide-outs (100ms linear)
3. **Opacity** - for tooltips, dropdowns (100ms linear)
4. **Glow intensity** - for active states (box-shadow change)

**Forbidden Animations:**
1. ❌ Scale effects on buttons
2. ❌ Smooth color transitions (use instant or 50ms max)
3. ❌ Bounce, spring, elastic easings
4. ❌ Rotate (except for loading spinners)
5. ❌ Blur amount changes
6. ❌ Transform skew/perspective

**Timing:**
- Fast: 50ms - color changes, border changes
- Standard: 100ms - position changes, opacity
- Slow: 200ms - only for page transitions (if any)
- Pulse: 2000ms - loading skeletons, status indicators

---

## Data Visualization

### Chart Colors
```css
--chart-positive: #00FF41;  /* Gains, buy volume */
--chart-negative: #FF0055;  /* Losses, sell volume */
--chart-neutral: #666666;   /* Historical, inactive data */
--chart-accent-1: #00D9FF;  /* Token 1 in multi-asset chart */
--chart-accent-2: #FFE600;  /* Token 2 in multi-asset chart */
--chart-accent-3: #FF0055;  /* Token 3 in multi-asset chart */
```

### Chart Styling Rules
- Background: pure black (#000000)
- Grid lines: terminal-green at 5% opacity
- Axis labels: monospace, 11px, terminal-text-secondary
- No curved lines - use straight segments
- Data points: 4px squares (not circles)
- Tooltips: same as cards, appears instantly (no fade)

### Pie/Donut Charts
- Segments use semantic colors (green, blue, red, yellow)
- 2px gap between segments
- No drop shadows
- Center hole for donut: 60% of radius
- Labels: monospace, outside chart, with line connectors

### Bar Charts
- Bars are solid fills, no gradients
- 1px gap between bars
- Grid lines every 20% (5 lines total)
- Values displayed on top of bars (monospace)

---

## Icons

**Style:** Use `lucide-react` exclusively
**Size:** 16px default, 20px for emphasis, 12px for inline
**Color:** Match parent text color
**Stroke:** 2px (default for lucide)

**Common Icons:**
- `TrendingUp` / `TrendingDown` - price changes
- `ArrowUpRight` / `ArrowDownRight` - value movements
- `Activity` - rebalancing status
- `Clock` - timestamps, pending states
- `CheckCircle` - success states
- `AlertTriangle` - errors, warnings
- `Info` - informational alerts
- `ChevronRight` - navigation, expansion
- `Terminal` - execute actions

**Rules:**
- Never use filled icons (use outline only)
- Never use circular backgrounds
- Position left of text with 8px gap
- Use semantic colors (green = positive, red = negative)

---

## Responsive Behavior

**Breakpoints:**
```css
--screen-sm: 640px;   /* Mobile landscape */
--screen-md: 768px;   /* Tablet */
--screen-lg: 1024px;  /* Desktop */
--screen-xl: 1280px;  /* Large desktop */
```

**Rules:**
- Information density remains high on all screens
- Font sizes never shrink below 12px
- Tables scroll horizontally below 768px
- Cards stack vertically below 640px
- Never hide data - provide horizontal scroll
- Maintain monospace alignment on all screen sizes

---

## Copy & Microcopy

**Voice:** Technical, precise, confident
**Tone:** Matter-of-fact, no fluff

**Examples:**

✅ **Good:**
- "Execute trade"
- "Rebalance in 42 min"
- "0.5% slippage"
- "Error: Insufficient balance"

❌ **Bad:**
- "Let's get started!"
- "Almost there..."
- "Oops! Something went wrong"
- "Your portfolio is looking great!"

**Rules:**
- Use exact numbers (not "about" or "approximately")
- Use technical terms (don't dumb down)
- No exclamation marks
- No emoji (ever)
- No pleasantries ("please", "thank you")
- No apologies ("sorry")
- Time format: "42 min" not "42 minutes"
- Money format: "$1,234.56" not "$1.2K"

---

## Layout Patterns

### Dashboard Grid
```tsx
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
  {/* Stat cards */}
</div>
```

### Two-Column Layout
```tsx
<div className="grid grid-cols-1 lg:grid-cols-[2fr_1fr] gap-6">
  <main>{/* Charts, tables */}</main>
  <aside>{/* Trading panel */}</aside>
</div>
```

### Data Table Page
```tsx
<div className="space-y-6">
  <header>{/* Page title + actions */}</header>
  <div className="border border-terminal-ash">
    <table>{/* Dense data table */}</table>
  </div>
</div>
```

**Rules:**
- Use CSS Grid, not Flexbox for page layouts
- Gap between sections: 24px
- Container max-width: 1440px
- Horizontal padding: 16px mobile, 32px desktop
- No centering - content aligned to grid

---

## Forms

### Form Field Pattern
```tsx
<div className="space-y-2">
  <label className="block text-terminal-text-secondary text-xs uppercase tracking-wide font-sans">
    Amount
  </label>
  <div className="relative">
    <input
      type="number"
      className="w-full px-3 py-2 bg-terminal-coal border border-terminal-ash
                 font-mono text-lg text-right"
    />
    <button className="absolute right-2 top-1/2 -translate-y-1/2
                       text-terminal-green text-xs font-sans uppercase">
      Max
    </button>
  </div>
  <div className="flex justify-between text-xs">
    <span className="text-terminal-text-tertiary font-sans">Balance: </span>
    <span className="text-terminal-text-secondary font-mono">1,234.5678</span>
  </div>
</div>
```

### Form Layout
```tsx
<form className="space-y-4">
  {/* Fields stacked vertically */}
  <div className="flex gap-2">
    <button type="button" variant="ghost">Cancel</button>
    <button type="submit" variant="primary">Execute</button>
  </div>
</form>
```

**Rules:**
- Labels above inputs, not beside
- Helper text below input, right-aligned for numbers
- Error messages appear below input, red text
- Max/Clear buttons inside input (right side)
- Submit button is always right-most
- Cancel/Secondary actions on left

---

## Accessibility

**Requirements:**
- All colors meet WCAG AA contrast (4.5:1 minimum)
- Focus states use 2px green border + ring
- Keyboard navigation: Tab, Shift+Tab, Enter, Escape
- Screen reader labels on icon-only buttons
- Error messages linked to inputs via aria-describedby
- Loading states announced to screen readers

**Focus Visible:**
```css
:focus-visible {
  outline: 2px solid var(--terminal-green);
  outline-offset: 2px;
}
```

---

## Anti-Patterns

**Never do these:**

1. ❌ Rounded pills for buttons
2. ❌ Gradient backgrounds anywhere
3. ❌ Drop shadows on cards (flat only)
4. ❌ Colorful illustrations or graphics
5. ❌ Smooth scrolling
6. ❌ Parallax effects
7. ❌ Decorative animations
8. ❌ Non-monospace fonts for numbers
9. ❌ Emojis or icon fonts
10. ❌ Friendly/casual copy
11. ❌ Fading skeletons with animated gradients
12. ❌ Confetti or celebration effects
13. ❌ Modal overlays with blur (use solid black at 90% opacity)
14. ❌ Tabs with rounded backgrounds
15. ❌ Colorful category badges (green/red/gray only)

---

## Implementation Checklist

When building a new component:

- [ ] Uses correct color variables (not hardcoded colors)
- [ ] Numbers are in monospace font
- [ ] Labels are in sans-serif font
- [ ] Border radius is 0px or 2px only
- [ ] Transitions are 50-100ms or none
- [ ] Focus states use green border
- [ ] Hover states are instant
- [ ] No gradients used anywhere
- [ ] Spacing follows 4px grid
- [ ] Text is uppercase where appropriate
- [ ] Component works at mobile width
- [ ] Passes WCAG AA contrast
- [ ] Keyboard navigation works
- [ ] Looks like a terminal, not a web app

---

## Quick Reference Card

```
┌─────────────────────────────────────┐
│ ICPI TERMINAL CHEAT SHEET          │
├─────────────────────────────────────┤
│ BG:         #000000                 │
│ Text:       #FFFFFF / #999999       │
│ Accent:     #00FF41                 │
│ Error:      #FF0055                 │
│ Warning:    #FFE600                 │
├─────────────────────────────────────┤
│ Radius:     0px or 2px only         │
│ Border:     1px or 2px              │
│ Spacing:    4, 8, 12, 16, 24px      │
│ Animation:  50-100ms or none        │
├─────────────────────────────────────┤
│ Numbers:    IBM Plex Mono           │
│ Labels:     Inter                   │
│ Uppercase:  Labels, headers         │
│ Lowercase:  Body text only          │
├─────────────────────────────────────┤
│ No:         Gradients, curves       │
│             Shadows, blur           │
│             Emoji, friendly copy    │
│             Slow animations         │
├─────────────────────────────────────┤
│ Yes:        Sharp edges             │
│             Instant feedback        │
│             Dense information       │
│             Terminal aesthetic      │
└─────────────────────────────────────┘
```

---

**Last Updated:** 2025-09-30
**Version:** 1.0
**Maintained by:** ICPI Development Team
