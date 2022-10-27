import { data } from "autoprefixer";
import { writable } from "svelte/store";

function createSocket() {
  const { subscribe, update } = writable({
    ws: null,
    ready: false,
    messages: []
  });

  const ws = new WebSocket("ws://127.0.0.1:3030/chat");
  update((data) => {
    data.ws = ws;
    return data;
  });

  ws.addEventListener('open', (event) => {
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
  });

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

  const sendPlaySong = (track_id) => {
    update((data) => {
      let json = JSON.stringify({
        PlaySong: {
          track_id
        }
      });
      data.ws.send(json);
      return data;
    })
  }

  return {
    subscribe,
    sendChatMessage,
    sendSetDevice,
    sendPlaySong,
  }
}

export const socket = createSocket();