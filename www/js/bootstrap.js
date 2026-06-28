import init, { start_game, version } from '../pkg/flf.js';

const loading = document.getElementById('loading');

function assetRoot() {
  // www/js/bootstrap.js -> www/assets/LF2_19
  const u = new URL('../assets/LF2_19', import.meta.url);
  // Use path+origin so fetch works; strip trailing slash
  let href = u.href;
  if (href.endsWith('/')) href = href.slice(0, -1);
  return href;
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
