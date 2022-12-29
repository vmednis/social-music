<script>
  import { socket } from "./socket.js";
</script>

<div>
  <ul>
    {#each $socket.presences as user}
      <li>{user}</li>
    {/each}
  </ul>
</div>