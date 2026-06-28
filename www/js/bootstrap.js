// F.LF Rust bootstrap — assets from separate LF2_19 repo (Project F style)
import init, { start_game, version } from '../pkg/flf.js?v=20260628c';

const loading = document.getElementById('loading');

/** Default remote package (same layout as Project-F/LF2_19, JSON on Pages) */
const REMOTE_PACKAGE = 'https://sn99.github.io/LF2_19';

function localPackage() {
  try {
    return new URL('../assets/LF2_19', import.meta.url).href.replace(/\/$/, '');
  } catch {
    return 'assets/LF2_19';
  }
}

function configPackage() {
  const el = document.getElementById('flf-config');
  if (el) {
    try {
      const cfg = JSON.parse(el.textContent);
      if (cfg.package) return cfg.package.replace(/\/$/, '');
    } catch (_) {}
  }
  const q = new URLSearchParams(location.search).get('package');
  if (q) return q.replace(/\/$/, '');
  return null;
}

async function probe(url) {
  try {
    const r = await fetch(url + '/manifest.json', { method: 'GET', cache: 'no-cache' });
    return r.ok;
  } catch {
    return false;
  }
}

async function resolvePackage() {
  const configured = configPackage();
  if (configured && await probe(configured)) return configured;
  // Prefer remote LF2_19 (small engine deploy)
  if (await probe(REMOTE_PACKAGE)) return REMOTE_PACKAGE;
  const local = localPackage();
  if (await probe(local)) return local;
  // last resort remote even if probe failed (CORS edge)
  return configured || REMOTE_PACKAGE;
}

try {
  await init();
  console.log('F.LF Rust', version());
  const root = await resolvePackage();
  console.log('asset package', root);
  if (loading) loading.textContent = 'Loading package… ' + root;
  await start_game(root);
  if (loading) loading.remove();
} catch (e) {
  console.error(e);
  if (loading) {
    loading.innerHTML = 'Failed to load: ' + e +
      '<br><small>Try <a href="?package=https://sn99.github.io/LF2_19">remote LF2_19</a> or local assets.</small>';
  }
}
