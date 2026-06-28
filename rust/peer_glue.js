/** PeerJS lockstep glue approximating F.LF core/network application layer.
 * Prefer network_core.js + flobby.js for full F.Lobby 0.1; this remains a thin fallback.
 */
(function (w) {
  w.__flf_peer_inbox = w.__flf_peer_inbox || [];
  var hostId = null;
  w.__flf_peer_connect = function (server, role, room) {
    room = (room || 'default').replace(/[^a-zA-Z0-9_-]/g, '');
    hostId = 'flf-' + room;
    console.log('[flf peer] connect', { server: server, role: role, room: room, hostId: hostId });
    if (!w.Peer) {
      console.warn('[flf peer] Load peerjs.min.js for WebRTC; BroadcastChannel still active.');
      return;
    }
    // If network_core already owns Peer, skip double-connect
    if (w.__flf_peer && w.__flf_network_config) {
      console.log('[flf peer] network_core owns peer; glue skip create');
      return;
    }
    try {
      var isHost = role !== 'passive' && role !== 'remote';
      var peer = new w.Peer(isHost ? hostId : undefined);
      w.__flf_peer = peer;
      peer.on('open', function (id) {
        console.log('[flf peer] open', id);
        if (!isHost) {
          var conn = peer.connect(hostId, { reliable: true });
          wire(conn);
        }
      });
      peer.on('connection', function (conn) {
        wire(conn);
      });
      peer.on('error', function (e) {
        console.warn('[flf peer] error', e);
      });
    } catch (e) {
      console.warn(e);
    }
  };
  function wire(conn) {
    w.__flf_conn = conn;
    conn.on('open', function () {
      console.log('[flf peer] connection open');
    });
    conn.on('data', function (d) {
      // Feed core network if present
      if (w.__flf_network && w.__flf_network.onTransportData) {
        try {
          w.__flf_network.onTransportData(typeof d === 'string' ? JSON.parse(d) : d);
        } catch (e) {}
      }
      var s = typeof d === 'string' ? d : JSON.stringify(d);
      w.__flf_peer_inbox.push(s);
      // Also legacy Rust inbox
      if (w.__flf_net_inbox && w.__flf_net_inbox.push) {
        w.__flf_net_inbox.push(s);
      } else if (w.__flf_net_inbox && typeof w.__flf_net_inbox === 'object') {
        try {
          // may be JS Array from wasm Reflect
        } catch (e) {}
      }
    });
  }
  w.__flf_peer_send = function (payload) {
    if (w.__flf_conn && w.__flf_conn.open) {
      try {
        if (typeof payload === 'string') {
          try { w.__flf_conn.send(JSON.parse(payload)); } catch (e) { w.__flf_conn.send(payload); }
        } else {
          w.__flf_conn.send(payload);
        }
      } catch (e) {}
    }
  };
})(window);
