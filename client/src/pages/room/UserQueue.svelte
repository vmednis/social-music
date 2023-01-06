<script>
	import { onDestroy } from 'svelte';
  import { socket } from "/src/pages/room/socket.js";

  export let roomId = "";

  let tracks = [];
  let ready = false;

  function getQueue() {
    fetch("/api/v1/queues/" + roomId, {
      method: "GET",
      headers: {
        "Accept": "application/json",
      },
    }).then((response) => response.text())
    .then((body) => {
      tracks = JSON.parse(body);
      ready = true;
    });
  }

  let queueChange = null;
  const unsubscribe = socket.subscribe((data) => {
    if(queueChange != data.queueChange) {
      //If there's been a change to the queue or it's our first time here getQueue
      queueChange = data.queueChange;
      getQueue();
    }
  })

  onDestroy(unsubscribe);
</script>

<div class="h-full flex">
  {#if ready}
    {#if tracks.length == 0}
      <p>Your Queue is empty, add some songs through the Search tab.</p>
    {:else}
      <div class="overflow-y-scroll w-full flex-grow flex-shrink">

        {#each tracks as track}
          <div class="flex flex-row border-b-2">
            <div class="w-16 h-16 bg-slate-800 bg-center bg-cover flex basis-16" style="background-image: url('{track.cover}')"></div>
            <div class="flex-grow basis-0 overflow-x-hidden ml-1 mr-1">
              <p class="whitespace-nowrap font-semibold text-lg mt-1">{track.name}</p>
              <p class="whitespace-nowrap">{track.artists}</p>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>