<script>
  import { socket } from "./socket.js";
</script>

<div class="w-full max-w-xs h-full max-h-full grow-0">
  <div>
    <b>Queue: (use /join)</b>
    <ol>
      {#each $socket.queue as user}
        <li>{user}</li>
      {/each}
    </ol>
  </div>
  <div>
    <b>Presences:</b>
    <ul>
      {#each $socket.presences as user}
        <li>{user}</li>
      {/each}
    </ul>
  </div>
</div>