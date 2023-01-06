import { writable } from "svelte/store";

function createSocket() {
  const { subscribe, update } = writable({
    ws: null,
    ready: false,
    messages: [],
    queue: [],
    presences: [],
    queueChange: 0,
  });

  const init = (roomId) => {
    let protocol = "ws:";
    if (location.protocol == "https:") {
      protocol = "wss:"
    }
    const ws = new WebSocket(`${protocol}//${location.host}/chat/${roomId}`);

    ws.addEventListener('open', (event) => {
      let keepAlive = () => {
        let data = crypto.randomUUID();
        let json = JSON.stringify({
          KeepAlivePing: {
            data
          }
        });
        ws.send(json)

        setTimeout(keepAlive, 15000);
      }
      keepAlive();

      update((data) => {
        data.ready = true;
        return data;
      });
    });

    ws.addEventListener('message', (event) => {
      let message = JSON.parse(event.data);
      if(message.ChatMessage) {
        update((data) => {
          data.messages.push(message.ChatMessage)
          return data;
        });
      }
      if(message.PresencesQueueMessage) {
        update((data) => {
          data.queue = message.PresencesQueueMessage.queue;
          data.presences = message.PresencesQueueMessage.presences;
          return data;
        });
      }
      if(message == "UserQueueChange") {
        update((data) => {
          data.queueChange += 1;
          return data;
        });
      }
    });

    update((data) => {
      data.ws = ws;
      return data;
    });
  };

  const close = () => {
    update((data) => {
      data.ready = false;
      data.ws.close();

      return data;
    });
  };

  const sendChatMessage = (message) => {
    update((data) => {
      let json = JSON.stringify({
        ChatMessage: {
          message
        }
      });
      data.ws.send(json);
      return data;
    })
  }

  const sendSetDevice = (device_id) => {
    update((data) => {
      let json = JSON.stringify({
        SetDevice: {
          device_id
        }
      });
      data.ws.send(json);
      return data;
    })
  }

  const sendQueueSong = (track_id) => {
    update((data) => {
      let json = JSON.stringify({
        QueueSong: {
          track_id
        }
      });
      data.ws.send(json);
      return data;
    })
  }

  const sendJoinQueue = () => {
    update((data) => {
      let json = JSON.stringify({
        JoinQueue: null
      });
      data.ws.send(json);
      return data;
    })
  }

  return {
    subscribe,
    init,
    close,
    sendChatMessage,
    sendSetDevice,
    sendQueueSong,
    sendJoinQueue,
  }
}

export const socket = createSocket();