# StubHouse — Marketing Website Specification

> **"Stub it. Ship it."**
> Companion document to `spec.md`. Defines the brand system, site architecture, and every page's content and behavior for `stubhouse.dev`.
> Version: 1.0.0-draft | Status: Implementation-Ready

---

## Table of Contents

1. [North Star](#1-north-star)
2. [Brand System](#2-brand-system)
   - 2.1 [Logo](#21-logo)
   - 2.2 [Color](#22-color)
   - 2.3 [Typography](#23-typography)
   - 2.4 [Voice & Tone](#24-voice--tone)
   - 2.5 [Motion](#25-motion)
   - 2.6 [Iconography & Imagery](#26-iconography--imagery)
3. [Theme System](#3-theme-system)
4. [Site Architecture](#4-site-architecture)
5. [Component Library](#5-component-library)
6. [Page Specifications](#6-page-specifications)
   - 6.1 [Home (`/`)](#61-home-)
   - 6.2 [Mock Server (`/mocks`)](#62-mock-server-mocks)
   - 6.3 [Pricing (`/pricing`)](#63-pricing-pricing)
   - 6.4 [Download (`/download`)](#64-download-download)
   - 6.5 [Docs gateway (`/docs`)](#65-docs-gateway-docs)
   - 6.6 [Changelog (`/changelog`)](#66-changelog-changelog)
   - 6.7 [Blog (`/blog`)](#67-blog-blog)
   - 6.8 [Manifesto (`/manifesto`)](#68-manifesto-manifesto)
7. [Navigation & Footer](#7-navigation--footer)
8. [Animations & Interactions](#8-animations--interactions)
9. [Responsive Behavior](#9-responsive-behavior)
10. [SEO, Metadata, Social](#10-seo-metadata-social)
11. [Performance Budget](#11-performance-budget)
12. [Accessibility](#12-accessibility)
13. [Tech Stack](#13-tech-stack)
14. [Asset Checklist](#14-asset-checklist)
15. [Launch Checklist](#15-launch-checklist)

---

## 1. North Star

The marketing site exists to do three things, in this order:

1. **Communicate the dual-mode pitch in under five seconds.** A visitor arrives and immediately understands: this is an API client *and* a mock server, both first-class, native, local. If they leave with only that single idea, the site has worked.
2. **Make the developer want to download it.** Not subscribe. Not "request access." Download. The CTA is a binary on their machine inside thirty seconds.
3. **Survive the comparison test.** A skeptical visitor lands here from a Hacker News thread, looks for the catch, and finds none. Every claim is concrete, every screenshot is real, every comparison is honest.

### Reference Aesthetic

The site should feel like it belongs in the same room as: **Zed**, **Linear**, **Vercel**, **Ghostty**, **Anthropic**, **Stripe Press**. Specifically:

- Typography that's a feature, not decoration
- Real product imagery, never illustrations of laptops or abstract gradients
- Whitespace measured in `rem`, not pixels
- Motion that's structural, not decorative — fades and translates, never bounces
- Code samples with the same care as marketing copy
- A monochrome discipline that signals confidence

What the site is **not**: a SaaS landing page. There is no "Get a demo," no chatbot bubble, no "Trusted by Fortune 500" carousel of greyed-out logos, no testimonial slider with stock-photo headshots. The product is the proof.

---

## 2. Brand System

### 2.1 Logo

The mark is a hollow house silhouette enclosing three offset, stacked "stubs." Each stub carries a single small port hole, suggesting an endpoint or a connection. The mark is **pure monochrome** — white on black, black on white, no exceptions.

#### Construction

- The house outline is built from two parallel chevron strokes meeting at the apex
- Three stubs sit center-stacked, each offset slightly from the one below to suggest depth and modularity
- The lowest stub carries an additional small slot detail (read as a USB port / data port)
- Stroke weights are uniform across the mark; corners are softly rounded (not pill-rounded)

#### Variants

| Variant | Use |
|---|---|
| **Mark** (the house silhouette alone) | Favicon, app icon, dock badge, GitHub avatar, footer accent, social og:image corner |
| **Wordmark** (StubHouse in display weight) | Header navigation, secondary placements where the mark alone wouldn't read |
| **Lockup** (mark + wordmark side-by-side) | Press kit, headers in long-form documents, sponsorship placements |
| **Lockup, stacked** (mark above wordmark) | Posters, business cards, vertical formats |

#### Clear Space

Minimum clear space around the mark equals **the height of one stub** in the construction. No type, no edge, no other element may enter that space.

#### Sizing

- Minimum size: **16px** (favicon-grade — the mark must remain identifiable)
- Recommended minimum on web: **24px**
- Hero placements: **64px–128px** depending on viewport

#### What Not to Do

- Never colorize the mark
- Never apply gradients, shadows, or glows
- Never rotate or skew
- Never place on a busy photographic background — always a flat surface (`#0A0A0A`, `#FAFAF7`, or a single solid neutral)
- Never combine with an emoji or icon-set glyph

### 2.2 Color

Pure monochrome. No accent. The website will feel disciplined the same way the logo does.

#### Tokens

```
/* Dark theme (default) */
--bg-canvas:        #0A0A0A    /* page background */
--bg-surface:       #141414    /* cards, code blocks, raised surfaces */
--bg-surface-2:     #1C1C1C    /* nested surfaces, inputs */
--border-subtle:    rgba(255,255,255,0.06)
--border-default:   rgba(255,255,255,0.10)
--border-strong:    rgba(255,255,255,0.18)
--text-primary:     #F5F5F2    /* headings, foreground */
--text-secondary:   #A3A3A0    /* body copy, descriptions */
--text-tertiary:    #6E6E6B    /* captions, metadata */
--text-disabled:    #3F3F3D
--code-comment:     #6E6E6B
--code-keyword:     #F5F5F2    /* yes — monochrome syntax */
--code-string:      #C4C4C0
--code-punct:       #6E6E6B
--shimmer:          rgba(255,255,255,0.04)

/* Light theme */
--bg-canvas:        #FAFAF7    /* warm bone, never pure white */
--bg-surface:       #FFFFFF
--bg-surface-2:     #F2F2EE
--border-subtle:    rgba(10,10,10,0.06)
--border-default:   rgba(10,10,10,0.10)
--border-strong:    rgba(10,10,10,0.18)
--text-primary:     #0A0A0A
--text-secondary:   #525250
--text-tertiary:    #82827F
--text-disabled:    #BABAB7
--code-comment:     #82827F
--code-keyword:     #0A0A0A
--code-string:      #3F3F3D
--code-punct:       #82827F
--shimmer:          rgba(10,10,10,0.03)
```

#### Notes on the Discipline

Pure black (`#000`) and pure white (`#FFF`) are deliberately avoided. `#0A0A0A` and `#FAFAF7` are softer at scale and read as intentional rather than default. The light theme background carries a subtle warmth (a touch of yellow-cream) that pairs with the off-black to feel editorial rather than clinical. Borders are alpha-channel based so they layer correctly over surfaces of any tone.

The only place a non-monochrome value is permissible is **status semantics inside live product UI screenshots** (a `200` is green, a `500` is red — that's product reality, not brand). Brand-side, this never leaks.

### 2.3 Typography

The type system is built from a single sans family for everything UI-adjacent and a matched mono for code, with one editorial serif reserved for moments that earn it.

#### Primary Stack — Geist

```
--font-display: 'Geist', 'Inter Tight', system-ui, -apple-system, sans-serif;
--font-sans:    'Geist', 'Inter', system-ui, -apple-system, sans-serif;
--font-mono:    'Geist Mono', 'JetBrains Mono', ui-monospace, 'SF Mono', monospace;
--font-serif:   'Newsreader', 'Tiempos Text', Georgia, serif;  /* editorial accent only */
```

Geist is the default for both display and body. It's free (OFL), modern, geometric, and was designed specifically for technical interfaces. Using one family for both display and body is a deliberate choice — it signals craft restraint, the same way the monochrome palette does.

#### Alternative Stacks

For a paid/premium upgrade path, in priority order:

| Display + Body | Mono | Notes |
|---|---|---|
| **ABC Diatype** | **ABC Diatype Mono** | Linear's typeface. The most "Zed-tier" pairing money can buy. Paid. |
| **Söhne** + **Söhne Mono** | (matched) | Klim Type Foundry. Anthropic-adjacent. Paid. |
| **GT America** + **GT America Mono** | (matched) | Grilli Type. Paid. |
| **Inter Tight** + **Inter** + **JetBrains Mono** | — | All free, classic. |

The site ships with **Geist + Geist Mono** at launch. A typeface upgrade is treated as a future investment, not a blocker.

#### Type Scale

A hard, non-fluid scale. No `clamp()` headlines. Sizes are defined per-breakpoint (see § 9).

```
/* Desktop (≥1024px) */
--text-display-1:  88px / 1.02 / -0.04em   /* hero headline only */
--text-display-2:  64px / 1.04 / -0.035em  /* section openers */
--text-display-3:  44px / 1.08 / -0.03em   /* sub-section */
--text-h1:         36px / 1.12 / -0.025em
--text-h2:         28px / 1.18 / -0.02em
--text-h3:         22px / 1.25 / -0.015em
--text-h4:         18px / 1.35 / -0.01em
--text-body-lg:    19px / 1.55 / -0.005em
--text-body:       16px / 1.6  /  0
--text-body-sm:    14px / 1.55 /  0
--text-mono:       14px / 1.6  /  0
--text-mono-sm:    13px / 1.55 /  0
--text-caption:    12px / 1.5  /  0.01em
--text-eyebrow:    13px / 1.4  /  0.06em   /* tracking-wider, sentence case */
```

#### Weights

Two weights only across the entire site: **Regular (400)** and **Medium (500)**. No bold, no extra-bold. The hero headline is set in 500 at 88px — it doesn't need more weight to feel heavy. Restricting to two weights is what makes the typography feel expensive.

#### Casing Rules

- **Sentence case** for all headlines, all section titles, all button labels
- **Never** Title Case
- **Never** ALL CAPS, with a single exception: the eyebrow micro-label above section headlines may use small-caps via `font-variant-caps: all-small-caps` for a refined effect (see Stripe Press, Vercel)
- Numbers in headlines use **tabular figures** (`font-variant-numeric: tabular-nums`)

#### Tracking

- Display sizes ≥44px get **negative tracking** (-0.025em to -0.04em). Default web type is too loose at large sizes.
- Body copy stays at **0**.
- Eyebrow labels and small caps get **+0.06em**.

### 2.4 Voice & Tone

The product is opinionated and the copy reflects that. A short voice charter:

#### What StubHouse Sounds Like

- **Direct.** "Files, not databases." Not "leveraging a file-based architecture for collaborative API definition workflows."
- **Confident, not boastful.** "The fastest API client we know of" is a claim. "The world's #1 leading API platform" is marketing.
- **Technical without jargon-cruft.** Use precise terms (`embedded server`, `route trie`, `passthrough`) freely. Never use empty filler (`leverage`, `synergize`, `next-generation`).
- **Restrained.** A homepage section can be three sentences. Whitespace is part of the message.
- **Slightly literary in the manifesto, never in the marketing.** Big philosophical claims are quarantined to one page.

#### Words We Use

`Local-first`, `native`, `offline`, `embedded`, `mock`, `stub`, `scenario`, `passthrough`, `recording`, `sandboxed`, `git-friendly`, `binary`, `port`, `endpoint`, `rule`, `trie`, `fixture`, `seed`, `fault`, `chaos`, `headless`, `CI`.

#### Words We Don't Use

`Solution`, `platform` (unless precise), `enterprise-grade`, `world-class`, `revolutionary`, `unleash`, `empower`, `seamless`, `cutting-edge`, `next-gen`, `leverage`, `synergy`, `ecosystem`, `journey`. No filler adverbs (`truly`, `simply`, `easily`).

#### Examples

> ❌ Easily build and test APIs with our truly seamless, next-generation API platform.
> ✅ An API client and a mock server in one binary. Files instead of a database.

> ❌ Empower your team to leverage best-in-class mocking workflows.
> ✅ Record a real API. Go offline. Replay it.

> ❌ Get started in seconds with our intuitive interface.
> ✅ Open a folder. Send a request. Done.

#### Headline Patterns

The site uses three headline patterns and rotates between them:

1. **The Imperative.** Two or three words, an action. *"Stub it. Ship it." · "Open a folder." · "Mock the future."*
2. **The Declaration.** A flat fact stated without hedging. *"The mock server is the product." · "Files are the source of truth."*
3. **The Inversion.** Set up an industry default, deny it. *"Most API clients treat mocks as an add-on. We don't."*

### 2.5 Motion

Motion exists to **structure** the experience, not decorate it. Three motion primitives. No others.

#### The Three Primitives

1. **Fade-up (8px).** Elements enter by translating up 8px while fading from `opacity: 0` to `1`. Duration `400ms`, ease `cubic-bezier(0.22, 1, 0.36, 1)`.
2. **Crossfade.** Two elements swap with a 200ms opacity exchange. Used for theme toggle, scenario switcher demo, before/after comparisons.
3. **Cursor-follow parallax (max 12px).** Hero elements respond to cursor position with a *very* subtle (≤12px) parallax. Disabled below 1024px and when `prefers-reduced-motion`.

#### Forbidden Motion

- No bounces, no springs that overshoot
- No "whoosh" enter animations from off-screen
- No sticky-scroll-jacking that hijacks the scroll wheel
- No looping background "particle" effects
- No mouse-trail effects, no cursor swap
- No auto-playing video carousels

#### Scroll-Triggered Behavior

A single pattern: as a section enters the viewport (root-margin: -10%), elements within fade-up in sequence with a 60ms stagger. The IntersectionObserver triggers once and never reverses.

#### Hover States

- Buttons: `200ms` background + border transition
- Cards: a `1px` border-color shift (`--border-default` → `--border-strong`), no transform, no shadow
- Links: underline appears `100ms` after hover begins, with a subtle `2px` offset (`text-underline-offset: 4px`)

### 2.6 Iconography & Imagery

#### Icons

The site uses a single icon set throughout: **Lucide** (the Feather successor), with a custom subset of marks added for product-specific concepts (port, scenario, fixture, route, fault). All icons are stroke-only, 1.5px, monochrome, on a 24px grid.

No filled icons. No icon backgrounds. No icon plates.

#### Product Imagery

Every screenshot, every UI render, every animated demo on the marketing site uses the **real product running with the real default theme**. No mocked-up Figma slides masquerading as product. If the site shows a feature, that feature exists.

Screenshots are captured at `2x` resolution, exported as both `webp` (primary) and `avif` (fallback for older browsers). They are placed inside a subtle browser/window chrome to ground the user — never floating in space.

#### Photography

There is none. StubHouse uses no photography. No team photos, no office shots, no abstract textures. The product, the typography, and the empty space carry the entire visual load.

---

## 3. Theme System

The site supports light and dark themes. Dark is the default. Theme is set by a CSS class on `<html>` (`.theme-dark` / `.theme-light`) and resolved in this priority order:

1. URL param (`?theme=dark|light`) — highest priority, used for shareable links
2. User preference, persisted in `localStorage` under `stubhouse-theme`
3. OS preference via `prefers-color-scheme`
4. Default: **dark**

#### Theme Toggle

A single icon button in the top-right of the navigation. Sun icon when in dark mode, moon icon when in light. Click cycles `dark → light → system`. The current state is announced to screen readers.

#### Implementation Note

Theme tokens are CSS custom properties scoped to the theme class. There is **no JavaScript theme flicker on load**: an inline blocking script in `<head>` reads `localStorage` and applies the theme class before first paint. This is the one and only blocking script allowed on the site.

```html
<script>
  (function() {
    var t = localStorage.getItem('stubhouse-theme');
    if (!t || t === 'system') {
      t = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    }
    document.documentElement.className = 'theme-' + t;
  })();
</script>
```

#### Theme-Aware Assets

Two sets of imagery exist for every product screenshot — a dark-theme capture and a light-theme capture. The site swaps them via `<picture>` with media queries on `prefers-color-scheme`, falling back to the active theme class.

The logo SVG uses `currentColor` for fill, inheriting the active text color. One file, both themes, no duplication.

---

## 4. Site Architecture

```
/                       — Home
/mocks                  — Mock server deep dive
/pricing                — Pricing (free + future tiers)
/download               — Platform download routing
/docs                   — External link or subdomain (docs.stubhouse.dev)
/changelog              — Version history
/blog                   — Engineering blog index
/blog/[slug]            — Individual post
/manifesto              — Long-form philosophy piece
/legal/privacy          — Privacy policy
/legal/terms            — Terms of use
/legal/license          — Open source license
/press                  — Press kit (logo files, screenshots)
/404                    — Not found
/500                    — Server error
```

External:
- **GitHub:** `github.com/stubhouse/stubhouse` (repo)
- **Docs:** `docs.stubhouse.dev` (separate site, mdx-based)
- **Status:** `status.stubhouse.dev` (mock server registry / community status)
- **Discord:** `discord.gg/stubhouse`

---

## 5. Component Library

The site is built from twelve primitives. No more.

| Component | Purpose |
|---|---|
| `<Button>` | Primary, secondary, ghost variants. Always one icon optional. |
| `<Link>` | Inline link with hover underline. |
| `<Card>` | Surface container with optional border and padding scales. |
| `<Code>` | Monospace inline. |
| `<CodeBlock>` | Multi-line code with optional filename header, language label, copy button. |
| `<Eyebrow>` | The small-caps label above section headings. |
| `<Heading>` | Display-1 through h4, scale-aware. |
| `<Prose>` | Body text container with measured line-length (max 65ch). |
| `<Tabs>` | Switch between code/UI views in feature sections. |
| `<Comparison>` | The competitive comparison table component. |
| `<DemoFrame>` | Browser-chrome wrapper for product screenshots and embedded demos. |
| `<NavBar>` / `<Footer>` | Site chrome. |

### CodeBlock Specification

The single most important visual element on the site after the hero. Specifications:

- Filename header bar at top with language label on the right
- Copy button appears on hover, top-right of the block
- Line numbers optional, off by default for inline marketing snippets, on for documentation-style usage
- Syntax highlighting uses a **monochrome theme** — keywords, strings, and comments are differentiated by weight and opacity, not by hue. This is non-negotiable and reinforces the brand discipline.
- Maximum width: container width. Long lines wrap or scroll horizontally with a soft fade indicator on the right edge.
- Background: `--bg-surface`. Border: `--border-subtle`.
- Font: `--font-mono`, 14px desktop / 13px mobile.

### Button Specification

Three variants:

```
Primary:    bg = text-primary,  fg = bg-canvas,    border = none
Secondary:  bg = transparent,   fg = text-primary, border = border-default
Ghost:      bg = transparent,   fg = text-secondary, border = none, hover bg = bg-surface
```

Sizes: `sm` (32px height), `md` (40px), `lg` (48px). Hero CTAs use `lg`. In-section CTAs use `md`.

A button can carry a single 16px leading or trailing icon. Never both.

---

## 6. Page Specifications

### 6.1 Home (`/`)

The home page is the most important page on the site. Every other page is downstream of a visitor who decided, on the home page, to keep reading.

#### Section 1 — Hero

**Layout.** Full-bleed, centered, vertical alignment biased toward upper-third of viewport. On desktop, the hero takes ~85vh; on mobile, ~75vh.

**Content.**

```
[Mark, 64px, monochrome]

Eyebrow:    Local-first API client + mock server
Headline:   Stub it. Ship it.   (display-1, 88px, weight 500)
Subhead:    StubHouse is the desktop API client that takes mocking
            seriously. Native, offline, file-based — built for the
            developer who has to ship before the API exists.
            (body-lg, --text-secondary, max-width: 580px)

Primary CTA:    [↓ Download for macOS]    (auto-detects OS)
Secondary CTA:  [View on GitHub →]
                (with star count badge, fetched at build-time)
```

**Visual.** Below the CTAs, a **looping product capture (~12s, silent, autoplay, no controls)** showing: open a workspace → send a real request → toggle the mock server on → switch a scenario from `success` to `not_found` → see the response change live. The capture is wrapped in `<DemoFrame>` chrome with a subtle title bar reading `StubHouse — my-project`.

The capture should be exported as **AVIF/WebP video** (≤2MB total), poster image as the first frame for instant render.

#### Section 2 — The Dual-Mode Pitch

**Eyebrow:** Two tools. One binary.

**Headline:** A request client and a mock server, both first-class.

**Body:** Other API clients bolt on mocks as a paid afterthought or a separate process. StubHouse was built around the assumption that you spend half your day calling APIs and the other half pretending an API exists. Both deserve a real tool.

**Visual.** A side-by-side composition:

- **Left panel:** A request being sent to `https://api.example.com/users` with a 200 response.
- **Right panel:** The mock server panel with three rules visible — `GET /users`, `POST /users`, `GET /users/:id` — and scenario dropdowns showing `success`, `not_found`, `slow_success`.

The two panels are presented as if they are sub-windows of the same app, because they are.

#### Section 3 — Files, Not a Database

**Eyebrow:** Source of truth.

**Headline:** Your API definitions live in a folder you own.

**Body:** No proprietary database. No cloud account required. A workspace is a `.stubhouse/` directory of YAML files: requests, mock rules, environments, scripts. Commit it to git. Diff it in PRs. Move it between machines with `cp -r`. The tool builds on top of files; the files outlive the tool.

**Visual.** A `<CodeBlock filename=".stubhouse/collections/users/get-user.yaml">` showing a real request definition (the example from `spec.md` § 6.1), paired beside a screenshot of the same request rendered in the StubHouse UI. The implication: edit the file, see it in the app. Edit in the app, see it in the file.

#### Section 4 — The Mock Server, in Depth

A scrolling section with **three sub-pillars**, each presented as a row with copy on the left and a visual on the right (alternating sides on alternating rows).

##### Sub-pillar A — Scenarios

> Switch your mock from `success` to `not_found` to `server_error` without restarting anything. From the UI, from the CLI, or from your test runner via the control API.

Visual: A live demo widget where the visitor can click between three scenario buttons and the response payload below changes in real time. **This is interactive on the marketing site itself** — no live server needed, just a small client-side state machine that swaps a JSON blob and a status code badge.

##### Sub-pillar B — Stateful mocks

> A mock that actually behaves like an API. POST creates. PUT updates. DELETE removes. GET returns what you wrote. All in memory, all reset on demand.

Visual: A code/UI split showing a `mock_resources` declaration and a small terminal pane where four `curl` commands run against the in-memory store, demonstrating real CRUD.

##### Sub-pillar C — Recording mode

> Point StubHouse at a real API. Make some calls. Save them. Replay them offline. Forever.

Visual: An animated diagram showing requests flowing **through** StubHouse to a real upstream, then a toggle flips and the same requests are now answered locally from saved YAML.

#### Section 5 — Fault Injection

**Eyebrow:** Chaos, on demand.

**Headline:** Test the unhappy path before production does.

**Body:** Connection resets. Timeouts. Slow responses. Partial bodies. Random 5xx at a configurable rate. Every fault is a checkbox. Your retry logic finally has something to retry against.

**Visual.** A grid of six toggle cards, each labeled with a fault type. Hovering a card reveals its YAML config. The whole grid animates subtly to suggest these are real toggles.

#### Section 6 — Scripting & Tests

**Eyebrow:** Automation, sandboxed.

**Headline:** Logic that doesn't need a Node.js runtime.

**Body:** StubHouse uses Rhai — a sandboxed scripting language designed for embedding in Rust. Pre-request scripts. Post-response assertions. Mock rule conditions. All run in a sandbox with no filesystem, no network, no escape. No 50MB V8 binary in your dock.

**Visual.** A `<CodeBlock language="rhai">` showing test assertions:

```rhai
test("Status is 200")        { response.status == 200 }
test("Body has user field")  { response.json()["user"] != null }
test("Response under 500ms") { response.time_ms < 500 }
```

…paired with a screenshot of the test runner panel showing pass/fail with timing.

#### Section 7 — CI / Headless

**Eyebrow:** It's a binary.

**Headline:** `stubhouse serve` and ship.

**Body:** The same engine that runs in the desktop app runs from the command line. Spin up your mocks in CI. Run your assertion suite against them. Get JUnit XML out the other side. Ten kilobytes of YAML replaces a thousand lines of `nock`.

**Visual.** A terminal pane showing the example from `spec.md` § 9 (CI/CD usage) with output rendered in the same monochrome syntax theme used elsewhere.

#### Section 8 — Comparison

**Eyebrow:** Honestly.

**Headline:** Where StubHouse fits.

**Body:** A direct, honest comparison with Postman, Insomnia, Bruno, and Yaak. The component is `<Comparison>` — a horizontally-scrolling table on mobile, a full grid on desktop. Cells use ✓ or ✕ glyphs (NOT colored) and a brief footnote where context matters.

The comparison data lives in `/data/comparison.yaml` and renders the table from § 12 of `spec.md` verbatim.

A small line below the table reads: *"This table is sourced from public information as of [date] and is updated when competitors ship features. Corrections welcome — open a PR."*

#### Section 9 — Built on Rust

**Eyebrow:** Under the hood.

**Headline:** Native binary. No Electron.

**Body:** Tauri 2 shell. Rust core. `hyper` HTTP runtime. `rustls` for TLS. Embedded mock server runs in a Tokio task on the same process as the UI. ~25MB installed. Cold start in under 200ms on a base-model M1.

**Visual.** A small horizontal bar chart comparing **install size** and **cold-start time** of StubHouse vs Postman vs Insomnia. The chart is monochrome — bars are filled with `--text-primary`, the StubHouse bar is solid, others are striped via SVG `<pattern>`. Numbers are shown on each bar as caption text.

> Note: All numbers must be measured, not estimated, before launch.

#### Section 10 — Open Source

**Eyebrow:** It's free. It's open.

**Headline:** Apache 2.0. Forever.

**Body:** StubHouse is open source under Apache 2.0. No usage tracking. No telemetry without an explicit opt-in. No "Pro" features held hostage. The full source — desktop app, CLI, mock runtime — is on GitHub. PRs welcome.

**Visual.** A simulated terminal showing `git clone github.com/stubhouse/stubhouse && cd stubhouse && cargo run` with output indicating a successful build.

#### Section 11 — Final CTA

**Layout.** Full-bleed, vertically centered, 70vh.

**Content.**

```
Headline:   Stub it. Ship it.    (display-1, 88px)
Subhead:    Free. Open source. Available for macOS, Linux, and Windows.
            (body-lg, --text-secondary)

Primary CTA:    [↓ Download for macOS]
Secondary CTA:  [View on GitHub →]
```

A single horizontal divider line (`1px`, `--border-subtle`) below the section, then the footer.

---

### 6.2 Mock Server (`/mocks`)

A dedicated deep-dive page for the feature that defines the product. Visitors land here from the Section 4 sub-pillars on the home page, from search ("mock server alternative"), and from documentation cross-references.

#### Sections

1. **Hero** — "The mock server is the product." (display-2). Subhead positions this page as the long-form treatment of the home page's Section 4.
2. **Architecture diagram** — A full-width diagram (the one in `spec.md` § 4) re-styled for marketing — same boxes, more whitespace, monochrome.
3. **Rule definition anatomy** — A live, annotated YAML block where each field has a hover tooltip explaining what it does. Hovering `priority:` highlights its line and shows a tooltip; same for `matcher.method`, `scenarios`, `passthrough`, `fault`.
4. **Route matching** — Animated diagram of the priority trie. As a path gets typed in an input field, the diagram highlights the matched path through the trie. (Pre-rendered animation, no live server.)
5. **Scenarios** — Same as home Section 4A, expanded with three more scenario examples.
6. **Stateful mocks** — Walkthrough of declaring `mock_resources` with `auto_crud: true` and seeing all five HTTP methods light up automatically.
7. **Recording mode** — Step-by-step, with screen captures.
8. **Fault injection** — All six fault types listed with their YAML and the symptom they simulate.
9. **The control API** — Full table of `/__mirage/*` endpoints with `curl` examples.
10. **CI integration** — Real example of using StubHouse in GitHub Actions.
11. **CTA** — Download.

This page is the single longest page on the site (~3000 words of body copy) and is the page most likely to be linked-to from external sources.

---

### 6.3 Pricing (`/pricing`)

#### Tier 1 — Free, forever

Everything in `spec.md` § 5 is free. The desktop app, the mock server, the CLI, the scripting engine, the plugin system. No login. No telemetry. No nag screens.

#### Tier 2 — Cloud (planned, not yet shipped)

A dedicated row indicates a future hosted offering — workspace sync, team collaboration, hosted mock servers reachable from public URLs — without committing to a date. The row reads:

> **Cloud.** Coming later. Want early access?
> [Email signup form, single field]

The signup is honest: no countdown timers, no "Limited spots." Just a list of people who'd like to know.

#### Layout

A single-column page. No "compare plans" 4-column matrix. The current state of the world is one tier (Free), and pretending otherwise dilutes the message.

---

### 6.4 Download (`/download`)

#### Hero

A single, prominent download button with auto-detected OS. Below it, a small "Choose another platform" toggle that expands to:

| Platform | Architectures | Format |
|---|---|---|
| macOS | Apple Silicon, Intel | `.dmg`, Homebrew tap |
| Linux | x86_64, ARM64 | `.deb`, `.rpm`, `.AppImage`, `.tar.gz` |
| Windows | x86_64, ARM64 | `.msi`, Scoop |

Below the matrix, a "Build from source" section with the `cargo install` instructions and a link to the GitHub release notes for the current version.

#### Verification

Below the downloads, **SHA256 checksums** for every artifact in a copy-able mono block, plus a one-line `gpg` verification command. This is the kind of detail that signals "made by people who care" to the audience that matters.

---

### 6.5 Docs Gateway (`/docs`)

If docs live on a separate subdomain (`docs.stubhouse.dev`), the `/docs` route on the marketing site is a **gateway page** that:

- Hero: "Documentation"
- Three large cards linking to: **Getting Started**, **Mock Server Guide**, **API Reference**
- Below: a search bar that proxies to the docs subdomain's search endpoint
- A list of the 10 most-viewed doc pages (curated, updated quarterly)

If docs are co-located, `/docs` is the docs root.

---

### 6.6 Changelog (`/changelog`)

A single long-form page listing every release, newest first. Each release entry has:

- Version number (large, mono, weight 500)
- Release date
- A short prose summary (1–2 sentences)
- A list of changes grouped under headers: **New**, **Improved**, **Fixed**, **Breaking**
- A direct link to the GitHub release

The changelog is generated from the GitHub releases API at build time. Manual prose summaries live in front-matter on each release tag in the repo.

---

### 6.7 Blog (`/blog`)

The engineering blog. Sparse posting cadence — quality over schedule. Topics are technical:

- "How we built a route trie matcher in Rust"
- "Why we chose Rhai over embedded V8"
- "Recording HTTP at the edge of a Tokio server"

#### Index (`/blog`)

A single column of posts in reverse chronological order. Each entry shows: title (h3), date, reading time, a 2-line excerpt. No tags, no categories, no filters at v1. The list will grow slowly enough that filters aren't yet needed.

#### Post (`/blog/[slug]`)

Markdown-rendered, max-width `680px` for prose, full-width for code blocks and diagrams. Author byline at top. A small "share on" footer at the bottom (links to copy URL, X, Hacker News submit). No social-share counters.

The serif typeface (`--font-serif`) is permitted **only** here, for the blog post body, **and only** for posts tagged `essay` (not for technical posts). Technical posts use `--font-sans` body throughout.

---

### 6.8 Manifesto (`/manifesto`)

The one page where the writing gets to be writerly. Long-form. Single column. Centered. Body in `--font-serif`. No images. No code blocks.

The manifesto argues for files-as-source-of-truth, for local-first tooling, for the dignity of offline work, for the belief that the API client should be the calmest tool on a developer's desktop. ~1500 words. Written once, edited for years, never rushed.

This page is the philosophical anchor of the brand. It's linked from the footer under "Why StubHouse," and the homepage Section 10 quotes a single line from it as a pull quote.

---

## 7. Navigation & Footer

### Top Navigation

A **single horizontal bar**, 64px tall, fixed to top, with `backdrop-filter: blur(12px)` and a `--border-subtle` bottom border that appears only after scroll.

```
[Mark + Wordmark]    Mocks    Pricing    Docs    Changelog    Blog    [GitHub ★]    [Theme]    [↓ Download]
```

- The mark is 24px and links to `/`
- Center links are `--text-secondary` until hover, then `--text-primary`
- GitHub button shows star count (build-time fetch)
- Theme is a single icon button
- Download is a primary button, always visible

#### Mobile Navigation

Below 768px, center links collapse into a hamburger menu that slides down from the top (not a full-screen overlay). The Download button stays visible. The mark stays left-aligned.

### Footer

Five columns on desktop, stacked on mobile:

```
[Column 1: Mark]              [Column 2: Product]    [Column 3: Resources]    [Column 4: Company]    [Column 5: Legal]

StubHouse                      Mocks                  Docs                     Manifesto              Privacy
A local-first API client       Pricing                GitHub                   Blog                   Terms
and mock server.               Download               Changelog                Press                  License
                               Roadmap                Discord                  RSS

[bottom row, full width, --border-subtle top]
© 2026 StubHouse · Apache 2.0    Built in Rust + Svelte    [GitHub] [X] [Discord]
```

The footer mark is `40px`, monochrome, links to `/`. The bottom row uses caption-size text in `--text-tertiary`.

---

## 8. Animations & Interactions

### Page-Load Sequence

1. `0ms` — HTML/CSS render, theme applied (no flicker)
2. `0ms` — Above-the-fold content visible
3. `100ms` — Logo and headline fade-up complete
4. `200ms` — Subhead fades in
5. `300ms` — CTAs fade in with 60ms stagger
6. `500ms` — Hero video poster swaps to first frame of video; video begins muted autoplay if `prefers-reduced-motion` is not set

If `prefers-reduced-motion: reduce`, all entrance animations are reduced to a single 100ms opacity transition with no translation.

### Scroll Behavior

- A **single line at the top of the page** indicates scroll progress. 1px tall, `--text-primary` filling left-to-right. No other scroll-position indicators.
- Sections enter via the fade-up primitive as described in § 2.5
- The hero video pauses when scrolled out of viewport (saves CPU)

### Interactive Demos

Two interactive elements on the home page:

1. **Scenario switcher** (Section 4A) — buttons swap a JSON payload. State managed in vanilla JS or a single small Svelte/React component.
2. **Comparison filters** (Section 8) — clicking a column header sorts/filters the table.

Both are progressive enhancements — the page works fully without JavaScript, with a static initial state shown.

### Easter Eggs

One easter egg, well-hidden: typing `↑↑↓↓←→←→` on the home page replaces the hero capture with a 4-second loop of someone closing Postman and opening StubHouse. Counts as a small joke for the audience that finds it; invisible to everyone else.

---

## 9. Responsive Behavior

### Breakpoints

```
sm:  640px    — small phones
md:  768px    — large phones / small tablets
lg:  1024px   — tablets / small laptops
xl:  1280px   — laptops
2xl: 1536px   — desktops
```

### Layout Behavior

- **<640px** — Single column. Hero collapses to ~70vh. Display-1 reduces to 56px. Side-by-side feature sections stack vertically (image first, copy second). Comparison table becomes horizontally scrollable.
- **640–1023px** — Single column with wider gutters. Display-1 reduces to 64px. Sub-pillars in Mock Server section stack but retain their visual treatment.
- **≥1024px** — Full design as specified. Side-by-side sections render properly. Cursor parallax enables.

### Container Widths

- Page max-width: **1280px**
- Prose max-width: **65ch** (~720px)
- Hero max-width: **1080px**

### Touch Targets

All interactive elements are minimum **44×44px** on touch devices. Links inside paragraphs receive an extra invisible padding ring on touch.

---

## 10. SEO, Metadata, Social

### Page-Level Metadata

Every page has explicit metadata:

```html
<title>{{page.title}} — StubHouse</title>
<meta name="description" content="{{page.description}}">

<!-- Open Graph -->
<meta property="og:title" content="{{page.title}} — StubHouse">
<meta property="og:description" content="{{page.description}}">
<meta property="og:image" content="https://stubhouse.dev/og/{{page.slug}}.png">
<meta property="og:url" content="https://stubhouse.dev{{page.path}}">
<meta property="og:type" content="website">

<!-- Twitter / X -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:site" content="@stubhouse">
```

### OG Image Generation

OG images are generated at build time per-page using a single template:

- `1200×630px`
- `--bg-canvas` background
- The mark, top-left, 64px
- Page title, display-2, top-center
- A single line of caption at bottom in mono
- An always-on `--border-subtle` outline 24px from the edges

Rendered via Satori or @vercel/og at build, cached, served from CDN.

### Structured Data

The homepage carries `SoftwareApplication` JSON-LD with:

- name, description, applicationCategory: "DeveloperApplication"
- operatingSystem: "macOS, Linux, Windows"
- offers: free
- softwareVersion: current release
- downloadUrl

Blog posts carry `BlogPosting` JSON-LD.

### sitemap.xml & robots.txt

A standard sitemap including all public routes (excluding `/legal`, `/press` — those are explicitly listed but lower priority). `robots.txt` allows all user agents on all paths except `/api/*`.

### Canonical URLs

Every page declares its canonical URL. The marketing site is served from `stubhouse.dev` with no `www.` redirect — `www.stubhouse.dev` 301s to the apex.

---

## 11. Performance Budget

These are hard limits enforced in CI. PRs that exceed them are blocked.

| Metric | Budget |
|---|---|
| First Contentful Paint (FCP) | ≤ 1.0s |
| Largest Contentful Paint (LCP) | ≤ 1.8s |
| Cumulative Layout Shift (CLS) | ≤ 0.05 |
| Time to Interactive (TTI) | ≤ 2.0s |
| Total transferred bytes (home page) | ≤ 500 KB |
| Hero video size | ≤ 2 MB |
| Total JS shipped (home page) | ≤ 50 KB gzipped |
| Total CSS shipped (home page) | ≤ 20 KB gzipped |
| Web font payload (combined) | ≤ 60 KB gzipped per face, max 4 faces |
| Lighthouse score (Performance) | ≥ 95 |
| Lighthouse score (Accessibility) | = 100 |
| Lighthouse score (Best Practices) | = 100 |
| Lighthouse score (SEO) | = 100 |

### Tactics

- All images use modern formats (`avif` with `webp` fallback) via `<picture>`
- The hero video is encoded in AV1 and H.265, served via `<video>` with `<source>` priority on AV1
- Fonts are subset to Latin + extended Latin only at build, served as `woff2`, declared with `font-display: swap`
- All third-party scripts are zero. No analytics, no chat, no ads. (See § 12 for the analytics decision.)
- HTML is minified, CSS is purged to only what's used, JS is split per-page

### Analytics

A single, privacy-respecting analytics service (Plausible, self-hosted) records page views and outbound link clicks. **No cookies, no fingerprinting, no cross-site tracking, no GDPR banner needed.** A single sentence in the footer: *"We use Plausible Analytics. No cookies, no personal data."* — linked to a one-paragraph explanation.

---

## 12. Accessibility

The site targets **WCAG 2.2 AA** at minimum, AAA where practical.

#### Hard Requirements

- All interactive elements reachable via keyboard with visible focus rings (2px solid `--text-primary`, 2px offset)
- All images have meaningful `alt` text; decorative images use `alt=""`
- All form inputs have associated labels
- All headings form a logical outline (one h1 per page, no skipped levels)
- Color contrast ratios: ≥ 7:1 for body text in both themes (AAA), ≥ 4.5:1 for all other text (AA)
- Theme toggle announces state to screen readers via `aria-label` updated on toggle
- Theme preference respects `prefers-color-scheme` and never overrides user OS preference without explicit interaction
- All animations respect `prefers-reduced-motion`
- The hero video has a still poster as its first frame, autoplay is muted, and a "Pause" button appears on hover/focus
- The site is fully usable with JavaScript disabled (interactive demos degrade to static state)
- The site renders correctly in screen readers (tested in VoiceOver, NVDA)

#### Skip Links

A keyboard-only "Skip to content" link is the first focusable element on every page.

---

## 13. Tech Stack

### Recommended Stack

| Layer | Choice | Rationale |
|---|---|---|
| Framework | **SvelteKit 2** | Matches the desktop app's Svelte 5 — single mental model. SSG-first, fast. |
| Styling | **Tailwind CSS 4** + custom CSS variables | Tokens defined once; utility classes for layout. |
| Content | **Markdoc** or MDX | Blog and changelog in markdown with custom components. |
| Code highlighting | **Shiki** with a custom monochrome theme | Builds tokens at compile time; zero runtime JS for syntax highlighting. |
| Hosting | **Cloudflare Pages** or **Vercel** | Edge-cached static, zero cold starts. |
| OG image generation | **Satori** | Build-time JSX → PNG. |
| Analytics | **Plausible** (self-hosted) | Privacy-respecting, single script. |
| Forms (early-access signup) | **Cloudflare Workers** + **D1** or simple **Loops.so** integration | No bloated CRM. |

### Build & Deploy

- Static site generation at build time
- Per-page code splitting
- Asset hashing for cache invalidation
- A `pre-deploy` step that runs Lighthouse CI against the perf budget; fails the build if any metric is exceeded
- Preview deployments on every PR

### Repository Layout

```
stubhouse-site/
├── src/
│   ├── routes/
│   │   ├── +page.svelte           # home
│   │   ├── mocks/+page.svelte
│   │   ├── pricing/+page.svelte
│   │   ├── download/+page.svelte
│   │   ├── changelog/+page.svelte
│   │   ├── blog/
│   │   │   ├── +page.svelte       # index
│   │   │   └── [slug]/+page.svelte
│   │   └── manifesto/+page.svelte
│   ├── lib/
│   │   ├── components/            # the 12 primitives
│   │   ├── tokens.css             # theme variables
│   │   └── motion.ts              # the 3 motion primitives
│   ├── content/
│   │   ├── blog/*.md
│   │   └── changelog/*.md
│   └── app.css
├── static/
│   ├── fonts/
│   ├── og-templates/
│   └── product-captures/
└── tests/
    └── lighthouse.config.ts
```

---

## 14. Asset Checklist

Every asset must exist before launch. A missing asset is a launch blocker.

#### Logo & Marks

- [ ] `mark.svg` — single-file, currentColor
- [ ] `mark-dark.png` — 1024px, white on transparent
- [ ] `mark-light.png` — 1024px, black on transparent
- [ ] `wordmark.svg`
- [ ] `lockup-horizontal.svg`
- [ ] `lockup-stacked.svg`
- [ ] `favicon.ico` — 32px, 16px
- [ ] `apple-touch-icon.png` — 180px
- [ ] `safari-pinned-tab.svg`
- [ ] Press kit ZIP — all of the above + brand guidelines PDF

#### Product Captures

- [ ] Hero loop video — dark theme, 12s, AV1 + H.265, ≤2MB
- [ ] Hero loop video — light theme, 12s, AV1 + H.265, ≤2MB
- [ ] Static screenshots for each home-page section (dark + light) — 18 total
- [ ] Mock server page diagram captures (dark + light) — 12 total
- [ ] Comparison table illustrative captures (3 small UI snippets, dark + light)
- [ ] Terminal pane captures with real CLI output

#### OG Images

- [ ] `og/home.png` (1200×630)
- [ ] `og/mocks.png`
- [ ] `og/pricing.png`
- [ ] `og/download.png`
- [ ] `og/changelog.png`
- [ ] `og/blog.png` — generic for blog index
- [ ] `og/manifesto.png`
- [ ] Per-blog-post OG images generated at build

#### Fonts

- [ ] `Geist-Regular.woff2`
- [ ] `Geist-Medium.woff2`
- [ ] `GeistMono-Regular.woff2`
- [ ] `GeistMono-Medium.woff2`
- [ ] `Newsreader-Regular.woff2` (manifesto + essay blog posts only)
- [ ] All subset to Latin + Latin Extended

#### Copy

- [ ] Home page sections 1–11 finalized
- [ ] Mock server page (~3000 words) finalized
- [ ] Pricing copy
- [ ] Download platform descriptions
- [ ] Manifesto (~1500 words) — single-author, multiple revisions
- [ ] First three blog posts (drafts at minimum)
- [ ] Privacy policy
- [ ] Terms of use

---

## 15. Launch Checklist

A staged checklist. Each stage gates the next.

#### Stage 1 — Foundation (week 1–2)

- [ ] Repository created, build pipeline green
- [ ] Theme system implemented with both light/dark variants verified
- [ ] All 12 primitive components built and documented in a private `/components` route
- [ ] Token system finalized; no hardcoded colors anywhere

#### Stage 2 — Pages (week 3–6)

- [ ] Home page complete, all 11 sections built with real content
- [ ] Mock server page complete
- [ ] Pricing, Download, Changelog, Blog index pages complete
- [ ] Manifesto written and laid out
- [ ] At least one blog post live as proof of layout

#### Stage 3 — Polish (week 7)

- [ ] All product captures shot in both themes
- [ ] Hero loop video shot, edited, encoded
- [ ] OG images generated for all routes
- [ ] Comparison table data sourced and verified against current competitor versions
- [ ] All copy proofread end-to-end by someone who didn't write it

#### Stage 4 — Performance & Accessibility (week 8)

- [ ] Lighthouse CI passing on all routes
- [ ] Manual screen reader test on home + mock server + manifesto
- [ ] Keyboard-only navigation test
- [ ] `prefers-reduced-motion` test
- [ ] All performance budgets met

#### Stage 5 — Pre-launch (week 9)

- [ ] DNS configured: apex `stubhouse.dev`, `www.` 301s, `docs.` separate
- [ ] SSL certificate issued (Cloudflare or Let's Encrypt)
- [ ] Analytics verified, single page view registers
- [ ] Email signup form tested end-to-end
- [ ] All external links verified (no 404s)
- [ ] Release notes for v1.0 of the desktop app drafted; download buttons point to real release artifacts
- [ ] HN / Reddit / X launch posts drafted

#### Stage 6 — Launch

- [ ] Site goes live at `stubhouse.dev`
- [ ] HN: "Show HN: StubHouse — local-first API client + mock server, no Electron"
- [ ] X: launch thread
- [ ] Tag stable v1.0 on the desktop app repo
- [ ] Press kit URL shared with relevant publications

---

## Appendix A — Visual References

The following sites are explicit references. Studying them is part of building this one.

| Site | What to study |
|---|---|
| **zed.dev** | Hero treatment, product capture quality, monochrome restraint |
| **linear.app** | Section pacing, comparison treatment, footer composition |
| **vercel.com** | Type discipline, code block treatment, dark/light toggle |
| **ghostty.org** | Monochrome confidence, single-color brand discipline |
| **stripe.com/press** | Editorial typography, manifesto-style writing |
| **anthropic.com** | Voice, restraint, generous whitespace |
| **railway.com** | Dark-default polish, animation discipline |

Do not copy any of them. Do internalize what they get right.

---

## Appendix B — Open Questions

Decisions deferred until implementation begins:

1. **Subdomain vs subfolder for docs.** `docs.stubhouse.dev` (clean separation, easier to swap engine) vs `stubhouse.dev/docs` (better SEO consolidation). Lean: subdomain. Confirm during § 6.5 build.
2. **Should the home page hero capture include audio?** No is the safer default. Reconsider only if the silent loop tests poorly.
3. **Newsletter or no newsletter.** A newsletter implies an obligation to ship one. Defer to post-launch.
4. **Comments on blog posts.** No, ever. Discussion happens on HN, in Discord, on X.
5. **Versioned URLs for the changelog.** `/changelog/v1.2.0` permalinks would be useful. Implement at v1.0.
6. **Localization.** English only at launch. Reconsider when ≥10% of organic traffic is from non-English-speaking regions.

---

*Marketing spec version 1.0.0-draft. Designed to ship `stubhouse.dev` v1.0 alongside StubHouse desktop v1.0. Subject to revision during build.*
