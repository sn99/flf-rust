/** PeerJS lockstep glue approximating F.LF core/network application layer. */
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
      var s = typeof d === 'string' ? d : JSON.stringify(d);
      w.__flf_peer_inbox.push(s);
    });
  }
  w.__flf_peer_send = function (payload) {
    if (w.__flf_conn && w.__flf_conn.open) {
      try {
        w.__flf_conn.send(payload);
      } catch (e) {}
    }
  };
})(window);
