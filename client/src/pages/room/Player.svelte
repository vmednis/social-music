<script>
  import { socket } from "./socket.js";

  let track = "unknown";
  let artists = "unknown";

  const script = document.createElement("script");
  script.src = "https://sdk.scdn.co/spotify-player.js";
  script.async = true;

  let token = "";
  fetch("/token").then((response) => response.text()).then((body) => {
    token = body;
    document.body.appendChild(script)
  });

  window.onSpotifyWebPlaybackSDKReady = () => {
    const player = new window.Spotify.Player({
      name: "Social Music Thingy",
      getOAuthToken: cb => {cb(token);},
      volume: 1.0,
    });

    player.addListener('ready', ({ device_id }) => {
      console.log('Ready with Device ID', device_id);
      socket.sendSetDevice(device_id);
    });

    player.addListener('not_ready', ({ device_id }) => {
      console.log('Device ID has gone offline', device_id);
    });

    player.addListener('player_state_changed', (state) => {
      let current_item = state.context.metadata.current_item;
      track = current_item.name;
      artists = current_item.artists.map(({ name }) => name).join(", ");
      console.log('Currently Playing', state);
    });

    player.addListener('autoplay_failed', () => {
      console.log('Autoplay is not allowed by the browser autoplay rules');
    });

    player.connect();
  }
</script>

<div class="grow-0">
  <p>Currently playing: <b>{track}</b> by <b>{artists}</b></p>
</div>

<style>
</style>