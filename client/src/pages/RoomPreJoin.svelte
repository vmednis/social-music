<script>
import Room from "/src/pages/Room.svelte"

export let roomId = "";
let joined = false;

function onJoin() {
  joined = true;
}

</script>

{#if joined}
  <Room roomId={roomId}></Room>
{:else}
  <button on:click={onJoin}>Join Room</button>
{/if}