<script>
import Room from "/src/pages/Room.svelte"

export let roomId = "";
let joined = false;

let room = {};
let error = "";
let ready = false;
fetch(`/api/v1/rooms/${roomId}`, {
  method: "GET",
  headers: {
    "Accept": "application/json",
  }
}).then((response) => {
  if(response.ok) {
    response.text().then((body) => {
      room = JSON.parse(body);
      ready = true;
    });
  } else {
    response.text().then((body) => {
      error = body;
      ready = true;
    });
  }
});

function onJoin() {
  joined = true;
}

</script>

{#if joined}
  <Room roomId={roomId}></Room>
{:else}
  <div class="w-full h-full grow bg-slate-50 flex justify-center">
    <div class="max-w-md w-full flex flex-col justify-center">
      <div class="w-full bg-slate-100 p-4 border-slate-200 rounded border">
        {#if ready}
          {#if error.length > 0}
            <p>{error}</p>
          {:else}
            <h1 class="text-2xl">Welcome to, {room.title}!</h1>
            <h2 class="text-lg text-slate-500 mb-2 leading-none">{room.id}</h2>
            <button on:click={onJoin} class="p-1 w-full border-amber-500 border rounded bg-amber-400 hover:bg-amber-500">Join Room</button>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}