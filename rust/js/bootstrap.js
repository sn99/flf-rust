import init, { start_game, version } from '../pkg/flf.js?v=20260629d';
// AI bridge loaded via script tag; ensure global exists

const loading = document.getElementById('loading');
const REMOTE = 'https://sn99.github.io/LF2_19';

function cfg() {
  const el = document.getElementById('flf-config');
  if (!el) return {};
  try { return JSON.parse(el.textContent); } catch { return {}; }
}

function absolutize(pkg) {
  if (!pkg) return null;
  if (pkg.startsWith('http')) return pkg.replace(/\/$/, '');
  // relative to page
  return new URL(pkg, location.href).href.replace(/\/$/, '');
}

async function probe(url) {
  try {
    const r = await fetch(url + '/manifest.json', { cache: 'no-cache', mode: 'cors' });
    return r.ok;
  } catch (e) {
    console.warn('probe fail', url, e);
    return false;
  }
}

async function resolvePackage() {
  const c = cfg();
  const q = new URLSearchParams(location.search).get('package');
  const candidates = [
    q,
    c.package,
    // same-origin assets (bundled on engine Pages)
    absolutize('assets/LF2_19'),
    c.remote || REMOTE,
    REMOTE,
  ].filter(Boolean).map(p => p.startsWith('http') ? p.replace(/\/$/, '') : absolutize(p));

  const seen = new Set();
  for (const u of candidates) {
    if (!u || seen.has(u)) continue;
    seen.add(u);
    console.log('trying package', u);
    if (await probe(u)) return u;
  }
  return candidates[0] || REMOTE;
}

try {
  await init();
  console.log('F.LF Rust', version());
  const root = await resolvePackage();
  console.log('using package', root);
  if (loading) loading.textContent = 'Loading ' + root + ' …';
  if (typeof window.__flf_ai_preload === 'function') {
    window.__flf_ai_preload(root, ['dumbass', 'Challangar', 'Crusher', 'Ninja']);
  }
  await start_game(root);
  if (loading) loading.remove();
} catch (e) {
  console.error(e);
  if (loading) loading.textContent = 'Failed: ' + e + ' — try ?package=' + REMOTE;
}
