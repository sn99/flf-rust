/**
 * F.Lobby 0.1 client — protocol fetch + lobby iframe + postMessage start → network.setup
 * Mirrors F.LF manager.js network_game + lobby UI_list.
 */
(function (w) {
  function normalizeAddress(str) {
    str = (str || '').trim();
    if (!str) return str;
    if (str.charAt(str.length - 1) === '/') return str.slice(0, -1);
    return str;
  }

  function ensureLobbyIframe() {
    var root = document.querySelector('.LFroot .window') || document.body;
    var lobby = document.querySelector('.lobby_panel');
    if (!lobby) {
      lobby = document.createElement('div');
      lobby.className = 'lobby_panel';
      lobby.style.cssText = 'display:none;position:absolute;inset:0;z-index:50;background:#001a4d;';
      lobby.innerHTML =
        '<div style="display:flex;justify-content:flex-end;padding:4px">' +
        '<button type="button" class="lobby_close_button">Close lobby</button></div>' +
        '<iframe class="lobby_window" style="width:100%;height:calc(100% - 32px);border:0;background:#fff"></iframe>';
      root.appendChild(lobby);
      lobby.querySelector('.lobby_close_button').onclick = function () {
        lobby.style.display = 'none';
        var iframe = lobby.querySelector('.lobby_window');
        iframe.removeAttribute('src');
      };
    }
    return lobby;
  }

  /**
   * Connect to F.Lobby server.
   * @param {string} serverAddress e.g. http://lobby.projectf.hk
   * @param {function} onLog
   * @param {function} onStart function(serverProtocolJson, paramFromLobby)
   * @param {function} onError
   */
  function connect(serverAddress, onLog, onStart, onError) {
    var addr = normalizeAddress(serverAddress);
    onLog = onLog || function () {};
    onError = onError || function (e) { console.warn(e); };
    onLog('F.Lobby: GET ' + addr + '/protocol');
    var request = new XMLHttpRequest();
    request.onreadystatechange = function () {
      if (this.readyState !== 4) return;
      if (this.status === 200) {
        var server;
        try {
          server = JSON.parse(this.responseText);
        } catch (e) {
          onError('Invalid /protocol JSON');
          return;
        }
        // ensure address field
        if (!server.address) server.address = addr;
        onLog('F.Lobby: protocol ok name=' + (server.name || '?'));
        openLobby(server, onLog, onStart, onError);
      } else {
        onError('[' + this.status + '] Failed to connect to F.Lobby at ' + addr);
        // Peer-room fallback: synthesize start with room from address
        var room = addr.indexOf('room-') >= 0 ? addr : ('room-' + Math.floor(Math.random() * 1e6));
        onLog('F.Lobby unavailable — Peer/BC fallback room=' + room);
        var fakeServer = { name: 'local-peer', address: addr, library: '' };
        var param = {
          role: addr.indexOf('passive') >= 0 ? 'passive' : 'active',
          id1: room,
          id2: room + '-p2',
          room: room
        };
        if (onStart) onStart(fakeServer, param);
      }
    };
    request.open('GET', addr + '/protocol', true);
    request.responseType = 'text';
    request.timeout = 2500;
    request.ontimeout = function () {
      onError('F.Lobby /protocol timeout');
      var room = 'room-' + Math.floor(Math.random() * 1e6);
      onLog('timeout — Peer/BC fallback room=' + room);
      if (onStart) {
        onStart(
          { name: 'local-peer', address: addr, library: '' },
          { role: 'active', id1: room, id2: room + '-p2', room: room }
        );
      }
    };
    try {
      request.send();
    } catch (e) {
      onError(String(e));
    }
  }

  function openLobby(server, onLog, onStart, onError) {
    var lobby = ensureLobbyIframe();
    var iframe = lobby.querySelector('.lobby_window');
    lobby.style.display = 'block';
    var origin = server.address;
    function windowMessage(event) {
      if (event.origin !== origin && event.origin + '/' !== origin + '/') {
        // allow same host variations
        try {
          if (new URL(event.origin).host !== new URL(origin).host) return;
        } catch (e) {
          return;
        }
      }
      if (!event.data) return;
      if (event.data.event === 'start') {
        onLog('F.Lobby: start role=' + (event.data.role || event.data.param && event.data.param.role));
        lobby.style.display = 'none';
        window.removeEventListener('message', windowMessage, false);
        var param = event.data.param || event.data;
        if (onStart) onStart(server, param);
      }
    }
    window.addEventListener('message', windowMessage, false);
    iframe.onload = function () {
      try {
        iframe.contentWindow.postMessage(
          { init: true, protocol: 'F.Lobby 0.1', room: 'F.LF' },
          server.address
        );
        onLog('F.Lobby: postMessage init F.Lobby 0.1');
      } catch (e) {
        onError('lobby postMessage failed: ' + e);
      }
    };
    iframe.src = server.address + '/lobby';
    onLog('F.Lobby: iframe ' + iframe.src);
  }

  /** After lobby start: wire core/network setup */
  function startNetwork(server, param, monitor) {
    monitor = monitor || {
      on: function (ev, data) {
        console.log('[flf lobby net]', ev, data);
        if (w.__flf_lobby_log) w.__flf_lobby_log(ev + (data != null ? ': ' + data : ''));
      }
    };
    if (!w.__flf_network) {
      monitor.on('error', 'network_core.js not loaded');
      return;
    }
    w.__flf_network.setup({ server: server, param: param }, monitor);
    w.__flf_lobby_param = param;
    w.__flf_lobby_server = server;
    // notify Rust side
    if (typeof w.__flf_on_lobby_start === 'function') {
      try {
        w.__flf_on_lobby_start(JSON.stringify({ server: server, param: param }));
      } catch (e) {}
    }
  }

  w.__flf_lobby_connect = function (serverAddress) {
    var logs = [];
    function log(m) {
      logs.push(m);
      console.log('[flobby]', m);
      var ta = document.querySelector('.network_log');
      if (ta) ta.value = (ta.value ? ta.value + '\n' : '') + m;
      if (w.__flf_lobby_log) w.__flf_lobby_log(m);
    }
    connect(
      serverAddress,
      log,
      function (server, param) {
        startNetwork(server, param, {
          on: function (ev, data) {
            if (ev === 'log') log(data);
            else if (ev === 'error') log('ERROR ' + data);
            else if (ev === 'open') log('network open');
            else if (ev === 'close') log('network close');
            else if (ev === 'sync_error') log('FATAL sync_error');
            else log(ev + (data != null ? ' ' + data : ''));
          }
        });
        // also kick legacy peer_glue for Rust BroadcastChannel path
        var role = (param && param.role) || 'active';
        var room = (param && (param.room || param.id1)) || 'default';
        if (typeof w.__flf_peer_connect === 'function') {
          w.__flf_peer_connect(serverAddress, role, room);
        }
      },
      function (err) {
        log(String(err));
      }
    );
  };

  w.__flf_lobby = { connect: connect, openLobby: openLobby, startNetwork: startNetwork };
})(window);
