<script>
  import RoomQueue from "/src/pages/room/RoomQueue.svelte"
  import Presences from "/src/pages/room/Presences.svelte"

  let showQueue = true;
  let showPresences = false;

  function switchTab(tab) {
    showQueue = false;
    showPresences = false;

    if(tab == "queue") {
      showQueue = true;
    }

    if(tab == "presences") {
      showPresences = true;
    }
  }
</script>

<div class="w-full max-w-xs h-full max-h-full grow-0 bg-slate-100">
  <div class="w-full flex flex-row">
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showQueue} class:bg-slate-200={!showQueue} on:click|preventDefault={() => switchTab("queue")} href="#room-queue">Up Next</a>
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showPresences} class:bg-slate-200={!showPresences} on:click|preventDefault={() => switchTab("presences")} href="#presences">Users Online</a>
  </div>
  {#if showQueue}
    <RoomQueue/>
  {/if}
  {#if showPresences}
    <Presences/>
  {/if}
</div>