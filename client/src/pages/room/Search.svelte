<script>
  import Fa from 'svelte-fa';
  import { faPlus } from '@fortawesome/free-solid-svg-icons';
  import { socket } from "./socket.js";

  let query = "";
  let tracks = [];

  function search() {
    if(query) {
      let params = new URLSearchParams({
        q: query,
      });
      let url = "/api/v1/search?" + params;

      fetch(url, {
        method: "GET",
        headers: {
          "Accept": "application/json",
        },
      }).then((response) => response.text())
      .then((body) => {
        tracks = JSON.parse(body);
      });
    }
  }

  function onSearchBarKeyDown(event) {
    if(event.key == "Enter") {
      search();
    }
  }

  function queueSong(uri) {
    socket.sendQueueSong(uri);
  }
</script>

<div class="h-full max-h-full flex flex-col">
  <div class="w-full flex flex-row">
    <input type="text" placeholder="Search query..." class="flex-1" bind:value={query} on:keydown={onSearchBarKeyDown}/>
    <button class="flex-0 border-amber-500 border rounded bg-amber-400 hover:bg-amber-500 active:bg-amber-600 text-center px-2" on:click={search}>Search</button>
  </div>
  <div class="overflow-y-scroll w-full flex-grow flex-shrink">
    {#each tracks as track}
      <div class="flex flex-row border-b-2">
        <div class="w-16 h-16 bg-slate-800 bg-center bg-cover flex basis-16" style="background-image: url('{track.cover}')"></div>
        <div class="flex-grow basis-0 overflow-x-hidden ml-1 mr-1">
          <p class="whitespace-nowrap font-semibold text-lg mt-1">{track.name}</p>
          <p class="whitespace-nowrap">{track.artists}</p>
        </div>
        <div class="w-8 h-16 flex basis-8 hover:bg-slate-200 active:bg-slate-300">
          <button class="w-full" on:click={() => queueSong(track.uri)}>
            <Fa icon={faPlus} size="lg" class="w-full"/>
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>