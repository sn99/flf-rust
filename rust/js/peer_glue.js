/** Optional PeerJS lockstep glue for flf-rust (loads peerjs from CDN if present). */
(function (w) {
  w.__flf_peer_inbox = w.__flf_peer_inbox || [];
  w.__flf_peer_connect = function (server, role, room) {
    console.log('[flf peer] connect', server, role, room);
    // Full PeerJS requires peerjs library; document for integrators
    if (!w.Peer) {
      console.warn('[flf peer] PeerJS not loaded — using BroadcastChannel only. Add <script src="https://unpkg.com/peerjs@1.5.4/dist/peerjs.min.js">');
      return;
    }
    try {
      var peer = new w.Peer(role === 'passive' ? undefined : 'flf-' + room);
      w.__flf_peer = peer;
      peer.on('open', function (id) {
        console.log('[flf peer] id', id);
        if (role === 'passive' || role === 'remote') {
          var conn = peer.connect('flf-' + room);
          w.__flf_conn = conn;
          conn.on('data', function (d) {
            w.__flf_peer_inbox.push(typeof d === 'string' ? d : JSON.stringify(d));
          });
        } else {
          peer.on('connection', function (conn) {
            w.__flf_conn = conn;
            conn.on('data', function (d) {
              w.__flf_peer_inbox.push(typeof d === 'string' ? d : JSON.stringify(d));
            });
          });
        }
      });
    } catch (e) {
      console.warn(e);
    }
  };
  w.__flf_peer_send = function (payload) {
    if (w.__flf_conn && w.__flf_conn.open) {
      try { w.__flf_conn.send(payload); } catch (e) {}
    }
  };
})(window);
