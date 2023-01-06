<script>
  import { socket } from "./socket.js";

  function onJoin() {
    socket.sendJoinQueue();
  }
</script>

<div>
  <button on:click={onJoin} class="p-1 w-full border-amber-500 border rounded bg-amber-400 hover:bg-amber-500 active:bg-amber-600">Join Queue</button>
  <ol>
    {#each $socket.queue as user}
      <li>{user}</li>
    {/each}
  </ol>
</div>