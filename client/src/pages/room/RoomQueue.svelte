<script>
  import { socket } from "./socket.js";
</script>

<div>
  <ol>
    {#each $socket.queue as user}
      <li>{user}</li>
    {/each}
  </ol>
</div>