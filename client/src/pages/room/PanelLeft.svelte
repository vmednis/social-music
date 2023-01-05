<script>
  import Search from "/src/pages/room/Search.svelte";
  import UserQueue from "/src/pages/room/UserQueue.svelte";

  export let roomId = "";

  let showSearch = true;
  let showQueue = false;

  function switchTab(tab) {
    showSearch = false;
    showQueue = false;

    if(tab == "search") {
      showSearch = true;
    }

    if(tab == "queue") {
      showQueue = true;
    }
  }
</script>

<div class="w-full max-w-md h-full max-h-full grow-0 flex flex-col bg-slate-100">
  <div class="w-full flex flex-row">
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showSearch} class:bg-slate-200={!showSearch} on:click|preventDefault={() => switchTab("search")} href="#search">Search</a>
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showQueue} class:bg-slate-200={!showQueue} on:click|preventDefault={() => switchTab("queue")} href="#user-queue">Queue</a>
  </div>
  {#if showSearch}
    <Search/>
  {/if}
  {#if showQueue}
    <UserQueue roomId={roomId}/>
  {/if}
</div>