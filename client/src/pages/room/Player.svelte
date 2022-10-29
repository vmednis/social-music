<script>
  import { socket } from "./socket.js";

  let ready = false;
  let track = "unknown";
  let artists = "unknown";
  let cover = "";

  let token = "";
  const script = document.createElement("script");
  script.src = "https://sdk.scdn.co/spotify-player.js";
  script.async = true;

  let socket_ready = false;
  socket.subscribe(({ready}) => {
    if(!socket_ready && ready) {
      fetch("/token").then((response) => response.text()).then((body) => {
        token = body;
        document.body.appendChild(script)
      });
      socket_ready = true;
    }
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
      ready = true;
    });

    player.addListener('not_ready', ({ device_id }) => {
      console.log('Device ID has gone offline', device_id);
    });

    player.addListener('player_state_changed', (state) => {
      let current_item = state.context.metadata.current_item;
      track = current_item.name;
      artists = current_item.artists.map(({ name }) => name).join(", ");
      cover = current_item.images
        .map(({height, width, url}) => {return {size: height * width, url};})
        .sort((left, right) => right.size - left.size)
        .reduce((acc, val) => val.size > 32 * 32 ? val : acc, {url: ""})
        .url;

      console.log('Currently Playing', state);
    });

    player.addListener('autoplay_failed', () => {
      console.log('Autoplay is not allowed by the browser autoplay rules');
    });

    player.connect();
  }
</script>

<div class="grow-0 bg-gray-100">
  <div class="flex w-full">
    <div class="w-16 h-16 bg-gray-800 mr-2" style="background-image: url('{cover}')"></div>
    <div class="pt-1">
      {#if ready}
        <b class="text-xl">{track}</b>
        <p class="text-lg">{artists}</p>
      {:else}
        <p>Initializing player...</p>
      {/if}
    </div>
  </div>
</div>

<style>
</style>