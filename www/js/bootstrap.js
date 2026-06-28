import init, { start_game, version } from '../pkg/flf.js';

const loading = document.getElementById('loading');

function assetRoot() {
  // Resolve LF2_19 relative to the page URL (works on GitHub Pages subpaths)
  const pageDir = location.pathname.endsWith('/')
    ? location.pathname
    : location.pathname.replace(/\/[^/]*$/, '/');
  // Prefer path relative to index.html location
  let root = pageDir + 'assets/LF2_19';
  // When using import.meta (module in js/), go up one
  try {
    root = new URL('../assets/LF2_19', import.meta.url).pathname;
  } catch (_) {}
  // Ensure no trailing slash (Rust adds paths)
  if (root.endsWith('/')) root = root.slice(0, -1);
  // On GitHub Pages, pathname is absolute from domain root — good for fetch
  // If we got a full path from URL API without origin, prefix origin for fetch
  if (root.startsWith('/')) {
    return root;
  }
  return root;
}

try {
  await init();
  console.log('F.LF Rust', version());
  const root = assetRoot();
  console.log('asset root', root);
  await start_game(root);
  if (loading) loading.remove();
} catch (e) {
  console.error(e);
  if (loading) loading.textContent = 'Failed to load: ' + e;
}
