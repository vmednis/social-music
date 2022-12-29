<script>
  import {onDestroy} from "svelte";
  import { socket } from "./socket.js";
  import SpotifyLogo from "/src/assets/spotify-icon-black.png";

  let ready = false;
  let track = "unknown";
  let artists = "unknown";
  let cover = "";
  let playing = false;
  let position = 0;
  let duration = Infinity;
  $: progress = position / duration * 100;

  if(!document.body.querySelector("#spotify-script")) {
    //Only add a new player if none exists
    const script = document.createElement("script");
    script.src = "https://sdk.scdn.co/spotify-player.js";
    script.async = true;
    script.id = "spotify-script";

    document.body.appendChild(script);
  } else {
    //Manually retrigger ready
    window.onSpotifyWebPlaybackSDKReady();
  }

  let socket_ready = false;
  socket.subscribe(({ready}) => {
    if(!socket_ready && ready) {
      socket_ready = true;
    }
  });

  const whenSocketReady = (func) => {
    if(socket_ready) {
      func();
    } else {
      setTimeout(whenSocketReady, 250, func);
    }
  }

  const kill_player = {
    on_kill: null,
    kill: () => {
      if(kill_player.on_kill) {
        kill_player.on_kill();
      }
    }
  };

  window.onSpotifyWebPlaybackSDKReady = () => {
    const player = new window.Spotify.Player({
      name: "Social Music Thingy",
      getOAuthToken: cb => {
        fetch("/token").then((response) => response.text()).then((token) => {
          cb(token)
        });
      },
      volume: 1.0,
    });

    player.addListener('ready', ({ device_id }) => {
      console.log('Ready with Device ID', device_id);
      whenSocketReady(() => {
        socket.sendSetDevice(device_id);
        ready = true;
      })
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
        .reduce((acc, val) => val.size >= 64 * 64 ? val : acc, {url: ""})
        .url;

      playing = !state.paused;
      position = state.position;
      duration = state.duration;

      console.log('Currently Playing', state);
    });

    player.addListener('autoplay_failed', () => {
      console.log('Autoplay is not allowed by the browser autoplay rules');
    });

    kill_player.on_kill = () => {
      player.disconnect();
      console.log("Disconnecting player");
    };

    player.connect();
  }

  const progressStep = 100;
  const advanceProgress = () => {
    if(playing) {
      position += progressStep;
    }
    setTimeout(advanceProgress, progressStep);
  }
  advanceProgress();

  onDestroy(() => {
    kill_player.kill();
  });
</script>

<div class="grow-0 bg-slate-100">
  <div class="flex w-full">
    <div class="w-16 h-16 bg-slate-800 bg-center bg-cover" style="background-image: url('{cover}')"></div>
    <div class="w-8 h-8 m-4 bg-center bg-cover" style="background-image: url('{SpotifyLogo}')"></div>
    <div class="pt-1">
      {#if ready}
        <b class="text-xl">{track}</b>
        <p class="text-lg">{artists}</p>
      {:else}
        <p>Initializing player...</p>
      {/if}
    </div>
  </div>
  <div class="w-full bg-slate-300 h-1">
    <div class="bg-amber-400 h-1 transition-[width] ease-linear duration-[100ms]" style="width: {progress}%;"></div>
  </div>
</div>