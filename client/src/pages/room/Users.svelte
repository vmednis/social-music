<script>
  import { socket } from "./socket.js";

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

<div class="w-full max-w-xs h-full max-h-full grow-0">
  <div class="w-full flex flex-row">
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showQueue} on:click|preventDefault={() => switchTab("queue")} href="#queue">Queue</a>
    <a class="flex-1 justify-self-stretch text-center border-0 border-x" class:border-b-2={!showPresences} on:click|preventDefault={() => switchTab("presences")} href="#presences">Users Online</a>
  </div>
  {#if showQueue}
    <div>
      <ol>
        {#each $socket.queue as user}
          <li>{user}</li>
        {/each}
      </ol>
    </div>
  {/if}
  {#if showPresences}
    <div>
      <ul>
        {#each $socket.presences as user}
          <li>{user}</li>
        {/each}
      </ul>
    </div>
  {/if}
</div>