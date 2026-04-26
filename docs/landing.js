/* ───── rules data ───── */
const RULES = [
  // Security
  { id: "SEC001", cat: "Security",     name: "No innerHTML usage",                       sev: "error",   acc: "exact" },
  { id: "SEC002", cat: "Security",     name: "No outerHTML usage",                       sev: "error",   acc: "exact" },
  { id: "SEC003", cat: "Security",     name: "No insertAdjacentHTML usage",              sev: "error",   acc: "exact" },
  { id: "SEC004", cat: "Security",     name: "No global window.app access",              sev: "warning", acc: "exact" },
  // Resources
  { id: "RES001", cat: "Resources",    name: "Plugin must implement onunload",           sev: "error",   acc: "exact" },
  { id: "RES002", cat: "Resources",    name: "No leaf detachment in onunload",           sev: "warning", acc: "exact" },
  { id: "API005", cat: "Resources",    name: "Don't pass plugin as MarkdownRenderer component", sev: "error", acc: "exact" },
  // Vault
  { id: "VAULT001", cat: "Vault",      name: "Prefer vault.process over vault.modify",   sev: "warning", acc: "exact" },
  { id: "VAULT002", cat: "Vault",      name: "Filter vault.getFiles results",            sev: "warning", acc: "exact" },
  { id: "VAULT003", cat: "Vault",      name: "Use normalizePath for user paths",         sev: "info",    acc: "exact" },
  { id: "API001",   cat: "Vault",      name: "Prefer Vault API over Adapter API",        sev: "warning", acc: "exact" },
  { id: "API002",   cat: "Vault",      name: "Prefer FileManager.trashFile",             sev: "warning", acc: "exact" },
  // Workspace
  { id: "WORK001",  cat: "Workspace",  name: "Avoid workspace.activeLeaf",               sev: "warning", acc: "exact" },
  { id: "WORK002",  cat: "Workspace",  name: "No stored references to custom views",     sev: "warning", acc: "exact" },
  // Commands
  { id: "CMD001", cat: "Commands",     name: "No default hotkey",                        sev: "info",    acc: "exact" },
  { id: "CMD002", cat: "Commands",     name: "Use appropriate callback type",            sev: "warning", acc: "exact" },
  { id: "CMD003", cat: "Commands",     name: "Command ID should not include plugin ID",  sev: "info",    acc: "exact" },
  { id: "CMD004", cat: "Commands",     name: "No \"command\" word in command ID",        sev: "info",    acc: "exact" },
  { id: "CMD005", cat: "Commands",     name: "No \"command\" word in command name",      sev: "info",    acc: "exact" },
  { id: "CMD006", cat: "Commands",     name: "No plugin name in command name",           sev: "info",    acc: "exact" },
  // UI
  { id: "UI001", cat: "UI",            name: "Use setHeading instead of h1/h2",          sev: "warning", acc: "exact" },
  { id: "UI002", cat: "UI",            name: "No \"settings\" in settings headings",     sev: "info",    acc: "exact" },
  { id: "UI003", cat: "UI",            name: "Use sentence case in UI text",             sev: "info",    acc: "approximate" },
  { id: "UI004", cat: "UI",            name: "No top-level heading in settings",         sev: "warning", acc: "approximate" },
  { id: "UI005", cat: "UI",            name: "No manual HTML headings (h3–h6) in settings", sev: "warning", acc: "exact" },
  { id: "API006", cat: "UI",           name: "Prefer AbstractInputSuggest",              sev: "warning", acc: "exact" },
  { id: "API007", cat: "UI",           name: "Use getLanguage() for locale",             sev: "warning", acc: "exact" },
  // Styling
  { id: "STYLE001", cat: "Styling",    name: "No hardcoded inline styles",               sev: "warning", acc: "exact" },
  // TypeScript
  { id: "TS001",  cat: "TypeScript",   name: "Prefer const/let over var",                sev: "info",    acc: "exact" },
  { id: "TS002",  cat: "TypeScript",   name: "Prefer async/await over raw Promise",      sev: "info",    acc: "exact" },
  { id: "API003", cat: "TypeScript",   name: "No as TFile / as TFolder type casting",    sev: "warning", acc: "exact" },
  { id: "API004", cat: "TypeScript",   name: "Prefer el.instanceOf() for DOM/UIEvent",   sev: "warning", acc: "exact" },
  // Mobile
  { id: "MOB001", cat: "Mobile",       name: "No regex lookbehind (unsupported on iOS)", sev: "warning", acc: "exact" },
  { id: "MOB002", cat: "Mobile",       name: "Prefer activeWindow.setTimeout",           sev: "warning", acc: "exact" },
  { id: "MOB003", cat: "Mobile",       name: "Prefer activeDocument/activeWindow",       sev: "warning", acc: "exact" },
  { id: "MOB004", cat: "Mobile",       name: "No navigator API for platform detection",  sev: "warning", acc: "exact" },
  // Manifest
  { id: "MAN001", cat: "Manifest",     name: "Required field: id",                       sev: "error",   acc: "exact" },
  { id: "MAN002", cat: "Manifest",     name: "Required field: name",                     sev: "error",   acc: "exact" },
  { id: "MAN003", cat: "Manifest",     name: "Required field: version",                  sev: "error",   acc: "exact" },
  { id: "MAN004", cat: "Manifest",     name: "Required field: minAppVersion",            sev: "error",   acc: "exact" },
  { id: "MAN005", cat: "Manifest",     name: "Required field: description",              sev: "error",   acc: "exact" },
  { id: "MAN006", cat: "Manifest",     name: "Required field: author",                   sev: "error",   acc: "exact" },
  { id: "MAN007", cat: "Manifest",     name: "Valid plugin ID format ([a-z0-9-]+)",      sev: "error",   acc: "exact" },
  { id: "MAN008", cat: "Manifest",     name: "Set isDesktopOnly when using Node/Electron APIs", sev: "warning", acc: "approximate" },
  { id: "MAN009", cat: "Manifest",     name: "Description ≤ 250 characters",             sev: "info",    acc: "exact" },
  { id: "MAN010", cat: "Manifest",     name: "Description should end with a period",     sev: "info",    acc: "exact" },
  { id: "MAN011", cat: "Manifest",     name: "No unchanged sample plugin ID",            sev: "error",   acc: "exact" },
  { id: "MAN012", cat: "Manifest",     name: "LICENSE file should contain copyright",    sev: "warning", acc: "approximate" },
  // General
  { id: "GEN001", cat: "General",      name: "No console.log in production",             sev: "warning", acc: "exact" },
  { id: "GEN002", cat: "General",      name: "Rename placeholder class names",           sev: "error",   acc: "exact" },
  { id: "GEN003", cat: "General",      name: "Avoid bare global app variable",           sev: "warning", acc: "approximate" },
];

/* ───── render rule tabs + table ───── */
(() => {
  const tabsEl = document.getElementById('rules-tabs');
  const tableEl = document.getElementById('rules-table');
  const cats = ['All', ...Array.from(new Set(RULES.map(r => r.cat)))];
  let active = 'All';

  function counts(c) {
    return c === 'All' ? RULES.length : RULES.filter(r => r.cat === c).length;
  }
  function renderTabs() {
    tabsEl.innerHTML = cats.map(c => `
      <button class="rules-tab ${c === active ? 'on' : ''}" data-cat="${c}">
        ${c} <span class="ct">${counts(c)}</span>
      </button>
    `).join('');
    tabsEl.querySelectorAll('.rules-tab').forEach(b => {
      b.addEventListener('click', () => { active = b.dataset.cat; renderTabs(); renderTable(); });
    });
  }
  function renderTable() {
    const rows = active === 'All' ? RULES : RULES.filter(r => r.cat === active);
    tableEl.innerHTML = `
      <thead>
        <tr>
          <th style="width:90px">ID</th>
          <th>Rule</th>
          <th style="width:120px">Category</th>
          <th style="width:110px">Severity</th>
          <th style="width:110px">Accuracy</th>
        </tr>
      </thead>
      <tbody>
        ${rows.map(r => `
          <tr>
            <td><span class="rule-id rule-id-${r.sev}">${r.id}</span></td>
            <td>${r.name}</td>
            <td style="color:var(--fg-2);font-size:12.5px">${r.cat}</td>
            <td><span class="sev-pill ${r.sev}"><span class="d"></span>${r.sev}</span></td>
            <td><span class="tag ${r.acc === 'exact' ? 'exact' : r.acc === 'heuristic' ? 'heur' : 'approx'}">${r.acc}</span></td>
          </tr>
        `).join('')}
      </tbody>
    `;
  }
  renderTabs(); renderTable();
})();

/* ───── animated counters ───── */
(() => {
  const stats = document.querySelectorAll('[data-count]');
  if (!stats.length || !('IntersectionObserver' in window)) {
    stats.forEach(el => { el.textContent = el.dataset.count + (el.dataset.suffix || ''); });
    return;
  }
  const io = new IntersectionObserver((entries) => {
    entries.forEach(e => {
      if (!e.isIntersecting) return;
      const el = e.target;
      const target = +el.dataset.count;
      const suffix = el.dataset.suffix || '';
      const dur = 1100;
      const t0 = performance.now();
      function tick(t) {
        const p = Math.min(1, (t - t0) / dur);
        const eased = 1 - Math.pow(1 - p, 3);
        el.textContent = Math.round(target * eased) + suffix;
        if (p < 1) requestAnimationFrame(tick);
      }
      requestAnimationFrame(tick);
      io.unobserve(el);
    });
  }, { threshold: 0.4 });
  stats.forEach(s => io.observe(s));
})();

/* ───── section reveal on scroll ───── */
(() => {
  const sections = document.querySelectorAll('.section');
  if (!('IntersectionObserver' in window)) {
    sections.forEach(s => s.classList.add('in'));
    return;
  }
  const io = new IntersectionObserver((entries) => {
    entries.forEach(e => { if (e.isIntersecting) { e.target.classList.add('in'); io.unobserve(e.target); } });
  }, { threshold: 0.08 });
  sections.forEach(s => io.observe(s));
})();

/* ───── terminal animation ───── */
(() => {
  const body = document.getElementById('term-body');
  const term = document.getElementById('terminal');
  const card = document.getElementById('report-card');
  if (!body) return;

  const STEPS = [
    { t: 250, html: `<span class="ln"><span class="prompt">$</span> <span class="cmd">oplint lint . -f html</span></span>` },
    { t: 700, html: `<span class="ln muted">  scanning my-plugin/</span>` },
    { t: 1000, html: `<span class="ln muted">  parsing 42 files via tree-sitter…</span>` },
    { t: 1450, html: `<span class="ln"><span class="err">✗</span> SEC001 src/views/SidebarView.ts:84  No innerHTML usage</span>` },
    { t: 1700, html: `<span class="ln"><span class="err">✗</span> RES001 src/main.ts:14  Plugin must implement onunload</span>` },
    { t: 1900, html: `<span class="ln"><span class="warn">⚠</span> MOB001 src/utils/parse.ts:22  No regex lookbehind</span>` },
    { t: 2100, html: `<span class="ln"><span class="warn">⚠</span> STYLE001 src/views/PaletteView.ts:48  No hardcoded styles</span>` },
    { t: 2300, html: `<span class="ln"><span class="info">ℹ</span> CMD001 src/main.ts:95  No default hotkey</span>` },
    { t: 2500, html: `<span class="ln muted">  …14 more findings</span>` },
    { t: 2900, html: `<span class="ln"></span><span class="ln"><span class="ok">✓</span> 68 / 88 checks passing — score <b>78</b> · grade <b>B+</b></span>` },
    { t: 3200, html: `<span class="ln muted">  ⏱  finished in 184ms · wrote report.html</span>` },
    { t: 3600, html: `<span class="ln"><span class="prompt">$</span> <span class="cursor"></span></span>` },
  ];

  let started = false;
  function start() {
    if (started) return;
    started = true;
    body.innerHTML = '';
    STEPS.forEach(s => setTimeout(() => { body.insertAdjacentHTML('beforeend', s.html); body.scrollTop = body.scrollHeight; }, s.t));
    setTimeout(() => { card.classList.add('show'); term.classList.add('shrunk'); }, 3900);
    // loop
    setTimeout(() => {
      card.style.transition = 'opacity .25s ease-in, transform .35s cubic-bezier(.4,0,.6,1)';
      card.classList.remove('show');
      term.classList.remove('shrunk');
      setTimeout(() => { card.style.transition = ''; }, 400);
      started = false;
      setTimeout(start, 1000);
    }, 9500);
  }

  const io = new IntersectionObserver((entries) => {
    entries.forEach(e => { if (e.isIntersecting) start(); });
  }, { threshold: 0.3 });
  io.observe(term);
})();

/* ───── copy buttons ───── */
document.querySelectorAll('.copy-btn').forEach(b => {
  const origHTML = b.innerHTML;
  b.addEventListener('click', () => {
    navigator.clipboard?.writeText(b.dataset.copy);
    b.innerHTML = 'Copied';
    b.classList.add('copied');
    setTimeout(() => { b.innerHTML = origHTML; b.classList.remove('copied'); }, 1400);
  });
});

/* ───── h1 cycling phrases ───── */
(() => {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  const cycle = document.querySelector('.h1-cycle');
  if (!cycle) return;
  const phrases = Array.from(cycle.querySelectorAll('.h1-phrase'));
  let idx = 0;
  setTimeout(() => {
    setInterval(() => {
      const prev = phrases[idx];
      prev.classList.remove('is-active');
      prev.classList.add('is-exiting');
      setTimeout(() => prev.classList.remove('is-exiting'), 400);
      idx = (idx + 1) % phrases.length;
      phrases[idx].classList.add('is-active');
    }, 3400);
  }, 2800);
})();

/* ───── faq toggle animation ───── */
(() => {
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  document.querySelectorAll('.faq details').forEach(el => {
    const summary = el.querySelector('summary');
    const body = el.querySelector('p');
    if (!summary || !body) return;
    summary.addEventListener('click', e => {
      e.preventDefault();
      if (el.open) {
        // close
        el.dataset.closing = '';
        setTimeout(() => {
          delete el.dataset.closing;
          el.removeAttribute('open');
        }, reduced ? 0 : 300);
      } else {
        // open: set [open] then pin body at 0 so CSS transition has a start point
        el.setAttribute('open', '');
        body.style.maxHeight = '0px';
        body.style.opacity = '0';
        body.style.margin = '0';
        void body.offsetHeight; // force reflow — browser paints at 0 before transition
        body.style.maxHeight = '';
        body.style.opacity = '';
        body.style.margin = '';
      }
    });
  });
})();

/* ───── scroll spy: highlight active nav link ───── */
(() => {
  const links = document.querySelectorAll('.nav-links a');
  const map = {};
  links.forEach(a => {
    const id = a.getAttribute('href')?.slice(1);
    if (id) map[id] = a;
  });
  const sections = Object.keys(map).map(id => {
    const el = document.getElementById(id);
    return el ? { el, a: map[id] } : null;
  }).filter(Boolean);

  if (!sections.length) return;

  const observer = new IntersectionObserver(entries => {
    entries.forEach(e => {
      if (e.isIntersecting) {
        links.forEach(l => l.classList.remove('active'));
        const match = sections.find(s => s.el === e.target);
        if (match) match.a.classList.add('active');
      }
    });
  }, { rootMargin: '-20% 0px -70% 0px', threshold: 0 });

  sections.forEach(s => observer.observe(s.el));
})();
